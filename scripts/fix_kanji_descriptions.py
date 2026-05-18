#!/usr/bin/env python3
"""
Migrate description fields in kanji.json from strings to arrays
and restore 74 lost secondary meanings from pre-i18n git history.

Commit 0701bc73 contains the last version with "description" field
before i18n split to "description_ru" / "description_en".
"""

import json
import os
import subprocess
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
KANJI_PATH = PROJECT_ROOT / "cdn" / "dictionary" / "kanji.json"
TMP_KANJI_PATH = KANJI_PATH.with_suffix(".json.tmp")
GIT_COMMIT = "0701bc73"
GIT_FILE_PATH = "origa/src/domain/dictionary/kanji.json"

# Kanji where the old semicolon meanings need forced re-restoration
# even if already processed (token-level mismatch detected in verification).
FORCE_RESTORE_KANJI = {"丸", "兄", "朕"}

# Special-case kanji where current description_ru is completely different
# from the old semicolon-separated values (not a subset).
SKIP_RESTORE_KANJI = {"実", "乗"}

# Manual override: hardcoded meaning for 可 in English that was missing
MANUAL_EN_OVERRIDE = {
    "可": {
        "description_ru": ["хороший", "возможный"],
        "description_en": ["good", "possible"],
    },
}


def extract_old_data() -> list[dict]:
    """Run git show to extract pre-i18n kanji data."""
    cmd = [
        "git", "-C", str(PROJECT_ROOT),
        "show", f"{GIT_COMMIT}:{GIT_FILE_PATH}",
    ]
    print(f"Extracting old data: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8")
    if result.returncode != 0:
        print(f"ERROR: git command failed (exit code {result.returncode})", file=sys.stderr)
        print(f"  stderr: {result.stderr.strip()}", file=sys.stderr)
        sys.exit(1)

    try:
        data = json.loads(result.stdout)
    except json.JSONDecodeError as e:
        print(f"ERROR: failed to parse old kanji.json: {e}", file=sys.stderr)
        sys.exit(1)

    if not isinstance(data, dict) or "kanji" not in data:
        print("ERROR: old kanji.json missing top-level 'kanji' key", file=sys.stderr)
        sys.exit(1)

    kanji_list = data["kanji"]
    print(f"  Old data extracted: {len(kanji_list)} kanji entries")
    return kanji_list


def build_old_lookup(old_data: list[dict]) -> dict[str, str]:
    """Build char -> old_description_string dict."""
    lookup: dict[str, str] = {}
    for entry in old_data:
        kanji = entry.get("kanji", "")
        desc = entry.get("description", "")
        if kanji:
            lookup[kanji] = desc
    return lookup


def load_current_data() -> dict:
    """Read current kanji.json."""
    print(f"Reading current: {KANJI_PATH}")
    with open(KANJI_PATH, "r", encoding="utf-8") as f:
        return json.load(f)


def process_entries(
    kanji_list: list[dict],
    old_lookup: dict[str, str],
) -> dict:
    """Process all kanji entries, converting descriptions to arrays."""
    stats = {
        "total": len(kanji_list),
        "restored": 0,
        "converted_only": 0,
        "skipped_already_list": 0,
        "skipped_special": 0,
        "manual_override": 0,
        "en_manual_fixes": 0,
        "diff_entries": {},
    }

    key_kanji_for_diff = {
        "可", "月", "日", "女", "高", "楽", "運", "円",
        "果", "余", "率", "仏", "華", "俳", "署", "角",
        "象", "刻", "喫", "乗", "令",
    }

    for entry in kanji_list:
        kanji = entry.get("kanji", "?")
        desc_ru = entry.get("description_ru", "")
        desc_en = entry.get("description_en", "")

        # Manual override for 可 — must happen before idempotency check
        # because on re-run, both fields are already lists but desc_en is incomplete.
        if kanji in MANUAL_EN_OVERRIDE:
            override = MANUAL_EN_OVERRIDE[kanji]
            if entry.get("description_ru") != override["description_ru"] or entry.get("description_en") != override["description_en"]:
                entry["description_ru"] = override["description_ru"]
                entry["description_en"] = override["description_en"]
                stats["manual_override"] += 1
            else:
                stats["skipped_already_list"] += 1
            if kanji in key_kanji_for_diff:
                stats["diff_entries"][kanji] = {
                    "old": old_lookup.get(kanji, ""),
                    "new_ru": entry["description_ru"],
                    "new_en": entry["description_en"],
                }
            continue

        # Force-restore: these kanji have old semicolon meanings that were
        # incorrectly collapsed with token-level mismatch (e.g. "круг" vs "круглый").
        if kanji in FORCE_RESTORE_KANJI:
            old_desc = old_lookup.get(kanji, "")
            if old_desc and ";" in old_desc:
                old_values = [v.strip() for v in old_desc.split(";") if v.strip()]
                # Compare lexically, ignoring order
                ru_val = entry.get("description_ru")
                if isinstance(ru_val, list):
                    cur_ru_list = ru_val
                elif isinstance(ru_val, str):
                    cur_ru_list = [ru_val]
                else:
                    cur_ru_list = []
                if sorted(cur_ru_list) != sorted(old_values):
                    print(f"  FORCE RESTORE: '{kanji}' — cur={cur_ru_list} → old={old_values}")
                    entry["description_ru"] = old_values
                    # Keep desc_en as is
                    stats["restored"] += 1
                else:
                    stats["skipped_already_list"] += 1
            else:
                stats["skipped_already_list"] += 1
            if kanji in key_kanji_for_diff:
                stats["diff_entries"][kanji] = {
                    "old": old_lookup.get(kanji, ""),
                    "new_ru": entry.get("description_ru", []),
                    "new_en": entry.get("description_en", []),
                }
            continue

        # Idempotency: skip only if BOTH are already lists
        if isinstance(desc_ru, list) and isinstance(desc_en, list):
            stats["skipped_already_list"] += 1
            continue

        # Partial fix-up: desc_ru is already list but desc_en is still string
        if isinstance(desc_ru, list):
            if isinstance(desc_en, str):
                entry["description_en"] = [desc_en.strip()] if desc_en.strip() else []
                stats["converted_only"] += 1
            if kanji in key_kanji_for_diff:
                stats["diff_entries"][kanji] = {
                    "old": old_lookup.get(kanji, ""),
                    "new_ru": entry.get("description_ru", []),
                    "new_en": entry.get("description_en", []),
                }
            continue

        old_desc = old_lookup.get(kanji, "")

        # ── Process description_ru ──
        if old_desc and ";" in old_desc:
            old_values = [v.strip() for v in old_desc.split(";") if v.strip()]
            current_clean = desc_ru.strip()

            if kanji in SKIP_RESTORE_KANJI:
                print(f"  SKIP RESTORE: '{kanji}' — current desc_ru ({desc_ru!r}) "
                      f"differs from all old values ({old_values})")
                entry["description_ru"] = [current_clean] if current_clean else []
                stats["skipped_special"] += 1
            else:
                # Tokenize current value: split by , ; and spaces, trim, lowercase
                current_tokens = {
                    t.strip().lower()
                    for t in current_clean.replace(",", ";").split(";")
                    if t.strip()
                }
                has_all = True
                for old_val in old_values:
                    if old_val.lower() not in current_tokens:
                        has_all = False
                        break

                if has_all:
                    entry["description_ru"] = [current_clean] if current_clean else old_values
                    stats["converted_only"] += 1
                else:
                    entry["description_ru"] = old_values
                    stats["restored"] += 1
        else:
            entry["description_ru"] = [desc_ru.strip()] if desc_ru.strip() else []

        # ── Process description_en: always string → array ──
        if isinstance(desc_en, str):
            entry["description_en"] = [desc_en.strip()] if desc_en.strip() else []

        # Collect diff for key kanji
        if kanji in key_kanji_for_diff:
            stats["diff_entries"][kanji] = {
                "old": old_desc,
                "new_ru": entry.get("description_ru", []),
                "new_en": entry.get("description_en", []),
            }

    # Manual English description fixes for key kanji (pre-i18n data only had Russian,
    # so English secondary values must be added manually)
    MANUAL_EN_FIXES = {
        "月": ["month", "moon"],
        "日": ["sun", "day"],
        "高": ["expensive", "tall / high"],
        "楽": ["music", "fun / enjoyable"],
        "運": ["carry", "luck / fortune"],
        "円": ["circle", "yen"],
        "果": ["result", "fruit", "to bear fruit"],
        "余": ["surplus", "remaining", "extra"],
        "率": ["lead", "ratio / rate"],
        "仏": ["Buddha", "France"],
        "華": ["China", "Chinese", "splendor"],
        "俳": ["actor", "haiku"],
        "署": ["station", "signature"],
        "角": ["corner", "horn"],
        "象": ["image", "elephant"],
        "刻": ["carve", "time"],
        "喫": ["eat", "drink", "smoke"],
        "乗": ["ride", "load", "power"],
        "令": ["order", "command", "good"],
    }

    for entry in kanji_list:
        kanji = entry["kanji"]
        if kanji in MANUAL_EN_FIXES:
            entry["description_en"] = MANUAL_EN_FIXES[kanji]
            stats["en_manual_fixes"] += 1

    return stats


def write_atomic(kanji_list: list[dict]) -> None:
    """Write data to .tmp file, validate, then rename."""
    data = {"kanji": kanji_list}

    # Serialize and re-parse to validate
    json_str = json.dumps(data, ensure_ascii=False, indent=2)
    try:
        json.loads(json_str)
    except json.JSONDecodeError as e:
        print(f"ERROR: generated JSON is invalid: {e}", file=sys.stderr)
        sys.exit(1)

    # Write to .tmp
    with open(TMP_KANJI_PATH, "w", encoding="utf-8") as f:
        f.write(json_str)
        f.write("\n")

    # Validate file on disk
    with open(TMP_KANJI_PATH, "r", encoding="utf-8") as f:
        json.load(f)

    # Atomic rename (os.replace is atomic on Windows)
    os.replace(TMP_KANJI_PATH, KANJI_PATH)
    print(f"Written {KANJI_PATH} (valid JSON, {os.path.getsize(KANJI_PATH):,} bytes)")


def print_statistics(stats: dict) -> None:
    """Print processing statistics."""
    print()
    print("=" * 60)
    print("STATISTICS")
    print("=" * 60)
    print(f"  Total entries processed:    {stats['total']}")
    print(f"  Values restored:            {stats['restored']}")
    print(f"  Converted only (str→list):  {stats.get('converted_only', 0)}")
    print(f"  Skipped (already list):     {stats['skipped_already_list']}")
    print(f"  Skipped (special 実/乗):     {stats['skipped_special']}")
    print(f"  Manual override:            {stats['manual_override']}")
    print(f"  EN manual fixes:            {stats['en_manual_fixes']}")


def print_diff(stats: dict) -> None:
    """Print diff for key kanji for manual review."""
    print()
    print("=" * 60)
    print("MANUAL REVIEW — KEY KANJI")
    print("=" * 60)

    ordered_keys = [
        "可", "月", "日", "女", "高", "楽", "運", "円",
        "果", "余", "率", "仏", "華", "俳", "署", "角",
        "象", "刻", "喫", "乗", "令",
    ]
    for kanji in ordered_keys:
        info = stats["diff_entries"].get(kanji)
        if info:
            print(f"  {kanji:4s}  ru: {str(info['new_ru']):<40s}  en: {info['new_en']}")


def main() -> None:
    print("fix_kanji_descriptions.py — migrating descriptions to arrays")
    print()

    # Step 1: Extract old data
    old_data = extract_old_data()

    # Step 2: Load current data
    current_data = load_current_data()
    kanji_list = current_data["kanji"]
    print(f"  Current entries: {len(kanji_list)}")

    # Step 3: Build lookup
    old_lookup = build_old_lookup(old_data)

    # Step 4: Process all entries
    stats = process_entries(kanji_list, old_lookup)

    # Step 5: Write atomically
    write_atomic(kanji_list)

    # Step 6: Print results
    print_statistics(stats)
    print_diff(stats)


if __name__ == "__main__":
    main()
