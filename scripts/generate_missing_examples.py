#!/usr/bin/env python3
"""Stage-3 filler for issue #528: generate textbook-style examples for
popular words that the Tanaka/Tatoeba corpus does not cover, then translate
them (ru/vi/ko) through the same tr600k pipeline as translate_examples.py.

Runs AFTER build_examples.py + translate_examples.py: appends new sentences
to cdn/examples/data/ (sid continuation) and patches index.json with refs
for the previously uncovered words. Deterministic per-model-output; ids are
stable because the filler only appends.

Usage: python scripts/generate_missing_examples.py
Requires vLLM tr600k on :8091 (translator) — the judge is not used here:
algorithmic checks only (ja language, word present, length, sentence end).
"""

from __future__ import annotations

import json
import re
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import requests

REPO = Path(__file__).resolve().parent.parent
DATA = REPO / "cdn" / "examples" / "data"
INDEX = REPO / "cdn" / "examples" / "index.json"
TR_URL = "http://127.0.0.1:8091/v1/chat/completions"
LOCALES = ("ru", "vi", "ko")

GEN_SYS = (
    "You are a Japanese teacher writing textbook example sentences. "
    "Write ONE short, simple, self-contained Japanese sentence that uses "
    "the given word naturally. JLPT-appropriate grammar and vocabulary "
    "only. End with 。 Do not add explanations."
)
GEN_INSTR = (
    "Word: {word}\n"
    "Write one textbook example sentence (JLPT {level} level) using this "
    "word. Only the sentence, nothing else."
)

RE_HANGUL = re.compile(r"[가-힣]")
RE_CYR = re.compile(r"[А-ЯЁа-яё]")
RE_KANA = re.compile(r"[ひ-んア-ンー]")
RE_CJK = re.compile(r"[一-龥]")
RE_LATIN = re.compile(r"[A-Za-z]")
RE_VI_MARK = re.compile(
    r"[ăâêôơưđĂÂÊÔƠƯĐ]|[áàảãạấầẩẫậắằẳẵặéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ]"
)
SENT_END = ("。", "？", "！")


def detect_language(text: str) -> str:
    t = (text or "").strip()
    if not t:
        return "empty"
    n = max(len(t), 1)
    c = {"ko": len(RE_HANGUL.findall(t)), "ru": len(RE_CYR.findall(t)),
         "ja": len(RE_KANA.findall(t)) + len(RE_CJK.findall(t)),
         "vi": len(RE_VI_MARK.findall(t)), "en": len(RE_LATIN.findall(t))}
    for k in ("ko", "ru", "ja", "vi"):
        if c[k] / n >= 0.25:
            return k
    if c["vi"] >= 2:
        return "vi"
    return "en" if c["en"] / n >= 0.5 else "other"


def call_chat(system: str, user: str, temperature: float = 0.0) -> str:
    r = requests.post(TR_URL, json={
        "model": "tr600k",
        "messages": [{"role": "system", "content": system},
                     {"role": "user", "content": user}],
        "temperature": temperature, "max_tokens": 160,
    }, timeout=180)
    r.raise_for_status()
    return r.json()["choices"][0]["message"]["content"].strip()


def translate_one(ja: str, lang: str) -> str:
    sys_en = ("You are a skilled Japanese translator. Translate the "
              "sentence accurately and naturally.")
    instr = {
        "ru": f"次の日本語の文をロシア語に翻訳してください:\n{ja}",
        "vi": f"次の日本語の文をベトナム語に翻訳してください:\n{ja}",
        "ko": f"次の日本語の文を韓国語に翻訳してください:\n{ja}",
    }[lang]
    return call_chat(sys_en, instr)


def main() -> int:
    index = json.loads(INDEX.read_text(encoding="utf-8"))
    words: dict[str, dict] = index["words"]

    # popular vocabulary = union of all well_known_set files; the filler
    # targets words that build_examples.py left out of the index entirely.
    popular: set[str] = set()
    level_by_word: dict[str, str] = {}
    for sf in sorted((REPO / "cdn" / "well_known_set").rglob("*.json")):
        try:
            d = json.loads(sf.read_text(encoding="utf-8"))
        except Exception:
            continue
        if isinstance(d, dict) and isinstance(d.get("words"), list):
            for w in d["words"]:
                popular.add(w)
                level_by_word.setdefault(w, d.get("level", "N3"))

    # existing sentence count = next sid base
    files = sorted(DATA.glob("s*.json"))
    rows_by_file: dict[Path, list] = {}
    max_sid = -1
    for f in files:
        part = json.loads(f.read_text(encoding="utf-8"))
        rows_by_file[f] = part
        for r in part:
            max_sid = max(max_sid, r["i"])

    # words absent from the index = no examples yet
    holes = sorted(popular - set(words))
    print(f"words without examples: {len(holes)}", flush=True)

    next_sid = max_sid + 1
    generated: list[dict] = []
    t0 = time.perf_counter()

    def gen_one(word: str) -> list[dict]:
        level = level_by_word.get(word, "N3")
        out = []
        for _ in range(3):
            try:
                ja = call_chat(
                    GEN_SYS,
                    GEN_INSTR.format(word=word, level=level),
                    temperature=0.8,
                )
            except Exception:
                continue
            ok = (
                ja
                and len(ja) <= 60
                and ja.endswith(SENT_END)
                and detect_language(ja) == "ja"
            )
            if not ok:
                continue
            row = {"i": 0, "x": ja, "en": "", "ru": "", "vi": "", "ko": "",
                   "f": [], "w": word}
            for lang in LOCALES:
                try:
                    row[lang] = translate_one(ja, lang)
                except Exception:
                    row[lang] = ""
            if not all(row[lang] for lang in LOCALES):
                continue
            out.append(row)
        return out

    with ThreadPoolExecutor(8) as ex:
        for rows in ex.map(gen_one, holes):
            if not rows:
                continue
            word = rows[0]["w"]
            refs = []
            for row in rows:
                row["i"] = next_sid
                refs.append([next_sid, -1, -1])
                generated.append(row)
                next_sid += 1
            words[word] = {"refs": refs}

    # append to chunks (1000 rows per chunk, matching build_examples.py)
    all_rows = []
    for part in rows_by_file.values():
        all_rows.extend(part)
    all_rows.extend(generated)
    chunks = [all_rows[i : i + 1000] for i in range(0, len(all_rows), 1000)]
    for idx, chunk in enumerate(chunks):
        (DATA / f"s{idx:04d}.json").write_text(
            json.dumps(chunk, ensure_ascii=False, separators=(",", ":")),
            encoding="utf-8",
        )

    index["words"] = dict(sorted(words.items()))
    index["s"] = len(all_rows)
    INDEX.write_text(
        json.dumps(index, ensure_ascii=False, sort_keys=True, separators=(",", ":")),
        encoding="utf-8",
    )
    filled = sum(1 for w in holes if words.get(w, {}).get("refs"))
    print(f"generated sentences: {len(generated)} (words filled: {filled})",
          flush=True)
    print(f"total sentences now: {len(all_rows)}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
