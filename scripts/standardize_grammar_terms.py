"""Standardize terminology in English content fields of grammar rules.

Canonical forms (EN fields only):
  - te-form         (was: "て-form", "te form")
  - dictionary form (was: "V-辞書", "V-辞書形", "V-dict", "dictionary-form")
  - nai-stem        (was: "nai stem", "nai-form")

Processes ONLY content.English.{explanation, how_to_form, nuances, pro_tip}.
Russian fields, examples, titles, and short_description are NOT touched.

Implementation: raw-text replacement via json.dumps round-trip to preserve
the original file formatting (compact arrays, brace spacing, key order).

Usage:
    python scripts/standardize_grammar_terms.py --root cdn/grammar/rules/
    python scripts/standardize_grammar_terms.py --root cdn/grammar/rules/ --dry-run
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

TARGET_FIELDS = ("explanation", "how_to_form", "nuances", "pro_tip")

# Ordered replacements. Kana-variant first, then space-variant, so we never
# double-process a string that was already canonicalized.
REPLACEMENTS: list[tuple[re.Pattern[str], str, str]] = [
    (re.compile(r"て-form"), "te-form", "て-form -> te-form"),
    (re.compile(r"\bte form\b"), "te-form", "te form -> te-form"),
    (re.compile(r"V-辞書形?"), "dictionary form", "V-辞書(形) -> dictionary form"),
    (re.compile(r"\bV-dict\b"), "dictionary form", "V-dict -> dictionary form"),
    (re.compile(r"\bdictionary-form\b"), "dictionary form", "dictionary-form -> dictionary form"),
    (re.compile(r"\bnai stem\b"), "nai-stem", "nai stem -> nai-stem"),
    (re.compile(r"\bnai-form\b"), "nai-stem", "nai-form -> nai-stem"),
]


def standardize_text(text: str) -> tuple[str, dict[str, int]]:
    """Apply all replacements to PROSE only (code-spans/code-blocks are protected).

    Inline code-spans (`` `...` ``) and code-blocks (``` ... ```) are masked
    before replacement and restored after, so terse notations like V-辞書 inside
    structural patterns are never corrupted.
    """
    # Mask code-spans, code-blocks, and markdown table rows to protect terse
    # notations (V-辞書, V-dict) used as structural labels from prose replacement.
    protected: list[str] = []

    def mask(match: re.Match[str]) -> str:
        protected.append(match.group(0))
        return f"\x00{len(protected) - 1}\x00"

    masked = re.sub(r"```\n.*?\n```", mask, text, flags=re.DOTALL)
    masked = re.sub(r"`[^`\n]+`", mask, masked)
    masked = re.sub(r"^\|.*$", mask, masked, flags=re.MULTILINE)

    changes: dict[str, int] = {label: 0 for *_, label in REPLACEMENTS}
    result = masked
    for pattern, replacement, label in REPLACEMENTS:
        result, n = pattern.subn(replacement, result)
        changes[label] = n

    # Restore protected content in reverse order — later masks (e.g. table rows)
    # may contain placeholders from earlier masks (e.g. code-spans inside table
    # cells), so outer masks must be unwrapped before inner placeholders resolve.
    for i in range(len(protected) - 1, -1, -1):
        result = result.replace(f"\x00{i}\x00", protected[i])

    return result, changes


def process_file(path: Path, dry_run: bool) -> tuple[bool, dict[str, int], bool]:
    """Process one rule file via raw-text replacement (preserves formatting).

    Returns (was_modified, aggregated per-pattern counts, had_warning).
    """
    raw = path.read_text(encoding="utf-8")
    data = json.loads(raw)

    content = data.get("content", {})
    english = content.get("English", {})
    if not isinstance(english, dict):
        return (False, {}, False)

    aggregated: dict[str, int] = {}
    modified = False
    had_warning = False

    for field_name in TARGET_FIELDS:
        old_value = english.get(field_name)
        if not isinstance(old_value, str):
            continue
        new_value, changes = standardize_text(old_value)
        if new_value == old_value:
            continue

        old_json = json.dumps(old_value, ensure_ascii=False)
        new_json = json.dumps(new_value, ensure_ascii=False)
        if old_json not in raw:
            print(f"WARN: {path.name} — could not locate {field_name} in raw text", file=sys.stderr)
            had_warning = True
            continue

        raw = raw.replace(old_json, new_json, 1)
        modified = True
        for label, n in changes.items():
            aggregated[label] = aggregated.get(label, 0) + n

    if modified and not dry_run:
        path.write_text(raw, encoding="utf-8")

    return (modified, aggregated, had_warning)


def main() -> int:
    parser = argparse.ArgumentParser(description="Standardize EN terminology in grammar rules")
    parser.add_argument("--root", type=Path, required=True, help="Root directory of rule files")
    parser.add_argument("--dry-run", action="store_true", help="Only report, do not write")
    args = parser.parse_args()

    if not args.root.exists():
        print(f"ERROR: root path does not exist: {args.root}", file=sys.stderr)
        return 1

    files = sorted(args.root.rglob("*.json"))
    total_changes: dict[str, int] = {}
    modified_files = 0
    had_any_warning = False
    modified_paths: list[Path] = []

    for path in files:
        modified, changes, warned = process_file(path, args.dry_run)
        if warned:
            had_any_warning = True
        if modified:
            modified_files += 1
            modified_paths.append(path)
            for label, n in changes.items():
                total_changes[label] = total_changes.get(label, 0) + n

    mode = "DRY RUN" if args.dry_run else "APPLIED"
    print(f"\n[{mode}] Files scanned: {len(files)}")
    print(f"[{mode}] Files modified: {modified_files}")
    print(f"[{mode}] Per-pattern replacement counts:")
    for label, n in total_changes.items():
        print(f"    {label}: {n}")
    if args.dry_run and modified_paths:
        print(f"\n[{mode}] Would modify:")
        for p in modified_paths:
            print(f"    {p}")

    return 1 if had_any_warning else 0


if __name__ == "__main__":
    sys.exit(main())
