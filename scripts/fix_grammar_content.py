"""Idempotent content cleanup for grammar rule files.

Applies two mechanical, unambiguous fixes to every per-rule JSON file under
``cdn/grammar/rules/`` (the source-of-truth consumed by ``merge_grammar_chunks``):

1. **Mixed brackets** — an opening Japanese ``「`` paired with a closing French
   ``»`` is normalised to a matching Japanese pair ``「…」``.
2. **Simplified-Chinese → Japanese-kanji word map** — only full-word /
   full-phrase replacements that are unambiguous (e.g. ``图书馆`` → ``図書館``).
   No blind single-character swaps: ``书`` and ``本`` are different words
   (書 = "to write", 本 = "book"), so ``书`` is only ever rewritten as part of a
   known verb fragment such as ``书かせる`` → ``書かせる``.

Non-mechanical fixes (full-sentence translations, full rule rewrites,
Korean/Russian cross-contamination) are handled by hand-authored patch files
under ``n5_legacy_fixes/`` and ``n4_legacy_fixes/`` and are intentionally NOT
covered here.

The script is idempotent: running it twice produces no further changes, so it
is safe to wire into CI as a regression guard. Idempotency holds as long as no
WORD_MAP value contains a substring that matches any key — a future contributor
adding an entry should verify this invariant (search & confirm).

Usage::

    python scripts/fix_grammar_content.py            # apply fixes in place
    python scripts/fix_grammar_content.py --dry-run  # report only, no writes

Requirements: Python 3.10+ (stdlib only).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections.abc import Iterator
from pathlib import Path
from typing import Any

DEFAULT_RULES_DIR = Path("cdn/grammar/rules")

# Opening Japanese 「 paired with closing French » (U+00BB) → matching Japanese 」.
# Non-greedy, no nested brackets.
BRACKET_RE = re.compile(r"「([^「」]*?)»")

# Unambiguous full-word / full-phrase simplified-Chinese → Japanese-kanji map.
# Rationale per entry is captured inline so future editors know WHY each is safe.
WORD_MAP: dict[str, str] = {
    # Compound nouns — identical meaning, only the glyph set differs.
    "图书馆": "図書館",   # library
    "书店": "本屋",       # bookstore
    "银行": "銀行",       # bank
    "汉字": "漢字",       # kanji
    "两国": "両国",       # both nations
    # 餐厅 (Chinese "dining hall/restaurant") → レストラン (katakana
    # "restaurant") is a semantic reinterpretation, not a pure glyph swap.
    # Kept here because in grammar examples 餐厅 was always meant to be a
    # restaurant; there is no common single Japanese kanji compound for it.
    "餐厅": "レストラン",
    "汇率": "為替レート", # exchange rate
    "会议": "会議",       # meeting
    # Grammar-table labels written with simplified glyphs.
    "动词": "動詞",
    "形容词": "形容詞",
    # 辞書形 (dictionary form) — simplified 辞书 only appears as contamination.
    "辞书": "辞書",
    # い-adjective whose first glyph was simplified.
    "难しい": "難しい",
    # 日記 (diary) — simplified 日记 only appears as a contamination.
    "日记": "日記",
    # 書く-family verb fragments. Safe because these suffixes only ever attach
    # to 書く — there is no other common word ending in 书か / 书い.
    "书かせる": "書かせる",
    "书かない": "書かない",
    "书かなく": "書かなく",
    # 見る → 見てもらう: the simplified 见 was a conjugation typo.
    "见てもらう": "見てもらう",
}

# Fields of a rule's per-language content block that hold prose to be cleaned.
STRING_FIELDS = (
    "short_description",
    "title",
    "explanation",
    "how_to_form",
    "examples",
    "nuances",
    "pro_tip",
)


def fix_text(text: str) -> str:
    """Apply bracket normalisation then the word map.

    The bracket regex and the word map operate on disjoint character classes
    (``「``/``»``/``」`` vs CJK glyphs), so their order is not load-bearing for
    correctness; brackets are applied first purely for readability of the
    intermediate state during debugging.
    """
    text = BRACKET_RE.sub(r"「\1」", text)
    for old, new in WORD_MAP.items():
        text = text.replace(old, new)
    return text


def clean_rule(rule: object) -> bool:
    """Apply mechanical fixes to every localised prose field of ``rule``.

    Returns ``True`` if any field changed (so the caller knows to rewrite the
    file). Mutates ``rule`` in place. Returns ``False`` for any input that is
    not a rule dict (e.g. a valid-JSON-but-wrong-shape top-level value), so a
    structurally malformed file is skipped rather than crashing the batch.
    """
    if not isinstance(rule, dict):
        return False
    changed = False
    content = rule.get("content")
    if not isinstance(content, dict):
        return False
    for lang_block in content.values():
        if not isinstance(lang_block, dict):
            continue
        for field in STRING_FIELDS:
            val = lang_block.get(field)
            if not isinstance(val, str):
                continue
            fixed = fix_text(val)
            if fixed != val:
                lang_block[field] = fixed
                changed = True
    return changed


def iter_rule_files(rules_dir: Path) -> Iterator[Path]:
    """Yield every ``rule_*.json`` under non-excluded subdirs of ``rules_dir``.

    Subdirectories whose name starts with ``_`` (samples, scratch) are skipped,
    matching the merge script's own exclusion rule.
    """
    for subdir in sorted(rules_dir.iterdir()):
        if not subdir.is_dir() or subdir.name.startswith("_"):
            continue
        yield from sorted(subdir.glob("rule_*.json"))


def process(rules_dir: Path, dry_run: bool) -> tuple[int, int]:
    """Return ``(files_scanned, files_changed)`` over all rule files.

    A single file that cannot be read or parsed (malformed JSON, wrong
    top-level shape, I/O error) is reported to stderr and skipped so that one
    bad file does not abort the whole batch. ``clean_rule`` additionally
    guards against valid-JSON-but-not-a-rule-dict inputs.
    """
    scanned = 0
    changed = 0
    for path in iter_rule_files(rules_dir):
        scanned += 1
        try:
            with path.open(encoding="utf-8") as f:
                rule = json.load(f)
        except (json.JSONDecodeError, ValueError, OSError) as exc:
            print(f"warning: skipping {path}: {exc}", file=sys.stderr)
            continue
        if clean_rule(rule):
            changed += 1
            if dry_run:
                print(f"[dry-run] would fix: {path}")
            else:
                write_atomic(path, rule)
                print(f"fixed: {path}")
    return scanned, changed


def write_atomic(path: Path, rule: dict[str, Any]) -> None:
    """Write ``rule`` to ``path`` via temp-file + rename, cleaning up the temp
    on failure so a mid-write crash does not leave an orphan ``.json.tmp``."""
    tmp = path.with_suffix(".json.tmp")
    try:
        with tmp.open("w", encoding="utf-8", newline="\n") as f:
            json.dump(rule, f, ensure_ascii=False, indent=2)
            f.write("\n")
        tmp.replace(path)
    except BaseException:
        tmp.unlink(missing_ok=True)
        raise


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Idempotent content cleanup (brackets + simplified-Chinese map) "
        "for grammar rule files."
    )
    parser.add_argument("--dry-run", action="store_true", help="Report only, write nothing")
    parser.add_argument(
        "--rules-dir",
        type=Path,
        default=DEFAULT_RULES_DIR,
        help="Root of per-rule files (default: %(default)s)",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if not args.rules_dir.is_dir():
        print(f"error: rules directory not found: {args.rules_dir}", file=sys.stderr)
        return 1
    scanned, changed = process(args.rules_dir, args.dry_run)
    verb = "would fix" if args.dry_run else "fixed"
    print(
        f"\nScanned {scanned} rule file(s); {verb} {changed}.",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
