"""Detect profanity / screaming-rude phrases for removal (#178 P-3).

The phrase dataset was extracted from anime/manga subtitle corpora and
includes a long tail of rude/aggressive forms that do not belong in a learning
app's ready-to-study pool. This scanner produces a validation report listing
phrase ids whose source text contains any of a curated set of rude lemmas.

The output JSON has the schema expected by scripts/remove_invalid_phrases.py:
    {"invalid_phrase_ids": ["...", "..."], "total": N}

Run:
    python scripts/detect_profanity_phrases.py --phrases cdn/phrases \
        --report scripts/profanity_report.json
    python scripts/detect_profanity_phrases.py --phrases cdn/phrases --dry-run
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# Curated rude-lemma list. Matched as substrings of the source (x) field, which
# catches conjugations and compounds (やがる/やがれ/しくさりやがった, etc.).
# Verified against jisho.org / common jp_ru slang lexicons:
#   やがる  — derogatory auxiliary ("to do the damn ...")
#   やがれ   — imperative ("do it, damn you")
#   こんちくしょ / こん畜生 — strong curse ("goddammit")
#   くそ / クソ / くそったれ — "shit" / "shithead"
#   てめぇ / 手前 — derogatory "you" (fighting words)
#   野郎    — "bastard" (when not part of a benign compound like 江戸っ子野郎)
PROFANITY_LEMMAS: tuple[str, ...] = (
    "やがる",
    "やがれ",
    "こんちくしょ",
    "こん畜生",
    "クソ",
    "くそったれ",
    "てめぇ",
    "野郎",
)

# クソ (katakan) vs くそ (hiragana) — both included. But we EXCLUDE the bare
# hiragana くす because it is a frequent benign verb ending (e.g. 苔す, 眠るまい
# → looking for "くそ" alone over-matches). Match the longer compound くそったれ.
# We also include "クソ" (katakana) since the profanity spelling is distinctive.
PROFANITY_LEMMAS_HIRAGANA_KUSO_ONLY_COMPOUND: tuple[str, ...] = (
    "くそったれ",
    "クソ",
    "クソったれ",
)


def all_lemmas() -> tuple[str, ...]:
    """Combined lemma set, with the bare-hiragana くそ intentionally omitted."""
    base = [lem for lem in PROFANITY_LEMMAS if lem not in {"クソ", "くそったれ"}]
    return tuple(base) + PROFANITY_LEMMAS_HIRAGANA_KUSO_ONLY_COMPOUND


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--phrases",
        required=True,
        help="Path to phrases directory (containing data/p*.json)",
    )
    parser.add_argument(
        "--report",
        help="Path to write the JSON validation report. If omitted with --dry-run, only prints stats.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Do not write the report file; only print summary and samples.",
    )
    return parser.parse_args()


def scan(data_dir: Path) -> tuple[list[tuple[str, str]], dict[str, list[str]]]:
    """Return (matches, by_lemma) where matches is [(phrase_id, source_x), ...]
    and by_lemma maps each lemma to the list of phrase ids it matched."""
    lemmas = all_lemmas()
    matches: list[tuple[str, str]] = []
    by_lemma: dict[str, list[str]] = {lemma: [] for lemma in lemmas}

    for path in sorted(data_dir.glob("p*.json")):
        with open(path, encoding="utf-8") as f:
            phrases = json.load(f)
        for record in phrases:
            phrase_id = record.get("i", "")
            x = record.get("x", "")
            if not x:
                continue
            for lemma in lemmas:
                if lemma in x:
                    matches.append((phrase_id, x))
                    by_lemma[lemma].append(phrase_id)
                    break
    return matches, by_lemma


def main() -> int:
    args = parse_args()
    phrases_dir = Path(args.phrases)
    data_dir = phrases_dir / "data"
    if not data_dir.exists():
        print(f"Error: {data_dir} not found")
        return 1

    matches, by_lemma = scan(data_dir)
    phrase_ids = [pid for pid, _ in matches]

    print(f"Profanity scan complete.")
    print(f"  Total phrases matched: {len(matches)}")
    print(f"  Unique phrase ids:     {len(set(phrase_ids))}")
    print(f"  Matches per lemma:")
    for lemma, ids in by_lemma.items():
        if ids:
            print(f"    {lemma:<14}  {len(ids):>4}")

    print(f"\n  Sample matched source texts (first 10):")
    for pid, x in matches[:10]:
        print(f"    [{pid}] {x[:80]!r}")

    if not args.dry_run:
        if not args.report:
            print("Error: --report is required when not in --dry-run mode")
            return 1
        report = {
            "invalid_phrase_ids": phrase_ids,
            "total": len(phrase_ids),
            "reason": "profanity (#178 P-3)",
            "lemmas": list(all_lemmas()),
        }
        out_path = Path(args.report)
        with open(out_path, "w", encoding="utf-8") as f:
            json.dump(report, f, ensure_ascii=False, separators=(",", ":"), indent=2)
        print(f"\nWrote report: {out_path} ({len(phrase_ids)} ids)")
    else:
        print("\n--dry-run: report not written.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
