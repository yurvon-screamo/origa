"""Remap Duolingo / Spy x Family JLPT levels in well_known_sets_meta.json (#178 S-3).

The meta file marks every Duolingo and Spy x Family set as N5, regardless of
actual content difficulty. The Duolingo Section number encoded in each title
("Section X Unit Y" / "Модуль X Раздел Y") and the Spy x Family content files
themselves both disagree with that blanket N5 tag.

Mapping (Duolingo official difficulty progression, validated against the
content of representative sets):
    Section 1-2  -> N5  (intro: hiragana, basic greetings, simple copula)
    Section 3-4  -> N4  (TE-form, conditionals, casual speech)
    Section 5-6  -> N3  (humble/polite forms, conditionals, abstract topics)

Spy x Family content files all carry `level: "N3"` in their own metadata
(verified across all 12 episodes); the meta file simply wasn't synced.

Run:
    python scripts/remap_duolingo_spy_levels.py --cdn cdn
    python scripts/remap_duolingo_spy_levels.py --cdn cdn --dry-run
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

SECTION_RE = re.compile(r"(?:Section|Модуль)\s+(\d+)", re.IGNORECASE)
SECTION_TO_LEVEL: dict[int, str] = {1: "N5", 2: "N5", 3: "N4", 4: "N4", 5: "N3", 6: "N3"}
SPY_FAMILY_LEVEL = "N3"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cdn",
        required=True,
        help="Path to cdn/ directory (containing well_known_set/)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show planned updates without writing files",
    )
    return parser.parse_args()


def derive_duolingo_level(title_en: str, title_ru: str) -> str | None:
    blob = f"{title_en} {title_ru}"
    m = SECTION_RE.search(blob)
    if not m:
        return None
    section = int(m.group(1))
    return SECTION_TO_LEVEL.get(section)


def is_duolingo(record: dict) -> bool:
    set_type = record.get("set_type", "")
    return set_type in {"DuolingoEn", "DuolingoRu"}


def is_spy_family(record: dict) -> bool:
    return record.get("set_type") == "SpyFamily"


def main() -> int:
    args = parse_args()
    meta_path = Path(args.cdn) / "well_known_set" / "well_known_sets_meta.json"
    if not meta_path.exists():
        print(f"Error: {meta_path} not found")
        return 1

    with open(meta_path, encoding="utf-8") as f:
        records = json.load(f)

    duolingo_changes: list[tuple[str, str, str]] = []
    spy_changes: list[tuple[str, str, str]] = []
    skipped_no_section: list[str] = []

    for record in records:
        rid = record.get("id", "?")
        old_level = record.get("level")

        if is_duolingo(record):
            new_level = derive_duolingo_level(
                record.get("title_en", "") or "",
                record.get("title_ru", "") or "",
            )
            if new_level is None:
                skipped_no_section.append(rid)
                continue
            if new_level != old_level:
                record["level"] = new_level
                duolingo_changes.append((rid, old_level, new_level))
        elif is_spy_family(record):
            if SPY_FAMILY_LEVEL != old_level:
                record["level"] = SPY_FAMILY_LEVEL
                spy_changes.append((rid, old_level, SPY_FAMILY_LEVEL))

    print(f"Duolingo updates: {len(duolingo_changes)}")
    by_change: dict[tuple[str, str], int] = {}
    for _, old, new in duolingo_changes:
        by_change[(old, new)] = by_change.get((old, new), 0) + 1
    for (old, new), count in sorted(by_change.items()):
        print(f"  {old} -> {new}: {count}")

    print(f"\nSpy x Family updates: {len(spy_changes)}")
    for rid, old, new in spy_changes[:5]:
        print(f"  [{rid}] {old} -> {new}")
    if len(spy_changes) > 5:
        print(f"  ... and {len(spy_changes) - 5} more")

    if skipped_no_section:
        print(f"\nWARNING: {len(skipped_no_section)} Duolingo sets with no Section/Модуль in title — left unchanged")
        for rid in skipped_no_section[:5]:
            print(f"  {rid}")

    if not args.dry_run and (duolingo_changes or spy_changes):
        with open(meta_path, "w", encoding="utf-8") as f:
            json.dump(records, f, ensure_ascii=False, separators=(",", ":"))
        print(f"\nWrote {len(duolingo_changes) + len(spy_changes)} updates to {meta_path}")
    elif args.dry_run:
        print("\n--dry-run: no files modified.")
    else:
        print("\nNo changes needed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
