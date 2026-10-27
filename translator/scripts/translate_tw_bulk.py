#!/usr/bin/env python3
"""Mass JA->zh-Hant (Taiwan) translation of all cdn phrases.

TW recipe (validated on 8-phrase test): EN instruction with explicit
"Traditional Chinese (Taiwan, 繁體中文)" — stable Hant output.
JA instruction produces mixed script — DO NOT use.
"""

import json
import os
import re
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import requests

CDN = sorted(Path("/home/yurvon/origa/cdn/phrases/data").glob("p*.json"))
OUT = Path("/home/yurvon/origa-translator/data/pipeline/tw")
OUT.mkdir(parents=True, exist_ok=True)
URL = "http://127.0.0.1:8091/v1/chat/completions"
BLOCK, CONC = 64, 12

SYS_EN = "You are a skilled Japanese translator. Translate the sentence accurately and naturally."
INSTR_EN = "Translate the following Japanese sentence into Traditional Chinese (Taiwan, 繁體中文):\n{x}"

RE_HAN = re.compile(r"[一-鿿㐀-䶿]")
RE_KANA = re.compile(r"[ぁ-ゖァ-ヺーｦ-ﾟゝゞヽヾ]")
# частые simplified-only формы (в Hant-выводе их быть НЕ должно)
RE_SIMP_ONLY = re.compile(
    r"[们来时说话气车马鸟龙门问间东书长开关电语体欢恋绝丰广归续点实现觉识断动为与会发说这边还现见机后来头对干么关车门马问间东习乡书导亚产亲们价亿从众优会传统们仪们价]"
)
# традиционные маркеры (хотя бы один должен присутствовать в предложении длиной >6)
RE_TRAD_MARK = re.compile(
    r"[們來時說話氣車馬鳥龍門問間東書長開關電語體歡戀絕豐廣歸續點實現覺識斷動為與會發說這那邊還現見機後來頭對乾麼關車門習鄉書導亞產親價億從眾優傳統儀]"
)

RETRY_INSTR = (
    "次の日本語の文を台湾の繁体字中国語（台湾で使われる表現）に翻訳してください:\n{x}"
)
SYS_JA = "あなたは熟練した日本語翻訳者です。文を正確かつ自然に翻訳してください。"


def _kana_ratio(t):
    return len(RE_KANA.findall(t)) / max(len(t), 1)


def _han_ratio(t):
    return len(RE_HAN.findall(t)) / max(len(t), 1)


def detect_hant(text: str, jp: str = "") -> bool:
    t = (text or "").strip()
    if not t:
        return False
    if _kana_ratio(t) >= 0.02:
        return False
    if _han_ratio(t) <= 0.15:
        return False
    if RE_SIMP_ONLY.search(t):
        return False  # simplified leak в Hant-выводе
    if t == jp:
        return False  # эхо-повтор входа (любая длина)
    # в осмысленном предложении ожидаем хотя бы один trad-маркер либо
    # короткая реплика без trad/simp-различимых символов (имена/междометия)
    if len(t) > 6 and not RE_TRAD_MARK.search(t):
        return False
    return True


def gen_one(x: str):
    r = requests.post(
        URL,
        json={
            "model": "tr600k",
            "messages": [
                {"role": "system", "content": SYS_EN},
                {"role": "user", "content": INSTR_EN.format(x=x)},
            ],
            "temperature": 0.0,
            "max_tokens": 200,
        },
        timeout=180,
    )
    r.raise_for_status()
    out = r.json()["choices"][0]["message"]["content"].strip()
    retries = 0
    if not detect_hant(out, x):
        r2 = requests.post(
            URL,
            json={
                "model": "tr600k",
                "messages": [
                    {"role": "system", "content": SYS_JA},
                    {"role": "user", "content": RETRY_INSTR.format(x=x)},
                ],
                "temperature": 0.0,
                "max_tokens": 200,
            },
            timeout=180,
        )
        r2.raise_for_status()
        out2 = r2.json()["choices"][0]["message"]["content"].strip()
        retries = 1
        if detect_hant(out2, x) or _han_ratio(out2) > _han_ratio(out):
            out = out2
    return out, retries


def atomic_checkpoint(done: int):
    tmp = OUT / "progress.json.tmp"
    tmp.write_text(json.dumps({"done": done}))
    os.replace(tmp, OUT / "progress.json")


def reconcile() -> int:
    path = OUT / "tw_translations.jsonl"
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
    phrases = []
    for f in CDN:
        phrases += json.load(open(f, encoding="utf-8"))
    print(f"phrases: {len(phrases)}", flush=True)
    done = reconcile()
    print(f"resume from {done}", flush=True)

    fh = open(OUT / "tw_translations.jsonl", "a", encoding="utf-8")
    t0 = time.perf_counter()
    n = retries = langfail = 0
    for start in range(done, len(phrases), BLOCK):
        block = phrases[start : start + BLOCK]
        with ThreadPoolExecutor(CONC) as ex:
            gens = list(ex.map(lambda p: gen_one(p["x"]), block))
        for p, (tw, r) in zip(block, gens):
            fh.write(
                json.dumps(
                    {
                        "i": p["i"],
                        "x": p["x"],
                        "tw": tw,
                        "retries": r,
                        "tw_ok": detect_hant(tw, p["x"]),
                    },
                    ensure_ascii=False,
                )
                + "\n"
            )
            n += 1
            retries += r
            if not detect_hant(tw, p["x"]):
                langfail += 1
        fh.flush()
        os.fsync(fh.fileno())
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
