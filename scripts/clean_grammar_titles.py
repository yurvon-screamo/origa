#!/usr/bin/env python3
"""Strip localized parenthetical insertions from grammar rule titles (#503 UX follow-up).

Rule titles mix a Japanese pattern with a localized gloss in fullwidth
parentheses: `～かった（прошедшее время）` / `～かった（past tense）`. With the
UI showing short_description as the primary heading, the gloss duplicates
the heading and turns the pattern into soup.

Which insertions are removed:

- **Localized gloss on a long pattern** — removed. `short_description`
  carries the same information and becomes the UI heading; a long pattern
  (more than SHORT_PATTERN_MAX_CHARS kana) is self-explanatory once the
  heading says what it means.
- **Localized gloss on a short pattern** (≤ 2 kana: `～も`, `～さん`) —
  KEPT: the title convention (#501, `scripts/_grammar_title.py`) makes the
  qualifier REQUIRED there, because bare particles are indistinguishable
  in card lists and lesson questions.
- **Kana-only qualifiers** (`（伝聞）`, `（様態・伝聞）`) — always KEPT: the
  tokenizer's grammar-label search anchors on them (`SouDaVariant` in
  `tokenizer/translation.rs`).

Title convention rules (uniqueness, required qualifiers, allowlist) are
imported from `scripts/_grammar_title.py` — the deploy validator — so the
script cannot drift from them. A strip is applied only when the result
stays unique across the locale. Idempotent.

Usage:
    python scripts/clean_grammar_titles.py --dry-run   # report only
    python scripts/clean_grammar_titles.py             # rewrite grammar_v2.json
"""

import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from _grammar_title import (  # noqa: E402 — deploy validator, single source of truth
    dup_key,
    split_title,
    normalize_pattern,
    SHORT_PATTERN_MAX_CHARS,
)

TRAILING_PAREN = re.compile(r"（([^）]*)）\s*$")
LOCALIZED_GLOSS = re.compile(r"[A-Za-zА-Яа-яЁё]")
LOCALES = ("English", "Russian")


def is_localized_gloss(inner: str) -> bool:
    return bool(LOCALIZED_GLOSS.search(inner))


def pattern_is_long_enough_to_strip(stripped_title: str) -> bool:
    """A pattern may lose its gloss only when it is self-explanatory."""
    pattern, _ = split_title(stripped_title)
    return len(normalize_pattern(pattern)) > SHORT_PATTERN_MAX_CHARS


def clean_title(title: str) -> str | None:
    """Remove the trailing localized gloss, or None when it must stay."""
    original = title
    while True:
        match = TRAILING_PAREN.search(title)
        if not (match and is_localized_gloss(match.group(1))):
            break
        title = title[: match.start()].rstrip()
    if (
        title == original
        or not pattern_is_long_enough_to_strip(title)
        or not title
    ):
        return None
    return title


def resolve_locale(rules: list[dict], locale: str) -> tuple[int, list[str]]:
    """Strip localized glosses while keeping the validator's uniqueness.

    Stripping is iterative: any candidate that would make its dup_key
    (normalized pattern + qualifier, the deploy validator's identity)
    collide with another rule's final title is reverted to the original
    qualified title, and the check re-runs until stable.
    """
    original = {r["rule_id"]: r["content"][locale]["title"] for r in rules}
    candidate = {rid: clean_title(title) for rid, title in original.items()}
    stripped: dict[str, str | None] = dict(candidate)

    reports: list[str] = []
    changed = True
    while changed:
        changed = False
        finals = {rid: stripped[rid] or original[rid] for rid in original}
        keys: dict[str, list[str]] = {}
        for rid, title in finals.items():
            keys.setdefault(dup_key(title), []).append(rid)
        for key, rids in keys.items():
            if len(rids) < 2:
                continue
            for rid in rids:
                if stripped[rid] is not None:
                    stripped[rid] = None
                    reports.append(
                        f"{rid}: kept gloss — '{original[rid]}' collides on '{key}'"
                    )
                    changed = True

    final_titles = {rid: stripped[rid] or original[rid] for rid in original}
    changed_count = sum(
        1 for rid in original if final_titles[rid] != original[rid]
    )

    # Apply and verify global uniqueness.
    for r in rules:
        r["content"][locale]["title"] = final_titles[r["rule_id"]]
    keys = [dup_key(r["content"][locale]["title"]) for r in rules]
    duplicates = {k for k in keys if keys.count(k) > 1}
    if duplicates:
        print(f"UNIQUE VIOLATION in {locale}: {sorted(duplicates)}", file=sys.stderr)
        sys.exit(1)
    return changed_count, reports


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
    rules = data["grammar"]

    total_changed = 0
    for locale in LOCALES:
        changed, conflicts = resolve_locale(rules, locale)
        total_changed += changed
        print(f"{locale}: {changed} titles stripped, {len(conflicts)} kept as disambiguators")
        for report in conflicts:
            print(f"  KEPT {report}")

    if total_changed == 0:
        print("nothing to do")
        return 0

    if args.dry_run:
        print(f"{total_changed} title fields would change")
        return 0

    args.corpus.write_text(
        json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"written: {args.corpus}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
