#!/usr/bin/env python3
"""Mass JA->zh-Hans translation of word-usage examples (cdn/examples, 23k).

Input rows: {"i": int, "x": "JP sentence", "en"/"ru"/"vi"/"ko": ..., "f": furigana}
Output: ex_zh.jsonl {"i", "x", "zh", "zh_ok"}
"""

import json
import os
import re
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import requests

SRC = sorted(Path("/home/yurvon/origa/cdn/examples/data").glob("s*.json"))
OUT = Path("/home/yurvon/origa-translator/data/pipeline/zh_examples")
OUT.mkdir(parents=True, exist_ok=True)
URL = "http://127.0.0.1:8091/v1/chat/completions"
BLOCK, CONC = 64, 12

SYS = "You are a skilled Japanese translator. Translate the sentence accurately and naturally."
INSTR = "Translate the following Japanese sentence into Chinese (Simplified):\n{x}"

RE_HAN = re.compile(r"[一-鿿㐀-䶿]")
RE_KANA = re.compile(r"[ぁ-ゖァ-ヺーｦ-ﾟゝゞヽヾ]")
RE_TRAD_ONLY = re.compile(
    r"[們來時說話氣車馬鳥龍門問間東書長開關電語體歡戀絕豐廣歸續點實現覺識斷]"
)


def detect_zh(text: str, jp: str = "") -> bool:
    t = (text or "").strip()
    if not t:
        return False
    if len(RE_KANA.findall(t)) / max(len(t), 1) >= 0.02:
        return False
    if len(RE_HAN.findall(t)) / max(len(t), 1) <= 0.15:
        return False
    if RE_TRAD_ONLY.search(t):
        return False
    if jp:
        common = len(set(t) & set(jp))
        if common / max(len(set(t)), 1) >= 0.8 and len(t) <= len(jp) * 1.6:
            return False
    return True


def gen_one(x: str):
    r = requests.post(
        URL,
        json={
            "model": "tr600k",
            "messages": [
                {"role": "system", "content": SYS},
                {"role": "user", "content": INSTR.format(x=x)},
            ],
            "temperature": 0.0,
            "max_tokens": 200,
        },
        timeout=180,
    )
    r.raise_for_status()
    return r.json()["choices"][0]["message"]["content"].strip()


def atomic_checkpoint(done: int):
    tmp = OUT / "progress.json.tmp"
    tmp.write_text(json.dumps({"done": done}))
    os.replace(tmp, OUT / "progress.json")


def reconcile() -> int:
    path = OUT / "ex_zh.jsonl"
    if not path.exists():
        return 0
    good_bytes, n = 0, 0
    with open(path, "rb") as f:
        for raw in f:
            try:
                json.loads(raw)
            except Exception:
                break
            good_bytes += len(raw)
            n += 1
    if good_bytes < path.stat().st_size:
        with open(path, "r+b") as f:
            f.truncate(good_bytes)
            f.flush()
            os.fsync(f.fileno())
    prog = 0
    pp = OUT / "progress.json"
    if pp.exists():
        prog = json.loads(pp.read_text())["done"]
    if prog > n:
        raise SystemExit(f"FATAL: progress={prog} > строк={n}")
    if prog < n:
        atomic_checkpoint(n)
        prog = n
    return prog


def main():
    rows = []
    for f in SRC:
        rows += json.load(open(f, encoding="utf-8"))
    print(f"examples: {len(rows)}", flush=True)
    done = reconcile()
    print(f"resume from {done}", flush=True)

    fh = open(OUT / "ex_zh.jsonl", "a", encoding="utf-8")
    t0 = time.perf_counter()
    n = fails = 0
    for start in range(done, len(rows), BLOCK):
        block = rows[start : start + BLOCK]
        with ThreadPoolExecutor(CONC) as ex:
            gens = list(ex.map(lambda r: gen_one(r["x"]), block))
        for r, zh in zip(block, gens):
            ok = detect_zh(zh, r["x"])
            fh.write(
                json.dumps(
                    {"i": r["i"], "x": r["x"], "zh": zh, "zh_ok": ok},
                    ensure_ascii=False,
                )
                + "\n"
            )
            n += 1
            if not ok:
                fails += 1
        fh.flush()
        os.fsync(fh.fileno())
        atomic_checkpoint(done + len(block))
        done += len(block)
        dt = time.perf_counter() - t0
        if (start // BLOCK) % 10 == 0:
            print(
                f"[{time.strftime('%H:%M:%S')}] {done}/{len(rows)} | {n / max(dt, 1):.2f} ex/s "
                f"| fails {fails}",
                flush=True,
            )
    print("COMPLETE", flush=True)


if __name__ == "__main__":
    main()
