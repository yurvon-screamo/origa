"""Scan a grammar JSON store for hybrid Cyrillic+Latin words in Russian fields.

A *hybrid* is an alphabetic token that mixes Cyrillic and Latin letters. Two
classes are reported with different severity:

1. **GLUED hybrids** — the two scripts touch with no separator, e.g.
   ``Пotentialная`` (Cyr+Lat+Cyr), ``итиidan``, ``фразa``. These are almost
   always OCR/typing typos and must be fully Cyrillic. Finding any GLUED
   hybrid makes the process exit non-zero, so this scanner doubles as a CI
   regression guard.

2. **HYPHENATED mixed compounds** — a hyphen separates the scripts, e.g.
   ``V-словарная``, ``te-форма``, ``Adj-основа``, ``Na-прилагательное``.
   These are STANDARD notation in Japanese-learning materials (the Latin
   prefix references a grammatical morpheme/category) and are legitimate.
   They are listed for review only and do NOT affect the exit code.

   Known residual quality issue in this class (tracked separately, out of
   scope for the hybrid-word fix): casing variance of the same term across
   rules, e.g. ``i-прилагательные`` vs ``I-прилагательные``. That is an
   orthographic-casing concern, not script-mixing, so it is intentionally
   not normalized here.

Usage:
    python scripts/scan_hybrid_words.py
    python scripts/scan_hybrid_words.py --file cdn/grammar/grammar.json

Requirements: Python 3.10+ (stdlib only).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path
from typing import Callable, NamedTuple

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_GRAMMAR_PATH = REPO_ROOT / "cdn" / "grammar" / "grammar.json"

CONTEXT_RADIUS = 25

# Direct Cyrillic<->Latin transition inside a separator-free token.
GLUED_TRANSITION = re.compile(r"[а-яёА-ЯЁ][a-zA-Z]|[a-zA-Z][а-яёА-ЯЁ]")
HAS_CYRILLIC = re.compile(r"[а-яёА-ЯЁ]")
HAS_LATIN = re.compile(r"[a-zA-Z]")

# Glued token = contiguous letters, NO hyphen (hyphen is a morpheme separator).
GLUED_TOKEN = re.compile(r"[a-zA-Zа-яёА-ЯЁ]+")
# Hyphenated compound = letters joined by one or more internal hyphens.
HYPHEN_COMPOUND = re.compile(r"[a-zA-Zа-яёА-ЯЁ]+(?:-[a-zA-Zа-яёА-ЯЁ]+)+")

SCAN_FIELDS = (
    "title",
    "short_description",
    "explanation",
    "how_to_form",
    "examples",
    "nuances",
    "pro_tip",
    "related_patterns",
)


class HybridHit(NamedTuple):
    """A single flagged token with enough context to locate and judge it."""

    rule_id: str
    field: str
    word: str
    context: str


def _iter_russian_text(data: dict):
    """Yield ``(rule_id, field_name, value)`` for every non-empty Russian field."""
    for rule in data.get("grammar", []):
        rid = rule.get("rule_id", "?")
        ru = rule.get("content", {}).get("Russian", {})
        for field in SCAN_FIELDS:
            value = ru.get(field)
            if value:
                yield rid, field, value


def scan(
    data: dict,
    token_regex: re.Pattern[str],
    acceptor: Callable[[str], bool],
) -> list[HybridHit]:
    """Collect ``token_regex`` matches that pass ``acceptor`` across Russian fields.

    Shared traversal for both anomaly classes; the per-class logic lives only
    in the ``acceptor`` predicate and the choice of ``token_regex``.
    """
    hits: list[HybridHit] = []
    for rid, field, value in _iter_russian_text(data):
        for match in token_regex.finditer(value):
            word = match.group()
            if not acceptor(word):
                continue
            start, end = match.start(), match.end()
            context = value[max(0, start - CONTEXT_RADIUS):min(len(value), end + CONTEXT_RADIUS)]
            hits.append(HybridHit(rid, field, word, context))
    return hits


def scan_glued(data: dict) -> list[HybridHit]:
    """GLUED hybrids — definite typos (both scripts in one separator-free token)."""
    return scan(data, GLUED_TOKEN, lambda word: bool(GLUED_TRANSITION.search(word)))


def scan_hyphenated(data: dict) -> list[HybridHit]:
    """Hyphenated mixed compounds — review only (standard notation, mostly legitimate)."""
    return scan(
        data,
        HYPHEN_COMPOUND,
        lambda word: bool(HAS_CYRILLIC.search(word) and HAS_LATIN.search(word)),
    )


def _print_glued(hits: list[HybridHit]) -> None:
    print(f"=== GLUED hybrids (definite typos): {len(hits)} ===")
    by_rule: dict[str, list[HybridHit]] = defaultdict(list)
    for hit in hits:
        by_rule[hit.rule_id].append(hit)
    for rid, rule_hits in by_rule.items():
        print(f"  [{rid}] ({len(rule_hits)} hit(s))")
        for hit in rule_hits:
            print(f'      {hit.field}: word="{hit.word}"')
            print(f'        ctx="...{hit.context}..."')


def _print_hyphenated(hits: list[HybridHit]) -> None:
    print(f"=== HYPHENATED mixed compounds (review only): {len(hits)} ===")
    counts: dict[tuple[str, str], int] = defaultdict(int)
    for hit in hits:
        counts[(hit.rule_id, hit.word)] += 1
    for (rid, word), count in sorted(counts.items()):
        print(f'  [{rid}] "{word}" x{count}')


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Scan grammar JSON for hybrid Cyrillic+Latin words in Russian fields.",
    )
    parser.add_argument(
        "--file",
        type=Path,
        default=DEFAULT_GRAMMAR_PATH,
        help="Grammar JSON store to scan (default: cdn/grammar/grammar.json)",
    )
    args = parser.parse_args()

    if not args.file.exists():
        print(f"ERROR: {args.file} not found", file=sys.stderr)
        return 2

    with args.file.open(encoding="utf-8") as handle:
        data = json.load(handle)

    glued = scan_glued(data)
    hyphenated = scan_hyphenated(data)

    _print_glued(glued)
    if glued:
        print()
    _print_hyphenated(hyphenated)

    # Non-zero exit when GLUED hybrids are present so CI can fail the build.
    return 1 if glued else 0


if __name__ == "__main__":
    raise SystemExit(main())
