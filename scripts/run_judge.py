#!/usr/bin/env python3
"""Judge wave: batched review of generated rules (OpenRouter, ~64 requests).

Queue: ALL no-reference rules (nothing grounds them but the judge and the
manual pass) + a random 10% sample of referenced rules. Batches of 3 rules
per request keep the quota: 136 + ~54 ≈ 190 rules ≈ 64 requests.

Verdict per rule: {"rule_id", "ok", "confidence", "issues"}.
Failed transports retry once; repeated failures land in judge_failed.txt.

Usage: python scripts/run_judge.py [--batch 3] [--workers 8]
"""

import argparse
import json
import os
import random
import re
import sys
import threading
import time
import urllib.error
import urllib.request
from pathlib import Path

STAGING = Path("/tmp/opencode/staging")
API = "https://openrouter.ai/api/v1/chat/completions"
MODEL = "stealth/union-alpha"

SYSTEM = """\
You are a strict Japanese-grammar reviewer. For EACH rule below you get:
reference facts (may be absent — then judge from your own knowledge of
Japanese grammar) and the generated content (Japanese examples plus the
English and Russian fields). Verify FACTUAL correctness:
- meaning/usage categories are correct and complete for the pattern,
- the formation table is grammatically right,
- every Japanese example uses the target pattern naturally,
- the English and Russian example translations are faithful.
Answer STRICT JSON: {"verdicts": [{"rule_id": ..., "ok": <bool>,
"confidence": <0..1>, "issues": ["<short>"]}, …]} — one per rule, same order."""


def extract_json(content: str) -> dict:
    s = re.sub(r"^```(?:json)?\s*|\s*```$", "", content.strip(), flags=re.S)
    start, end = s.find("{"), s.rfind("}")
    return json.loads(s[start : end + 1])


def rule_payload(rid: str) -> dict:
    ctx = json.loads((STAGING / "contexts" / f"{rid}.json").read_text(encoding="utf-8"))
    gen = json.loads((STAGING / "gen" / f"{rid}.json").read_text(encoding="utf-8"))
    return {
        "rule": ctx["rule"],
        "detection": ctx["detection"],
        "reference": ctx["reference"],
        "generated": {
            "jp_examples": gen.get("jp_examples"),
            "English": {k: gen.get("English", {}).get(k)
                        for k in ("short_description", "explanation", "how_to_form")},
            "Russian": {k: gen.get("Russian", {}).get(k)
                        for k in ("short_description", "explanation")},
        },
    }


def judge_batch(batch: list[str], stats: dict) -> None:
    user = json.dumps([rule_payload(r) for r in batch], ensure_ascii=False, indent=1)
    for attempt in range(2):
        try:
            req = urllib.request.Request(
                API,
                data=json.dumps(
                    {
                        "model": MODEL,
                        "messages": [
                            {"role": "system", "content": SYSTEM},
                            {"role": "user", "content": user},
                        ],
                        "temperature": 0.1,
                        "max_tokens": 2400,
                    }
                ).encode(),
                headers={
                    "Authorization": f"Bearer {os.environ['OPENROUTER_API_KEY']}",
                    "Content-Type": "application/json",
                },
            )
            with urllib.request.urlopen(req, timeout=600) as resp:
                body = json.loads(resp.read())
            verdicts = extract_json(body["choices"][0]["message"]["content"]).get("verdicts", [])
            out_dir = STAGING / "judge"
            out_dir.mkdir(exist_ok=True)
            for v in verdicts:
                rid = v.get("rule_id")
                if rid:
                    (out_dir / f"{rid}.json").write_text(
                        json.dumps(v, ensure_ascii=False, indent=1), encoding="utf-8"
                    )
            with open(STAGING / "request_log.jsonl", "a", encoding="utf-8") as f:
                f.write(json.dumps({"ts": time.time(), "kind": "judge-batch",
                                    "rule_id": ",".join(batch), "ok": True,
                                    "info": str(len(verdicts))}) + "\n")
            with threading.Lock():
                pass
            stats["done"] += len(batch)
            print(f"[judge] {stats['done']}/{stats['total']} (+{len(verdicts)})", flush=True)
            return
        except urllib.error.HTTPError as e:
            if e.code in (429, 502, 503, 529):
                time.sleep(30 + 30 * attempt)
                continue
            break
        except (json.JSONDecodeError, KeyError, ValueError):
            time.sleep(5)
            continue
    (STAGING / "judge_failed.txt").write_text(
        (STAGING / "judge_failed.txt").read_text() + "\n" + ",".join(batch)
        if (STAGING / "judge_failed.txt").exists()
        else ",".join(batch)
    )
    stats["failed"] += len(batch)
    print(f"[judge-FAIL] {batch}", flush=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--batch", type=int, default=3)
    parser.add_argument("--workers", type=int, default=8)
    args = parser.parse_args()

    corpus = json.loads((STAGING / "grammar_v2.pruned.json").read_text(encoding="utf-8"))
    refs = json.loads((STAGING / "ref_mapping_pruned.json").read_text(encoding="utf-8"))
    by_id = {r["rule_id"]: r for r in corpus["grammar"]}
    refs_by_id = {x["rule_id"]: x for x in refs}

    judged = {p.stem for p in (STAGING / "judge").glob("*.json")} if (STAGING / "judge").exists() else set()
    no_ref = [rid for rid in by_id if not refs_by_id.get(rid, {}).get("refs")]
    with_ref = [rid for rid in by_id if refs_by_id.get(rid, {}).get("refs")]
    random.seed(42)
    sample = random.sample(with_ref, len(with_ref) // 10)
    queue = [rid for rid in no_ref + sample if rid not in judged]
    batches = [queue[i : i + args.batch] for i in range(0, len(queue), args.batch)]
    print(f"judge queue: {len(queue)} rules ({len(no_ref)} no-ref + {len(sample)} sample) "
          f"-> {len(batches)} requests")

    stats = {"done": 0, "failed": 0, "total": len(queue)}
    threads = []
    for b in batches:
        while len(threads) >= args.workers:
            threads = [t for t in threads if t.is_alive()]
            time.sleep(0.2)
        t = threading.Thread(target=judge_batch, args=(b, stats), daemon=True)
        t.start()
        threads.append(t)
    for t in threads:
        t.join()

    print(f"DONE: judged={stats['done']} failed={stats['failed']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
