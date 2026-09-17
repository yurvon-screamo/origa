#!/usr/bin/env python3
"""Full generation run: N parallel workers over all staging rules.

- Resumable: rules with an existing staging/gen/<rid>.json are skipped.
- 429/5xx storms shrink the pool (64 -> 32 -> 16) and back off.
- Each success is logged to request_log.jsonl (quota tracking) and the
  local checkers run inline so FAILs surface immediately.
- No retries beyond transport errors: regeneration passes are separate
  decisions (quota is 1000, the base run costs ~677).

Usage: python scripts/run_generation.py [--workers 64] [--limit N]
"""

import argparse
import json
import os
import re
import sys
import threading
import time
import urllib.error
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from check_generation import run as run_checks  # noqa: E402
from generate_content import (  # noqa: E402
    SYSTEM_PROMPT,
    build_user_prompt,
    extract_json,
)

STAGING = Path("/tmp/opencode/staging")
API = "https://openrouter.ai/api/v1/chat/completions"
MODEL = "stealth/union-alpha"

_print_lock = threading.Lock()
_log_lock = threading.Lock()
_state_lock = threading.Lock()
consecutive_429 = 0


def say(msg: str) -> None:
    with _print_lock:
        print(msg, flush=True)


def log_request(kind: str, rid: str, ok: bool, info: str = "") -> None:
    with _log_lock:
        with open(STAGING / "request_log.jsonl", "a", encoding="utf-8") as f:
            f.write(
                json.dumps(
                    {"ts": time.time(), "kind": kind, "rule_id": rid, "ok": ok, "info": info},
                    ensure_ascii=False,
                )
                + "\n"
            )


def call_model(user_prompt: str) -> dict:
    req = urllib.request.Request(
        API,
        data=json.dumps(
            {
                "model": MODEL,
                "messages": [
                    {"role": "system", "content": SYSTEM_PROMPT},
                    {"role": "user", "content": user_prompt},
                ],
                "temperature": 0.4,
                "max_tokens": 6000,
            }
        ).encode(),
        headers={
            "Authorization": f"Bearer {os.environ['OPENROUTER_API_KEY']}",
            "Content-Type": "application/json",
        },
    )
    with urllib.request.urlopen(req, timeout=600) as resp:
        body = json.loads(resp.read())
    return {
        "content": body["choices"][0]["message"]["content"],
        "usage": body.get("usage", {}),
    }


def worker(rid: str, pool: list[int], stats: dict) -> None:
    global consecutive_429
    gen_path = STAGING / "gen" / f"{rid}.json"
    if gen_path.exists():
        with _state_lock:
            stats["skipped"] += 1
        return
    try:
        ctx = json.loads((STAGING / "contexts" / f"{rid}.json").read_text(encoding="utf-8"))
    except FileNotFoundError:
        say(f"[NOCTX] {rid}")
        with _state_lock:
            stats["failed"] += 1
        return

    prompt = build_user_prompt(ctx)
    for attempt in range(3):
        try:
            t0 = time.time()
            result = call_model(prompt)
            parsed = extract_json(result["content"])
            gen_path.write_text(
                json.dumps(parsed, ensure_ascii=False, indent=1), encoding="utf-8"
            )
            log_request("generate", rid, True, json.dumps(result["usage"].get("total_tokens", "")))
            with _state_lock:
                consecutive_429 = 0
                stats["done"] += 1
                n = stats["done"]
            checks = run_checks(rid)
            mark = "PASS" if checks["ok"] else "FAIL"
            say(
                f"[{mark}] {n}/{stats['total']} {rid} "
                f"({time.time()-t0:.0f}s)"
                + ("" if checks["ok"] else f" issues={checks['schema'] + checks['copy'] + checks['examples_audit']['errors']}")
            )
            if not checks["ok"]:
                with _state_lock:
                    stats["check_fail"] += 1
            return
        except urllib.error.HTTPError as e:
            if e.code in (429, 502, 503, 529):
                with _state_lock:
                    consecutive_429 += 1
                    if consecutive_429 >= 8 and pool[0] > 16:
                        pool[0] = max(16, pool[0] // 2)
                        say(f"~~ throttling: pool -> {pool[0]}")
                    wait = min(120, 15 * attempt + 15)
                time.sleep(wait)
                continue
            log_request("generate", rid, False, f"HTTP {e.code}")
            with _state_lock:
                stats["failed"] += 1
            say(f"[HTTP{e.code}] {rid}")
            return
        except (urllib.error.URLError, json.JSONDecodeError, KeyError, ValueError) as e:
            log_request("generate", rid, False, str(e)[:120])
            time.sleep(5)
            continue
    with _state_lock:
        stats["failed"] += 1
    say(f"[GAVEUP] {rid}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workers", type=int, default=64)
    parser.add_argument("--limit", type=int, default=0)
    args = parser.parse_args()

    corpus = json.loads((STAGING / "grammar_v2.pruned.json").read_text(encoding="utf-8"))
    rids = [r["rule_id"] for r in corpus["grammar"]]
    if args.limit:
        rids = rids[: args.limit]

    stats = {"total": len(rids), "done": 0, "skipped": 0, "failed": 0, "check_fail": 0}
    pool = [args.workers]
    t0 = time.time()
    say(f"run: {len(rids)} rules, workers={args.workers}")

    threads: list[threading.Thread] = []
    active = 0
    i = 0
    while i < len(rids) or active > 0:
        while i < len(rids) and active < pool[0]:
            t = threading.Thread(target=worker, args=(rids[i], pool, stats), daemon=True)
            t.start()
            threads.append(t)
            i += 1
            active += 1
        threads = [t for t in threads if t.is_alive()]
        active = len(threads)
        time.sleep(0.25)

    dt = time.time() - t0
    say(
        f"DONE in {dt/60:.1f}min: done={stats['done']} skipped={stats['skipped']} "
        f"failed={stats['failed']} check_fail={stats['check_fail']}"
    )
    return 0 if stats["failed"] == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
