"""
Detect problematic vocabulary entries in dictionary chunks.

Task 1: Chinese contamination in RU (38 entries)
  - Simplified Chinese characters (not in Japanese) in russian_translation
  - CJK bullet-point definitions in russian_translation
  - CJK inline with Russian morphology in russian_translation

Task 2: Garbage CJK in EN (16 entries)
  - Any CJK starting a bullet point in english_translation

Outputs a JSON report to bad_entries_report.json.
"""

import json
import os
import re

DICT_DIR = r"D:\origa_worktree\origa\cdn\dictionary"
OUTPUT_PATH = r"D:\origa_worktree\origa\scripts\bad_entries_report.json"
NUM_CHUNKS = 11


def is_cjk(ch):
    cp = ord(ch)
    return (
        0x4E00 <= cp <= 0x9FFF
        or 0x3400 <= cp <= 0x4DBF
        or 0xF900 <= cp <= 0xFAFF
        or 0x20000 <= cp <= 0x2A6DF
    )


def is_kana(ch):
    cp = ord(ch)
    return (
        0x3040 <= cp <= 0x309F
        or 0x30A0 <= cp <= 0x30FF
        or 0x31F0 <= cp <= 0x31FF
        or 0xFF65 <= cp <= 0xFF9F
    )


# Simplified Chinese characters that are NOT used in Japanese.
# These are mainland Chinese simplifications where Japanese kept a different form
# (traditional or its own shinjitai). Characters where the simplified form coincides
# with Japanese shinjitai (like 来, 当, 辞, 寿, etc.) are EXCLUDED.
SIMPLIFIED_ONLY = set("织预绕宽结书计线贤适间杂节说纲压产广剧亲华汉导")


def find_cjk_runs(text):
    """Find all consecutive CJK ideograph runs in text."""
    runs = []
    j = 0
    while j < len(text):
        if is_cjk(text[j]):
            start = j
            while j < len(text) and is_cjk(text[j]):
                j += 1
            run = text[start:j]
            runs.append((start, run))
        else:
            j += 1
    return runs


def detect_task1(data, chunk_file):
    """
    Detect Chinese contamination in russian_translation.

    Three detection methods:
    1. Simplified Chinese characters (unambiguously non-Japanese)
    2. CJK starting a bullet point (not the word's own kanji)
    3. CJK inline followed by Russian morphology (suffix)
    """
    results = []

    for word, entry in data.items():
        ru = entry.get("russian_translation", "")
        en = entry.get("english_translation", "")

        runs = find_cjk_runs(ru)
        if not runs:
            continue

        word_kanji = set(c for c in word if is_cjk(c))
        bad_runs = []

        for pos, run in runs:
            is_own_kanji = all(c in word_kanji for c in run)
            if is_own_kanji:
                continue

            has_simplified = any(c in SIMPLIFIED_ONLY for c in run)

            # Check bullet-point pattern
            line_start = ru.rfind("\n", 0, pos)
            line_start = 0 if line_start == -1 else line_start + 1
            prefix = ru[line_start:pos].strip()
            is_bullet_start = prefix in ("-", "- ", "")
            bullet_contamination = is_bullet_start and len(run) >= 2

            # Check inline Russian morphology
            after_run = ru[pos + len(run): pos + len(run) + 6]
            has_ru_suffix = bool(re.match(r"[а-яёА-ЯЁ]{2,}", after_run))
            inline_contamination = has_ru_suffix and len(run) >= 2

            if not (has_simplified or bullet_contamination or inline_contamination):
                continue

            # Filter: skip CJK inside parentheses unless it has simplified chars
            before = ru[:pos]
            open_p = before.count("(") - before.count(")")
            in_parens = open_p > 0
            if in_parens and not has_simplified:
                continue

            ctx_start = max(0, pos - 40)
            ctx_end = min(len(ru), pos + len(run) + 40)
            context = ru[ctx_start:ctx_end]

            det_type = (
                "simplified"
                if has_simplified
                else "bullet" if bullet_contamination else "inline"
            )
            bad_runs.append({
                "run": run,
                "pos": pos,
                "context": context,
                "detection_type": det_type,
            })

        if bad_runs:
            bad_chars = "".join(br["run"] for br in bad_runs)
            contexts = [br["context"] for br in bad_runs]
            results.append({
                "word": word,
                "chunk_file": chunk_file,
                "current_ru": ru,
                "current_en": en,
                "bad_chars": bad_chars,
                "context": contexts,
            })

    return results


def detect_task2(data, chunk_file):
    """
    Detect garbage CJK in english_translation.

    Pattern: any CJK word/compound starting a bullet point in EN.
    This includes both simplified Chinese words and Japanese words
    that appear as garbage bullet-point entries.
    """
    results = []

    for word, entry in data.items():
        en = entry.get("english_translation", "")
        ru = entry.get("russian_translation", "")

        lines = en.split("\n")
        bad_bullets = []

        for line in lines:
            stripped = line.strip()
            if not stripped:
                continue

            # Pattern: "-CJK" at start of line
            match = re.match(r"^-\s*[\u4e00-\u9fff]", stripped)
            if not match:
                continue

            after_dash = re.sub(r"^-\s*", "", stripped)
            # Collect CJK run including の and kana (for compounds like 座右の銘, 透かし絵)
            cjk_run = ""
            for ch in after_dash:
                if is_cjk(ch) or ch == "\u306e" or is_kana(ch):
                    cjk_run += ch
                else:
                    break

            if cjk_run:
                ctx_start = max(0, len(en) - len(en))
                bad_bullets.append({
                    "bad_text": cjk_run,
                    "line": stripped[:120],
                    "context": stripped[:120],
                })

        if bad_bullets:
            # Deduplicate bad_text values
            seen_texts = set()
            unique_bullets = []
            for b in bad_bullets:
                if b["bad_text"] not in seen_texts:
                    seen_texts.add(b["bad_text"])
                    unique_bullets.append(b)

            results.append({
                "word": word,
                "chunk_file": chunk_file,
                "current_en": en,
                "current_ru": ru,
                "bad_text": [b["bad_text"] for b in unique_bullets],
                "context": [b["context"] for b in unique_bullets],
            })

    return results


def main():
    task1_results = []
    task2_results = []

    for chunk_idx in range(1, NUM_CHUNKS + 1):
        chunk_file = f"chunk_{chunk_idx:02d}.json"
        chunk_path = os.path.join(DICT_DIR, chunk_file)

        with open(chunk_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        task1_results.extend(detect_task1(data, chunk_file))
        task2_results.extend(detect_task2(data, chunk_file))

    report = {
        "task1_chinese_in_ru": {
            "description": (
                "Entries where russian_translation contains Chinese contamination: "
                "simplified Chinese characters, CJK bullet-point definitions, or "
                "CJK inline with Russian morphology"
            ),
            "count": len(task1_results),
            "entries": task1_results,
        },
        "task2_garbage_in_en": {
            "description": (
                "Entries where english_translation contains CJK starting a bullet point. "
                "These are Japanese/Chinese words appearing as garbage definitions in English."
            ),
            "count": len(task2_results),
            "entries": task2_results,
        },
    }

    with open(OUTPUT_PATH, "w", encoding="utf-8") as f:
        json.dump(report, f, ensure_ascii=False, indent=2)

    print(f"Task 1 (Chinese contamination in RU): {len(task1_results)} entries")
    print(f"Task 2 (Garbage CJK in EN):           {len(task2_results)} entries")
    print(f"Report saved to: {OUTPUT_PATH}")

    # Verification: print summary
    print("\n--- Task 1 Summary ---")
    for idx, e in enumerate(task1_results):
        print(f"  {idx + 1}. {e['word']} | bad={e['bad_chars']} | chunk={e['chunk_file']}")

    print("\n--- Task 2 Summary ---")
    for idx, e in enumerate(task2_results):
        bt = ", ".join(e["bad_text"])
        print(f"  {idx + 1}. {e['word']} | bad={bt} | chunk={e['chunk_file']}")


if __name__ == "__main__":
    main()
