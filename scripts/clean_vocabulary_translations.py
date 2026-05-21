#!/usr/bin/env python3
"""
Clean up vocabulary translations by removing commentary blocks (> blocks)
from chunk_01.json through chunk_11.json.

About ~70% of Russian translations and ~10% of English translations contain
extra commentary blocks formatted as markdown blockquote-style notes after
the actual translations. This script removes them.

Usage:
    python scripts/clean_vocabulary_translations.py          # Process all chunks
    python scripts/clean_vocabulary_translations.py --dry-run # Preview only
    python scripts/clean_vocabulary_translations.py --chunk 3 # Process chunk_03 only
    python scripts/clean_vocabulary_translations.py --verbose # Show sample changes
"""

import json
import os
import re
import sys
import tempfile
import argparse
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
CHUNKS_DIR = PROJECT_ROOT / "cdn" / "dictionary"

JP_REGEX = re.compile(r"[\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FFF\u3400-\u4DBF]")

CYRILLIC_REGEX = re.compile(r"[\u0400-\u04FF]")
LATIN_WORD_REGEX = re.compile(r"[a-zA-Z]{3,}")


def clean_translation(text: str) -> tuple[str, bool, str]:
    """Remove > blocks from translation text.

    A > block starts with a line beginning with '>' (greater-than) and
    continues until the end of the string. Everything from that line
    onward is removed, along with any trailing blank lines before it.

    Returns:
        (cleaned_text, was_modified, removed_content_preview)
    """
    if ">" not in text:
        return text, False, ""

    lines = text.split("\n")

    # Find the first line that starts with '>'
    blockquote_index = None
    for i, line in enumerate(lines):
        if line.lstrip().startswith(">"):
            blockquote_index = i
            break

    if blockquote_index is None:
        return text, False, ""

    removed_lines = lines[blockquote_index:]
    removed_preview = "\n".join(removed_lines[:3])
    if len(removed_lines) > 3:
        removed_preview += f"\n  ... ({len(removed_lines) - 3} more lines)"

    kept_lines = lines[:blockquote_index]

    # Remove trailing blank lines before the > block
    while kept_lines and not kept_lines[-1].strip():
        kept_lines.pop()

    cleaned = "\n".join(kept_lines)
    return cleaned, True, removed_preview


def check_japanese_in_text(text: str) -> list[str]:
    """Find Japanese characters remaining in translation text.

    Returns list of unique Japanese characters found.
    """
    matches = JP_REGEX.findall(text)
    if not matches:
        return []
    seen = []
    for ch in matches:
        if ch not in seen:
            seen.append(ch)
    return seen


def check_cross_language(text: str, field_type: str) -> list[str]:
    """Check for cross-language contamination.

    For russian_translation: detect English phrases (3+ Latin words in sequence).
    For english_translation: detect Cyrillic characters.

    Returns list of warnings.
    """
    warnings = []

    if field_type == "russian":
        # Look for sequences of Latin words (English phrases, not single loanwords)
        latin_sequences = re.findall(r"(?:[a-zA-Z]+[\s-]+){2,}[a-zA-Z]+", text)
        for seq in latin_sequences:
            # Filter out common Russian transliterations / abbreviations
            words = seq.split()
            # Skip if all words are short abbreviations like "IT", "AI", "CD"
            if all(len(w) <= 3 for w in words):
                continue
            warnings.append(f"English phrase in RU: '{seq[:80]}'")

    elif field_type == "english":
        cyrillic_matches = CYRILLIC_REGEX.findall(text)
        if cyrillic_matches:
            # Find actual Cyrillic words (sequences), not single chars
            cyrillic_words = re.findall(r"[\u0400-\u04FF]{2,}", text)
            if cyrillic_words:
                warnings.append(
                    f"Russian text in EN: '{' '.join(cyrillic_words[:5])}'"
                )

    return warnings


def process_chunk(
    chunk_path: Path, dry_run: bool, verbose: bool
) -> dict:
    """Process a single chunk file. Returns statistics."""
    stats = {
        "total_entries": 0,
        "ru_cleaned": 0,
        "en_cleaned": 0,
        "ru_empty_after": 0,
        "en_empty_after": 0,
        "jp_in_ru": 0,
        "jp_in_en": 0,
        "cross_lang_ru_has_en": 0,
        "cross_lang_en_has_ru": 0,
        "not_modified": 0,
    }
    sample_changes: list[dict] = []
    warnings: list[str] = []
    jp_flagged: list[dict] = []
    cross_lang_flagged: list[dict] = []

    print(f"\n{'=' * 60}")
    print(f"Processing: {chunk_path.name}")
    print(f"{'=' * 60}")

    with open(chunk_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    stats["total_entries"] = len(data)
    modified_count = 0

    for word, entry in data.items():
        modified = False

        for field in ("russian_translation", "english_translation"):
            original = entry.get(field, "")
            if not original:
                continue

            cleaned, was_modified, removed_preview = clean_translation(original)

            if was_modified:
                modified = True
                field_short = "ru" if field == "russian_translation" else "en"
                stats[f"{field_short}_cleaned"] += 1

                if not cleaned.strip():
                    stats[f"{field_short}_empty_after"] += 1
                    warnings.append(
                        f"  WARNING: Empty {field} after cleaning for '{word}'\n"
                        f"    Removed: {removed_preview[:120]}"
                    )

                if verbose and len(sample_changes) < 10:
                    sample_changes.append({
                        "word": word,
                        "field": field,
                        "before": original[:200],
                        "after": cleaned[:200],
                        "removed": removed_preview[:200],
                    })

                entry[field] = cleaned

        # Secondary checks (on cleaned data)
        for field, field_type in [
            ("russian_translation", "ru"),
            ("english_translation", "en"),
        ]:
            text = entry.get(field, "")
            if not text:
                continue

            # Japanese character check
            jp_chars = check_japanese_in_text(text)
            if jp_chars:
                stats[f"jp_in_{field_type}"] += 1
                jp_flagged.append({
                    "word": word,
                    "field": field,
                    "chars": jp_chars[:10],
                    "text_preview": text[:100],
                })

            # Cross-language check
            lang_type = "russian" if field_type == "ru" else "english"
            cross_warnings = check_cross_language(text, lang_type)
            if cross_warnings:
                if field_type == "ru":
                    stats["cross_lang_ru_has_en"] += 1
                else:
                    stats["cross_lang_en_has_ru"] += 1
                cross_lang_flagged.append({
                    "word": word,
                    "field": field,
                    "warnings": cross_warnings,
                })

        if modified:
            modified_count += 1
        else:
            stats["not_modified"] += 1

    # Print chunk stats
    print(f"  Entries: {stats['total_entries']}")
    print(f"  Modified: {modified_count}")
    print(f"  RU cleaned: {stats['ru_cleaned']}")
    print(f"  EN cleaned: {stats['en_cleaned']}")
    print(f"  Not modified: {stats['not_modified']}")

    # Print warnings
    if warnings:
        print(f"\n  WARNINGS ({len(warnings)}):")
        for w in warnings:
            print(w)

    if jp_flagged:
        print(f"\n  JP chars remaining ({len(jp_flagged)} entries):")
        for item in jp_flagged[:10]:
            char_list = "".join(item["chars"])
            print(f"    '{item['word']}' ({item['field']}): {char_list}")
            print(f"      Preview: {item['text_preview'][:80]}")
        if len(jp_flagged) > 10:
            print(f"    ... and {len(jp_flagged) - 10} more")

    if cross_lang_flagged:
        print(f"\n  Cross-language flags ({len(cross_lang_flagged)} entries):")
        for item in cross_lang_flagged[:10]:
            print(f"    '{item['word']}' ({item['field']}):")
            for w in item["warnings"]:
                print(f"      {w}")
        if len(cross_lang_flagged) > 10:
            print(f"    ... and {len(cross_lang_flagged) - 10} more")

    # Print sample changes in verbose mode
    if verbose and sample_changes:
        print(f"\n  SAMPLE CHANGES:")
        for change in sample_changes:
            print(f"    Word: '{change['word']}' ({change['field']})")
            print(f"      BEFORE: {change['before'][:120]}...")
            print(f"      AFTER:  {change['after'][:120]}...")
            print()

    # Write if not dry run
    if not dry_run and modified_count > 0:
        write_atomic(data, chunk_path)
    elif dry_run and modified_count > 0:
        print(f"  [DRY RUN] Would modify {modified_count} entries in {chunk_path.name}")
    else:
        print(f"  No changes needed for {chunk_path.name}")

    stats["modified_count"] = modified_count
    return stats


def write_atomic(data: dict, target_path: Path) -> None:
    """Write JSON data atomically using temp file + os.replace.

    The file is first written to a temporary location, validated,
    then atomically moved to the target path.
    """
    json_str = json.dumps(data, ensure_ascii=False, indent=2)

    # Validate generated JSON
    try:
        json.loads(json_str)
    except json.JSONDecodeError as e:
        print(f"  ERROR: generated JSON is invalid: {e}", file=sys.stderr)
        sys.exit(1)

    # Write to temp file in the same directory (ensures same filesystem)
    tmp_path = target_path.with_suffix(".json.tmp")
    try:
        with open(tmp_path, "w", encoding="utf-8") as f:
            f.write(json_str)
            f.write("\n")

        # Validate file on disk by re-reading
        with open(tmp_path, "r", encoding="utf-8") as f:
            json.load(f)

        # Atomic replace
        os.replace(tmp_path, target_path)
        size = os.path.getsize(target_path)
        print(f"  Written {target_path.name} ({size:,} bytes, valid JSON)")
    except Exception:
        # Clean up temp file on error
        if tmp_path.exists():
            tmp_path.unlink()
        raise


def get_chunk_paths(args) -> list[Path]:
    """Resolve which chunk files to process based on arguments."""
    if args.chunk is not None:
        chunk_num = args.chunk
        if not 1 <= chunk_num <= 11:
            print(f"ERROR: chunk number must be 1-11, got {chunk_num}", file=sys.stderr)
            sys.exit(1)
        path = CHUNKS_DIR / f"chunk_{chunk_num:02d}.json"
        if not path.exists():
            print(f"ERROR: {path} not found", file=sys.stderr)
            sys.exit(1)
        return [path]

    paths = sorted(CHUNKS_DIR.glob("chunk_*.json"))
    if not paths:
        print(f"ERROR: no chunk_*.json files in {CHUNKS_DIR}", file=sys.stderr)
        sys.exit(1)
    return paths


def print_summary(all_stats: list[dict], dry_run: bool) -> None:
    """Print overall summary across all processed chunks."""
    print(f"\n{'=' * 60}")
    mode = "DRY RUN — " if dry_run else ""
    print(f"{mode}SUMMARY")
    print(f"{'=' * 60}")

    total_entries = sum(s["total_entries"] for s in all_stats)
    total_modified = sum(s["modified_count"] for s in all_stats)
    total_ru = sum(s["ru_cleaned"] for s in all_stats)
    total_en = sum(s["en_cleaned"] for s in all_stats)
    total_ru_empty = sum(s["ru_empty_after"] for s in all_stats)
    total_en_empty = sum(s["en_empty_after"] for s in all_stats)
    total_jp_ru = sum(s["jp_in_ru"] for s in all_stats)
    total_jp_en = sum(s["jp_in_en"] for s in all_stats)
    total_cross_ru_en = sum(s["cross_lang_ru_has_en"] for s in all_stats)
    total_cross_en_ru = sum(s["cross_lang_en_has_ru"] for s in all_stats)

    print(f"  Total entries:           {total_entries}")
    print(f"  Total modified entries:  {total_modified}")
    print(f"  RU translations cleaned: {total_ru} ({total_ru / total_entries * 100:.1f}%)")
    print(f"  EN translations cleaned: {total_en} ({total_en / total_entries * 100:.1f}%)")
    print(f"  RU empty after cleaning: {total_ru_empty}")
    print(f"  EN empty after cleaning: {total_en_empty}")
    print()
    print(f"  JP chars in RU (flag):   {total_jp_ru}")
    print(f"  JP chars in EN (flag):   {total_jp_en}")
    print(f"  English in RU (flag):    {total_cross_ru_en}")
    print(f"  Russian in EN (flag):    {total_cross_en_ru}")

    # Per-chunk breakdown
    print(f"\n  Per-chunk breakdown:")
    print(f"  {'Chunk':<14s} {'Entries':>8s} {'Modified':>10s} {'RU clean':>10s} {'EN clean':>10s}")
    print(f"  {'-' * 54}")
    for s in all_stats:
        chunk_name = s.get("chunk_name", "?")
        print(
            f"  {chunk_name:<14s} {s['total_entries']:>8d} "
            f"{s['modified_count']:>10d} {s['ru_cleaned']:>10d} {s['en_cleaned']:>10d}"
        )


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Clean vocabulary translations by removing > commentary blocks"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview changes without modifying files",
    )
    parser.add_argument(
        "--chunk",
        type=int,
        help="Process only specific chunk number (1-11)",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Show sample before/after changes",
    )
    args = parser.parse_args()

    if not CHUNKS_DIR.exists():
        print(f"ERROR: chunks directory not found: {CHUNKS_DIR}", file=sys.stderr)
        sys.exit(1)

    chunk_paths = get_chunk_paths(args)

    print("clean_vocabulary_translations.py")
    print(f"Chunks directory: {CHUNKS_DIR}")
    print(f"Files to process: {len(chunk_paths)}")
    if args.dry_run:
        print("Mode: DRY RUN (no files will be modified)")

    all_stats: list[dict] = []

    for chunk_path in chunk_paths:
        stats = process_chunk(chunk_path, args.dry_run, args.verbose)
        stats["chunk_name"] = chunk_path.name
        all_stats.append(stats)

    print_summary(all_stats, args.dry_run)


if __name__ == "__main__":
    main()
