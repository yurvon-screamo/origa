"""Apply curated one-shot phrase translation corrections (#178 P-7).

The phrase dataset occasionally contains translations where the segmenter
misinterpreted the source and produced a Russian gloss that has nothing to do
with the actual Japanese. These are corrected by hand here, keyed by phrase id,
so the fix is auditable and survives future dataset regenerations.

This script touches ONLY cdn/phrases/data/p*.json. The phrase_index.json hash
is unaffected: the index entries contain id/tokens/chunk/grammar only, never
the translation text itself.

Run:
    python scripts/fix_phrase_translations.py --phrases cdn/phrases
    python scripts/fix_phrase_translations.py --phrases cdn/phrases --dry-run
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

# Hand-verified translation corrections, keyed by phrase id (Ulid-style string
# from the "i" field). Each value partially replaces keys inside the phrase
# record — only the listed fields are mutated, others (x/en/tokens) are left
# alone so the change is minimal and reviewable.
CORRECTIONS: dict[str, dict[str, str]] = {
    # Japanese: いやはや。危ない危ない。ふぅ～
    # Previous RU: "Ох, чуть-чуть не получилось! Фух!" — wrong (segmenter picked
    # the "almost failed" sense of 危ない, but the repetition 危ない危ない is the
    # idiomatic "that was close / dangerous!" relief exclamation).
    # English reference: "Oh my, that was close! Whew!"
    "0000000000NTQET51VCTVJAB4Q": {
        "ru": "Ох, опасно, опасно! Фух~",
    },
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--phrases",
        required=True,
        help="Path to phrases directory (containing data/p*.json)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show planned updates without writing files",
    )
    return parser.parse_args()


def find_phrase_record(
    phrases: list[dict[str, Any]],
    phrase_id: str,
) -> tuple[int, dict[str, Any]] | None:
    for idx, record in enumerate(phrases):
        if record.get("i") == phrase_id:
            return idx, record
    return None


def apply_to_chunk_file(
    path: Path,
    dry_run: bool,
) -> list[tuple[str, str, str]]:
    """Apply corrections to one chunk file. Returns list of (id, field, change)."""
    with open(path, encoding="utf-8") as f:
        phrases = json.load(f)

    changes: list[tuple[str, str, str]] = []
    for phrase_id, field_overrides in CORRECTIONS.items():
        found = find_phrase_record(phrases, phrase_id)
        if found is None:
            continue
        _, record = found
        for field, new_value in field_overrides.items():
            old_value = record.get(field)
            if old_value == new_value:
                continue
            record[field] = new_value
            changes.append((phrase_id, field, f"{old_value!r} -> {new_value!r}"))

    if changes and not dry_run:
        with open(path, "w", encoding="utf-8") as f:
            json.dump(phrases, f, ensure_ascii=False, separators=(",", ":"))
    return changes


def main() -> int:
    args = parse_args()
    phrases_dir = Path(args.phrases)
    data_dir = phrases_dir / "data"
    if not data_dir.exists():
        print(f"Error: {data_dir} not found")
        return 1

    total_changes = 0
    missing_ids: set[str] = set(CORRECTIONS)
    for path in sorted(data_dir.glob("p*.json")):
        changes = apply_to_chunk_file(path, args.dry_run)
        if not changes:
            continue
        for phrase_id, field, change in changes:
            print(f"  {path.name} [{phrase_id}] {field}: {change}")
            missing_ids.discard(phrase_id)
            total_changes += 1

    if missing_ids:
        print(f"\nWARNING: {len(missing_ids)} phrase id(s) not found in any chunk: {sorted(missing_ids)}")
    print(f"\nTotal field updates: {total_changes}")
    if args.dry_run:
        print("--dry-run: no files modified.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
