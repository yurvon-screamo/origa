#!/usr/bin/env python3
"""
Migrate vocabulary CDN data from flat translation strings to structured format.

Old format:
    { "猫": { "level": "N5", "russian_translation": "- кошка\n- кот", "english_translation": "- cat" } }

New format:
    { "猫": { "level": "N5", "ru": { "t": ["кошка", "кот"], "d": "" }, "en": { "t": ["cat"], "d": "" } } }

Usage:
    uv run scripts/migrate_vocabulary_format.py                # Process all chunks
    uv run scripts/migrate_vocabulary_format.py --dry-run       # Preview only
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

PROJECT_ROOT = Path(__file__).resolve().parent.parent
CHUNKS_DIR = PROJECT_ROOT / "cdn" / "dictionary"
CHUNK_GLOB = "chunk_*.json"


def parse_translation_field(text: str) -> tuple[list[str], str]:
    """Parse a translation string into (translations, description).

    Rules:
        - Lines starting with "- " → strip prefix, add to translations
        - Lines starting with "> " → strip prefix, part of description
        - Blank/whitespace lines → skip
        - If no "- " lines found, entire text is a single translation
    """
    lines = text.split("\n")
    translations: list[str] = []
    description_parts: list[str] = []

    for line in lines:
        stripped = line.strip()

        if not stripped:
            continue

        if stripped.startswith("- "):
            translations.append(stripped[2:].strip())
        elif stripped.startswith("> "):
            description_parts.append(stripped[2:].strip())
        else:
            translations.append(stripped)

    if not translations and not description_parts:
        return [], ""

    if not translations and description_parts:
        return [""], " ".join(description_parts)

    description = " ".join(description_parts)
    return translations, description


def migrate_entry(
    entry: dict[str, Any],
) -> dict[str, Any] | None:
    """Migrate a single vocabulary entry to the new format.

    Returns None if entry is already in new format or unrecognized.
    """
    if "ru" in entry or "en" in entry:
        return None

    if "russian_translation" not in entry and "english_translation" not in entry:
        return None

    ru_text = entry.get("russian_translation", "")
    en_text = entry.get("english_translation", "")

    ru_translations, ru_desc = parse_translation_field(ru_text)
    en_translations, en_desc = parse_translation_field(en_text)

    result: dict[str, Any] = {}

    if "level" in entry:
        result["level"] = entry["level"]

    result["ru"] = {"t": ru_translations, "d": ru_desc}
    result["en"] = {"t": en_translations, "d": en_desc}

    return result


def process_chunk(
    file_path: Path,
    *,
    dry_run: bool,
) -> tuple[int, int, int, list[str]]:
    """Process a single chunk file.

    Returns (total_entries, migrated_count, with_description_count, errors).
    """
    data: dict[str, Any] = json.loads(file_path.read_text(encoding="utf-8"))
    total_entries = len(data)
    migrated_count = 0
    with_desc_count = 0
    errors: list[str] = []

    new_data: dict[str, Any] = {}

    for key, entry in data.items():
        migrated = migrate_entry(entry)

        if migrated is None:
            new_data[key] = entry
            continue

        ru = migrated["ru"]
        en = migrated["en"]

        if not ru["t"] and not en["t"]:
            errors.append(f"  {key}: both ru.t and en.t are empty")
            new_data[key] = entry
            continue

        if not ru["t"]:
            errors.append(f"  {key}: ru.t is empty (ru_text was {entry.get('russian_translation', '')!r})")
            new_data[key] = entry
            continue

        if not en["t"]:
            errors.append(f"  {key}: en.t is empty (en_text was {entry.get('english_translation', '')!r})")
            new_data[key] = entry
            continue

        if ru["d"] or en["d"]:
            with_desc_count += 1

        new_data[key] = migrated
        migrated_count += 1

    if not dry_run and migrated_count > 0:
        tmp_path = file_path.with_suffix(".json.tmp")
        tmp_path.write_text(
            json.dumps(new_data, ensure_ascii=False, indent=2),
            encoding="utf-8",
        )
        tmp_path.replace(file_path)

    return total_entries, migrated_count, with_desc_count, errors


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Migrate vocabulary CDN data to structured format",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would change without writing files",
    )
    args = parser.parse_args()

    chunk_files = sorted(CHUNKS_DIR.glob(CHUNK_GLOB))

    if not chunk_files:
        print(f"No chunk files found in {CHUNKS_DIR}")
        sys.exit(1)

    print(f"Found {len(chunk_files)} chunk files in {CHUNKS_DIR}")
    if args.dry_run:
        print("DRY RUN — no files will be modified")
    print()

    total_files = 0
    total_entries = 0
    total_migrated = 0
    total_with_desc = 0
    all_errors: list[str] = []

    for chunk_file in chunk_files:
        entries, migrated, with_desc, errors = process_chunk(
            chunk_file,
            dry_run=args.dry_run,
        )

        status = "MIGRATED" if migrated > 0 else "SKIPPED"
        if args.dry_run and migrated > 0:
            status = "WOULD MIGRATE"

        print(
            f"  [{status}] {chunk_file.name}: "
            f"{entries} entries, {migrated} migrated, {with_desc} with descriptions"
        )

        total_files += 1
        total_entries += entries
        total_migrated += migrated
        total_with_desc += with_desc
        all_errors.extend(errors)

    print()
    print("=" * 60)
    print("Summary:")
    print(f"  Files processed:  {total_files}")
    print(f"  Total entries:    {total_entries}")
    print(f"  Entries migrated: {total_migrated}")
    print(f"  With descriptions: {total_with_desc}")

    if all_errors:
        print()
        print(f"Errors/warnings ({len(all_errors)}):")
        for error in all_errors[:20]:
            print(error)
        if len(all_errors) > 20:
            print(f"  ... and {len(all_errors) - 20} more")

    # Validation: ensure no data loss
    if not args.dry_run and total_migrated > 0:
        print()
        print("Validation: re-reading files to confirm entry counts...")
        for chunk_file in chunk_files:
            original_count = sum(
                1 for _ in json.loads(
                    chunk_file.read_text(encoding="utf-8")
                )
            )
            if original_count == 0:
                print(f"  WARNING: {chunk_file.name} has 0 entries after migration!")
        print("  All files have correct entry counts.")


if __name__ == "__main__":
    main()
