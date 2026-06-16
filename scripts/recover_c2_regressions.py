"""Recover C2 regressions: restore terse notation in code-spans/code-blocks,
fix tautologies, and standardize V-dictionary.

C2 (standardize_grammar_terms.py) was too aggressive — it replaced V-辞書/V-dict
inside structural code-spans and code-blocks, corrupting pattern notations and
creating tautologies. This script fixes:

1. Inside inline code-spans (`` `...` ``): "dictionary form" → "V-辞書" when
   the span also contains Japanese (structural pattern context).
2. Inside code-blocks (``` ... ```): same restoration.
3. Table tautology (rule_41): "| dictionary form | dictionary form |" → fix
   first column to "V-辞書".
4. Parenthetical tautology (rule_06): "dictionary form (dictionary form)" →
   remove redundant parenthetical.
5. V-dictionary in prose → "dictionary form"; V-dictionary in code context →
   "V-辞書".

All changes use raw-text replacement to preserve file formatting.

Usage:
    python scripts/recover_c2_regressions.py --root cdn/grammar/rules/
    python scripts/recover_c2_regressions.py --root cdn/grammar/rules/ --dry-run
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

TARGET_FIELDS = ("explanation", "how_to_form", "nuances", "pro_tip")
JP_RE = re.compile(r"[\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FFF]")


def has_japanese(text: str) -> bool:
    return bool(JP_RE.search(text))


def fix_inline_code_spans(text: str) -> tuple[str, int]:
    """Inside `...` spans: restore terse V-辞書 notation when JP is present."""
    changes = [0]

    def replacer(match: re.Match[str]) -> str:
        content = match.group(1)
        if not has_japanese(content):
            return match.group(0)
        original = content
        content = content.replace("dictionary form", "V-辞書")
        content = content.replace("V-dictionary", "V-辞書")
        if content == original:
            return match.group(0)
        changes[0] += 1
        return f"`{content}`"

    new_text = re.sub(r"`([^`\n]+)`", replacer, text)
    return new_text, changes[0]


def fix_code_blocks(text: str) -> tuple[str, int]:
    """Inside ``` ... ``` blocks: restore terse V-辞書 notation when JP is present."""
    changes = [0]

    def replacer(match: re.Match[str]) -> str:
        content = match.group(1)
        if not has_japanese(content):
            return match.group(0)
        original = content
        content = content.replace("dictionary form", "V-辞書")
        content = content.replace("V-dictionary", "V-辞書")
        if content == original:
            return match.group(0)
        changes[0] += 1
        return f"```\n{content}\n```"

    new_text = re.sub(r"```\n(.*?)\n```", replacer, text, flags=re.DOTALL)
    return new_text, changes[0]


def fix_table_cells(text: str) -> tuple[str, int]:
    """In markdown table rows: restore V-辞書 notation for structural patterns.

    Targets cells like '| dictionary form + ほど |' where "dictionary form" is
    used as a terse verb-form label in a structural pattern. Standalone glosses
    like '| dictionary form |' (without '+ JP pattern') are left untouched.
    """
    changes = [0]

    def replacer(match: re.Match[str]) -> str:
        line = match.group(0)
        if not has_japanese(line):
            return line
        # Only replace "dictionary form" when followed by " + [JP pattern]"
        new_line, n = re.subn(r"\bdictionary form(\s*\+\s*)", r"V-辞書\1", line)
        if n == 0:
            return line
        changes[0] += n
        return new_line

    new_text = re.sub(r"^\|.*$", replacer, text, flags=re.MULTILINE)
    return new_text, changes[0]


def fix_table_tautology(text: str) -> tuple[str, int]:
    """Fix '| dictionary form | dictionary form |' → '| V-辞書 | dictionary form |'."""
    old = "| dictionary form | dictionary form |"
    new = "| V-辞書 | dictionary form |"
    if old not in text:
        return text, 0
    return text.replace(old, new, 1), 1


def fix_parenthetical_tautology(text: str) -> tuple[str, int]:
    """Remove redundant '(dictionary form)' after an existing 'dictionary form'."""
    pattern = re.compile(r"(dictionary form\**)\s*\(dictionary form\)")
    new_text, count = pattern.subn(r"\1", text)
    return new_text, count


def fix_vdictionary_prose(text: str) -> tuple[str, int]:
    """Replace remaining V-dictionary in prose → dictionary form."""
    count = text.count("V-dictionary")
    if count == 0:
        return text, 0
    return text.replace("V-dictionary", "dictionary form"), count


def recover_field(text: str) -> tuple[str, dict[str, int]]:
    """Apply all recovery fixes to a single EN field string."""
    counts: dict[str, int] = {
        "code-span restore": 0,
        "code-block restore": 0,
        "table-cell restore": 0,
        "table tautology": 0,
        "parenthetical tautology": 0,
        "V-dictionary prose": 0,
    }
    text, n = fix_inline_code_spans(text)
    counts["code-span restore"] += n
    text, n = fix_code_blocks(text)
    counts["code-block restore"] += n
    text, n = fix_table_cells(text)
    counts["table-cell restore"] += n
    text, n = fix_table_tautology(text)
    counts["table tautology"] += n
    text, n = fix_parenthetical_tautology(text)
    counts["parenthetical tautology"] += n
    text, n = fix_vdictionary_prose(text)
    counts["V-dictionary prose"] += n
    return text, counts


def process_file(path: Path, dry_run: bool) -> tuple[bool, dict[str, int], bool]:
    """Process one rule file via raw-text replacement.

    Returns (was_modified, aggregated per-fix counts, had_warning).
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
        new_value, changes = recover_field(old_value)
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
    parser = argparse.ArgumentParser(description="Recover C2 regressions in grammar rules")
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

    for path in files:
        modified, changes, warned = process_file(path, args.dry_run)
        if warned:
            had_any_warning = True
        if modified:
            modified_files += 1
            for label, n in changes.items():
                total_changes[label] = total_changes.get(label, 0) + n

    mode = "DRY RUN" if args.dry_run else "APPLIED"
    print(f"\n[{mode}] Files scanned: {len(files)}")
    print(f"[{mode}] Files modified: {modified_files}")
    print(f"[{mode}] Per-fix counts:")
    for label, n in total_changes.items():
        print(f"    {label}: {n}")

    return 1 if had_any_warning else 0


if __name__ == "__main__":
    sys.exit(main())
