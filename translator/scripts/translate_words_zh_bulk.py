#!/usr/bin/env python3
"""Mass JA-word -> zh-Hans glosses for the dictionary (97k words).

Output per word: 1-3 short dictionary glosses (semicolon-separated),
parsed into a t-list compatible with cdn dictionary schema.
"""
import json
import os
import re
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import requests

CDN = sorted(Path("/home/yurvon/origa/cdn/dictionary").glob("chunk_*.json"))
OUT = Path("/home/yurvon/origa-translator/data/pipeline/zh_words")
OUT.mkdir(parents=True, exist_ok=True)
URL = "http://127.0.0.1:8091/v1/chat/completions"
BLOCK, CONC = 64, 12

SYS = "You are a skilled Japanese lexicographer. Give concise, natural dictionary translations."
INSTR = ("Translate the Japanese word/expression into Chinese (Simplified). "
         "Give 1-3 short dictionary glosses separated by ';', most common first. "
         "No explanations, no romanization, Chinese only:\n{x}")

RE_HAN = re.compile(r"[一-鿿]")
RE_KANA = re.compile(r"[ぁ-ゖァ-ヺーｦ-ﾟゝゞ]")


def parse_glosses(out: str, word: str):
    parts = [p.strip(' 。.．、\t') for p in re.split(r'[;；]', out)]
    parts = [p for p in parts if p]
    glosses = []
    seen = set()
    for p in parts:
        if RE_KANA.search(p) or not RE_HAN.search(p) or p == word or p in seen:
            continue
        seen.add(p)
        if len(p) > 40:
            p = p[:40]
        glosses.append(p)
        if len(glosses) >= 3:
            break
    return glosses


def gen_one(word: str):
    r = requests.post(URL, json={
        "model": "tr600k",
        "messages": [{"role": "system", "content": SYS},
                     {"role": "user", "content": INSTR.format(x=word)}],
        "temperature": 0.0, "max_tokens": 80}, timeout=120)
    r.raise_for_status()
    out = r.json()["choices"][0]["message"]["content"].strip()
    return parse_glosses(out, word)


def atomic_checkpoint(done: int):
    tmp = OUT / "progress.json.tmp"
    tmp.write_text(json.dumps({"done": done}))
    os.replace(tmp, OUT / "progress.json")


def reconcile() -> int:
    path = OUT / "zh_words.jsonl"
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
    words = []
    for f in CDN:
        words += list(json.load(open(f, encoding="utf-8")).keys())
    print(f"words: {len(words)}", flush=True)
    done = reconcile()
    print(f"resume from {done}", flush=True)

    fh = open(OUT / "zh_words.jsonl", "a", encoding="utf-8")
    t0 = time.perf_counter()
    n = empty = 0
    for start in range(done, len(words), BLOCK):
        block = words[start:start + BLOCK]
        with ThreadPoolExecutor(CONC) as ex:
            gens = list(ex.map(gen_one, block))
        for w, gl in zip(block, gens):
            fh.write(json.dumps({"word": w, "zh_t": gl}, ensure_ascii=False) + "\n")
            n += 1
            if not gl:
                empty += 1
        fh.flush()
        os.fsync(fh.fileno())
        atomic_checkpoint(done + len(block))
        done += len(block)
        dt = time.perf_counter() - t0
        if (start // BLOCK) % 20 == 0:
            print(f"[{time.strftime('%H:%M:%S')}] {done}/{len(words)} | {n/max(dt,1):.2f} w/s "
                  f"| empty {empty}", flush=True)
    print("COMPLETE", flush=True)


if __name__ == "__main__":
    main()
