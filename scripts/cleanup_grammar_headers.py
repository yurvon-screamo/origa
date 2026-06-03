"""Clean emoji-headers and trailing --- from grammar.json section fields.

Removes:
  - First-line emoji headers (e.g. "📖 What is it?\\n\\n") from section fields
  - Trailing "---" separators at end of section fields (preserving --- inside markdown tables)

Usage:
  python scripts/cleanup_grammar_headers.py [--dry-run]
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

EMOJI_PATTERN = re.compile(
    r"^[\U0001f4d6\U0001f527\U0001f5e3\U0001f5e3\ufe0f\U0001f4a1\U0001f338\U0001f504][^\n]*\n+"
)
TRAILING_DASHES = re.compile(r"\n+---\s*$")

SECTION_FIELDS: frozenset[str] = frozenset(
    ["explanation", "how_to_form", "examples", "nuances", "pro_tip", "related_patterns"]
)

PROJECT_ROOT = Path(__file__).resolve().parent.parent
GRAMMAR_PATH = PROJECT_ROOT / "cdn" / "grammar" / "grammar.json"


def clean_field(value: str) -> tuple[str, bool, bool]:
    """Clean a single section field.

    Returns (cleaned_value, stripped_header, stripped_trailing).
    """
    if not value:
        return value, False, False

    stripped_header = False
    stripped_trailing = False

    cleaned = EMOJI_PATTERN.sub("", value, count=1)
    if cleaned != value:
        stripped_header = True

    prev = cleaned
    cleaned = TRAILING_DASHES.sub("", cleaned)
    if cleaned != prev:
        stripped_trailing = True

    return cleaned, stripped_header, stripped_trailing


def show_diff(rule_id: str, lang: str, field: str, before: str, after: str) -> None:
    """Print a before/after diff for one field."""
    before_preview = before[:120].replace("\n", "\\n")
    after_preview = after[:120].replace("\n", "\\n")
    print(f"  [{rule_id}] {lang}.{field}:")
    print(f"    BEFORE: {before_preview}{'...' if len(before) > 120 else ''}")
    print(f"    AFTER:  {after_preview}{'...' if len(after) > 120 else ''}")
    print()


def main() -> None:
    parser = argparse.ArgumentParser(description="Clean grammar.json emoji-headers and trailing ---")
    parser.add_argument("--dry-run", action="store_true", help="Show diff for first 10 rules, don't write")
    args = parser.parse_args()

    if not GRAMMAR_PATH.exists():
        print(f"ERROR: {GRAMMAR_PATH} not found", file=sys.stderr)
        sys.exit(1)

    raw = GRAMMAR_PATH.read_text(encoding="utf-8")
    data = json.loads(raw)

    rules: list[dict] = data.get("grammar", [])
    total_rules = len(rules)

    total_headers_stripped = 0
    total_trailing_stripped = 0
    diff_shown = 0

    for rule in rules:
        content = rule.get("content", {})
        rule_id = rule.get("rule_id", "unknown")

        for lang_key in ("English", "Russian"):
            lang_section = content.get(lang_key)
            if not isinstance(lang_section, dict):
                continue

            for field in SECTION_FIELDS:
                if field not in lang_section:
                    continue

                value = lang_section[field]
                if not isinstance(value, str):
                    continue

                cleaned, stripped_header, stripped_trailing = clean_field(value)

                if stripped_header:
                    total_headers_stripped += 1
                if stripped_trailing:
                    total_trailing_stripped += 1

                if stripped_header or stripped_trailing:
                    if args.dry_run and diff_shown < 10:
                        show_diff(rule_id, lang_key, field, value, cleaned)
                        diff_shown += 1
                    lang_section[field] = cleaned

    if args.dry_run:
        print(f"DRY RUN: {total_rules} rules, {total_headers_stripped} headers, "
              f"{total_trailing_stripped} trailing --- would be stripped")
        return

    output = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
    GRAMMAR_PATH.write_text(output, encoding="utf-8")

    print(f"Validation report:")
    print(f"  Total rules:       {total_rules}")
    print(f"  Headers stripped:  {total_headers_stripped}")
    print(f"  Trailing ---:      {total_trailing_stripped}")

    verification = json.loads(GRAMMAR_PATH.read_text(encoding="utf-8"))
    verified_count = len(verification.get("grammar", []))
    print(f"  Verified rules:    {verified_count}")
    if verified_count != total_rules:
        print(f"  ERROR: rule count mismatch!", file=sys.stderr)
        sys.exit(1)
    print("  OK: JSON parses correctly")


if __name__ == "__main__":
    main()
