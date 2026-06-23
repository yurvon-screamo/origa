"""Split semicolon-joined translations in cdn/dictionary/chunk_*.json.

Context (issue #178 L-4): entries like ``意思`` carry translations stored as a
single string with ``;`` separators::

    "ru": { "t": ["намерение; Воля; Цель", "смысл; Значение; Суть"], "d": "" }

Each ``;``-fragment is an independent synonym and must become its own array
entry, mirroring the structure already used by clean entries ("кошка", "кот").

This script is the data-side fix; the runtime parser
(``origa::dictionary::vocabulary::split_semicolon_joined_translations``) now
splits on the same rule so the existing CDN content already renders correctly.
Running this script on the CDN normalizes the stored form so future consumers
(static exports, audit tools) see clean arrays without re-running the split.

Idempotent: running twice is a no-op (no ``;`` left to split).

Run::

    python scripts/fix_dict_semicolon_translations.py --dictionary cdn/dictionary
    python scripts/fix_dict_semicolon_translations.py --dictionary cdn/dictionary --dry-run
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from _cdn_io import atomic_write_json

CHUNK_GLOB = "chunk_*.json"


def split_translation_list(values: list[str]) -> tuple[list[str], bool]:
    """Split each entry on ``;`` and drop empty fragments.

    Returns ``(new_list, changed)``. ``changed`` is True iff the output list
    differs from the input (semantically — order and trimmed values matter).
    """
    out: list[str] = []
    changed = False
    for raw in values:
        text = raw.strip()
        if ";" in text:
            parts = [p.strip() for p in text.split(";")]
            parts = [p for p in parts if p]
            if len(parts) != 1 or parts[0] != text:
                changed = True
            out.extend(parts)
        else:
            if text != raw:
                changed = True
            if text:
                out.append(text)
    if len(out) != len(values):
        changed = True
    return out, changed


def process_translation_block(block: dict) -> bool:
    """Mutate a ``{"t": [...], "d": ...}`` block in place. Return changed flag."""
    if not isinstance(block, dict):
        return False
    values = block.get("t")
    if not isinstance(values, list):
        return False
    new_list, changed = split_translation_list(values)
    if changed:
        block["t"] = new_list
    return changed


def process_entry(entry: dict) -> bool:
    """Mutate a vocabulary entry's ru/en translation blocks. Return changed flag."""
    changed = False
    for lang_key in ("ru", "en"):
        block = entry.get(lang_key)
        if isinstance(block, dict):
            changed = process_translation_block(block) or changed
    return changed


def process_chunk_file(path: Path, dry_run: bool) -> tuple[int, int]:
    """Process one chunk file. Return ``(entries_modified, total_entries)``."""
    raw = path.read_text(encoding="utf-8")
    data = json.loads(raw)
    if not isinstance(data, dict):
        return (0, 0)
    modified = 0
    total = 0
    for key, entry in data.items():
        total += 1
        if not isinstance(entry, dict):
            continue
        if process_entry(entry):
            modified += 1
    if modified > 0 and not dry_run:
        atomic_write_json(path, data)
    return (modified, total)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--dictionary",
        type=Path,
        required=True,
        help="Path to cdn/directory directory containing chunk_*.json",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print what would change without writing files",
    )
    args = parser.parse_args(argv)

    dict_dir: Path = args.dictionary
    if not dict_dir.is_dir():
        print(f"ERROR: {dict_dir} is not a directory", file=sys.stderr)
        return 2

    chunk_files = sorted(dict_dir.glob(CHUNK_GLOB))
    if not chunk_files:
        print(f"ERROR: no {CHUNK_GLOB} files found in {dict_dir}", file=sys.stderr)
        return 2

    total_modified = 0
    total_entries = 0
    for path in chunk_files:
        modified, total = process_chunk_file(path, args.dry_run)
        total_modified += modified
        total_entries += total
        action = "would modify" if args.dry_run else "modified"
        if modified:
            print(f"{action}: {path.name} ({modified}/{total} entries)")

    verb = "would change" if args.dry_run else "changed"
    print(
        f"Summary: {verb} {total_modified}/{total_entries} entries across "
        f"{len(chunk_files)} chunk files"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
