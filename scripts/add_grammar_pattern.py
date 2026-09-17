#!/usr/bin/env python3
"""Add a `pattern` field to grammar rule content locales (#503 UX schema).

The title convention (#501) fuses two things into one string:
`title = "pattern（qualifier）"`. The qualifier duplicates
short_description (verified across the corpus: all 22 same-pattern rule
groups are already distinguished by sd), while the tokenizer's string
anchors move to rule_ids — so the qualifier has no remaining consumer in
new code. This migration materializes the clean pattern as its own field:

    { "title": "～も（тоже・дажえ）", "pattern": "～も", ... }

`title` stays untouched as a legacy field for already-released clients
(serde ignores unknown fields, old readers keep working). The Rust model
exposes `pattern()` with a runtime fallback to splitting the title for
stores predating this migration.

Splitting rules are imported from `scripts/_grammar_title.py` — the
deploy validator, single source of truth. Idempotent.

Usage:
    python scripts/add_grammar_pattern.py --dry-run   # report only
    python scripts/add_grammar_pattern.py             # rewrite grammar_v2.json
"""

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _grammar_title import strip_trailing_qualifiers  # noqa: E402 — single source of truth

LOCALES = ("English", "Russian")


def pattern_of(title: str) -> str:
    """The pattern part of a legacy title: trailing qualifier groups
    removed, INNER parenthetical groups kept — `（よ）` optionality and
    `（さ）` partial forms are part of the grammar pattern, not glosses.
    """
    return strip_trailing_qualifiers(title)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="report what would change without rewriting the file",
    )
    parser.add_argument(
        "--corpus",
        type=Path,
        default=Path("cdn/grammar/grammar_v2.json"),
        help="path to the grammar store (default: cdn/grammar/grammar_v2.json)",
    )
    args = parser.parse_args()

    raw = args.corpus.read_text(encoding="utf-8")
    data = json.loads(raw)

    changed = 0
    inconsistent = 0
    for rule in data["grammar"]:
        content = rule.get("content", {})
        for locale in LOCALES:
            section = content.get(locale)
            if not section:
                continue
            title = section.get("title", "")
            expected = pattern_of(title)
            existing = section.get("pattern")
            if existing is not None and existing != expected:
                print(
                    f"INCONSISTENT {rule['rule_id']}/{locale}: "
                    f"pattern={existing!r} but title implies {expected!r}",
                    file=sys.stderr,
                )
                inconsistent += 1
            if existing != expected:
                section["pattern"] = expected
                changed += 1

    if inconsistent:
        return 1

    print(f"{changed} pattern fields {'would be' if args.dry_run else ''} set")
    if args.dry_run:
        return 0

    args.corpus.write_text(
        json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"written: {args.corpus}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
