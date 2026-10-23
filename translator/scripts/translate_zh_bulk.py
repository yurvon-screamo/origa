#!/usr/bin/env python3
"""Mass JA->zh(Simplified) translation of all cdn phrases.

v2 (post code-review a5c18bc):
- resume: recount jsonl lines, verify vs progress.json, trim torn tail,
  hard-fail on mismatch beyond repair (no silent garbage propagation)
- checkpoint: atomic (tmp + os.replace) with data fsync before it
- detect_zh v2: kanji-echo guard (zh too similar to JP input),
  Traditional-only chars detection, halfwidth kana, CJK Ext-A
"""

import json
import os
import re
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import requests

CDN = sorted(Path("/home/yurvon/origa/cdn/phrases/data").glob("p*.json"))
OUT = Path("/home/yurvon/origa-translator/data/pipeline/zh")
OUT.mkdir(parents=True, exist_ok=True)
URL = "http://127.0.0.1:8091/v1/chat/completions"
BLOCK, CONC = 64, 12

SYS_EN = "You are a skilled Japanese translator. Translate the sentence accurately and naturally."
SYS_JA = "あなたは熟練した日本語翻訳者です。文を正確かつ自然に翻訳してください。"
INSTR_EN = "Translate the following Japanese sentence into Chinese (Simplified):\n{x}"
INSTR_JA = "次の日本語の文を中国語（簡体字）に翻訳してください:\n{x}"

# ranges: CJK Unified (incl. ext ranges) / kana incl. halfwidth & iteration marks
RE_HAN = re.compile(r"[一-鿿㐀-䶿]")
RE_KANA = re.compile(r"[ぁ-ゖァ-ヺーｦ-ﾟゝゞヽヾ]")
RE_TRAD_ONLY = re.compile(
    r"[們來時說話氣車馬鳥龍門問間東書長開關電語體歡戀絕豐廣歸續點實現覺識斷]"
)
RE_LATIN = re.compile(r"[A-Za-z]")


def _zh_ratio(t: str) -> float:
    return len(RE_HAN.findall(t)) / max(len(t), 1)


def _kana_ratio(t: str) -> float:
    return len(RE_KANA.findall(t)) / max(len(t), 1)


def _echo_ratio(zh: str, x: str) -> float:
    """shared-ALL-chars ratio vs the JP input: high => echo, not translation.
    NB: считать по ВСЕМ символам, не только кандзи — иначе имена-кандзи
    (白百合, общие для JA/ZH) дают ложные эхо-флаги на валидных переводах."""
    xs, zs = set(x), set(zh)
    if not xs or not zs:
        return 0.0
    return len(xs & zs) / max(len(zs), 1)


def detect_zh(text: str, jp: str = "") -> bool:
    t = (text or "").strip()
    if not t:
        return False
    if _kana_ratio(t) >= 0.02:  # any real kana presence -> not zh
        return False
    if _zh_ratio(t) <= 0.15:  # mostly non-CJK junk
        return False
    if RE_TRAD_ONLY.search(t):  # Traditional chars in a Simplified run
        return False
    if jp and _echo_ratio(t, jp) >= 0.8 and len(t) <= len(jp) * 1.6:
        return False  # kanji echo of the input
    return True


def gen_one(x: str):
    def call(sys, instr):
        r = requests.post(
            URL,
            json={
                "model": "tr600k",
                "messages": [
                    {"role": "system", "content": sys},
                    {"role": "user", "content": instr.format(x=x)},
                ],
                "temperature": 0.0,
                "max_tokens": 200,
            },
            timeout=180,
        )
        r.raise_for_status()
        return r.json()["choices"][0]["message"]["content"].strip()

    out = call(SYS_EN, INSTR_EN)
    retries = 0
    if not detect_zh(out, x):
        out2 = call(SYS_JA, INSTR_JA)
        retries = 1
        if detect_zh(out2, x) or (
            len(out2) >= len(out) * 0.5 and not detect_zh(out, x)
        ):
            out = out2
    return out, retries


def atomic_checkpoint(done: int):
    tmp = OUT / "progress.json.tmp"
    tmp.write_text(json.dumps({"done": done}))
    os.replace(tmp, OUT / "progress.json")


def reconcile() -> int:
    """Recount valid jsonl lines; trim torn tail; verify against progress."""
    path = OUT / "zh_translations.jsonl"
    if not path.exists():
        return 0
    good_bytes = 0
    n = 0
    with open(path, "rb") as f:
        for raw in f:
            try:
                json.loads(raw)
            except Exception:
                break  # torn tail: cut here
            good_bytes += len(raw)
            n += 1
    size = path.stat().st_size
    if good_bytes < size:
        with open(path, "r+b") as f:
            f.truncate(good_bytes)
            f.flush()
            os.fsync(f.fileno())
    prog = 0
    pp = OUT / "progress.json"
    if pp.exists():
        prog = json.loads(pp.read_text())["done"]
    if prog > n:
        raise SystemExit(
            f"FATAL: progress={prog} > валидных строк={n}; "
            "битый чекпоинт — починить вручную (см. ревью a5c18bc High-1)"
        )
    if prog < n:
        print(f"resume: прогресс {prog} < строк {n} — выравниваю по строкам")
        atomic_checkpoint(n)
        prog = n
    return prog


def main():
    phrases = []
    for f in CDN:
        phrases += json.load(open(f, encoding="utf-8"))
    print(f"phrases: {len(phrases)}", flush=True)

    done = reconcile()
    print(f"resume from {done} (сверено с jsonl)", flush=True)

    fh = open(OUT / "zh_translations.jsonl", "a", encoding="utf-8")
    t0 = time.perf_counter()
    n = retries = langfail = 0
    for start in range(done, len(phrases), BLOCK):
        block = phrases[start : start + BLOCK]
        with ThreadPoolExecutor(CONC) as ex:
            gens = list(ex.map(lambda p: gen_one(p["x"]), block))
        for p, (zh, r) in zip(block, gens):
            fh.write(
                json.dumps(
                    {
                        "i": p["i"],
                        "x": p["x"],
                        "zh": zh,
                        "retries": r,
                        "zh_ok": detect_zh(zh, p["x"]),
                    },
                    ensure_ascii=False,
                )
                + "\n"
            )
            n += 1
            retries += r
            if not detect_zh(zh, p["x"]):
                langfail += 1
        fh.flush()
        os.fsync(fh.fileno())  # data durable BEFORE checkpoint
        atomic_checkpoint(done + len(block))
        done += len(block)
        dt = time.perf_counter() - t0
        if (start // BLOCK) % 10 == 0:
            print(
                f"[{time.strftime('%H:%M:%S')}] {done}/{len(phrases)} | {n / max(dt, 1):.2f} ph/s "
                f"| retry {retries} langfail {langfail}",
                flush=True,
            )
    print("COMPLETE", flush=True)


if __name__ == "__main__":
    main()
