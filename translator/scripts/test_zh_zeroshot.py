#!/usr/bin/env python3
"""Zero-shot JA->zh test on the 8 reference phrases (same set as my/th/id)."""
import json
import re
import time
from pathlib import Path

import requests

URL = "http://127.0.0.1:8091/v1/chat/completions"
SYS = "You are a skilled Japanese translator. Translate the sentence accurately and naturally."
DATA = Path("/home/yurvon/origa-translator/data")

# те же 8 JP-фраз из test_20x4 (RU-подмножество), что и в my/th/id-тесте
rows = [json.loads(l) for l in open(DATA / "test_20x4.jsonl", encoding="utf-8")]
ru8 = [r for r in rows if r["lang"] == "ru"][:8]

RE_KANA = re.compile(r"[ひ-んァ-ヺーぁ-ゖ]")
ZH_VARIANTS = {"Simplified": "简", "Traditional": "繁"}


def ask(jp, variant):
    lname = f"Chinese ({variant})"
    r = requests.post(URL, json={
        "model": "tr600k",
        "messages": [{"role": "system", "content": SYS},
                     {"role": "user", "content": f"Translate the following Japanese sentence into {lname}:\n{jp}"}],
        "temperature": 0.0, "max_tokens": 200}, timeout=180)
    r.raise_for_status()
    return r.json()["choices"][0]["message"]["content"].strip()


t0 = time.perf_counter()
print("=== JA→Chinese (Simplified), zero-shot, адаптер ep1 ===\n")
for r in ru8:
    jp = r["x"]
    zh = ask(jp, "Simplified")
    kana = bool(RE_KANA.search(zh))
    marker = " ⚠️КАНА!" if kana else ""
    print(f"JP: {jp[:60]}")
    print(f"ZH: {zh[:80]}{marker}")
    print(f"RU(эталон смысла): {r['gold'][:60]}")
    print()
print(f"[{time.perf_counter()-t0:.0f}s]")

# одна фраза традиционным — проверка переключения варианта
print("=== Контроль варианта: Traditional ===")
zh_t = ask(ru8[1]["x"], "Traditional")
print(f"JP: {ru8[1]['x'][:60]}")
print(f"ZH-Trad: {zh_t[:80]}")
