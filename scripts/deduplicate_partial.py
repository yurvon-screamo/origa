"""
Detect partial duplicate phrases across all chunks.

Finds phrases with identical Japanese text (x) but different translations (ru/en),
selects the best one (shortest combined translation), and marks the rest for removal.

Best phrase selection algorithm:
    1. score = len(ru) + len(en) — shorter translations are preferred
    2. Tiebreaker: smallest ULID (earliest entry)

Rationale for shortest translation:
    - More precise translations tend to be shorter (no hallucinated additions)
    - Longer translations often contain guesswork and padding
    - For educational content, brevity = clarity

Output is compatible with remove_invalid_phrases.py.

Usage:
    python scripts/deduplicate_partial.py --input cdn/phrases/data --output partial_dedup_report.json
    python scripts/deduplicate_partial.py --input cdn/phrases/data --verbose

Requirements: Python 3.9+ (stdlib only).
"""

import argparse
import json
import sys
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Detect partial duplicate phrases (same Japanese text, different translations)"
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Path to directory containing p*.json chunk files",
    )
    parser.add_argument(
        "--output",
        default=None,
        help="Path to write the partial dedup report JSON (default: stdout summary only)",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Print top-10 largest partial duplicate groups to stderr",
    )
    return parser.parse_args()


def collect_chunk_files(data_dir: Path) -> list[Path]:
    """Collect all p*.json chunk files sorted by name."""
    return sorted(data_dir.glob("p*.json"))


def read_chunks_and_group(
    chunk_files: list[Path],
) -> dict[str, list[tuple[str, int, str, str]]]:
    """
    Read all chunks and group phrase data by Japanese text (x).

    Returns a mapping: x -> list of (ulid, chunk_index, ru, en).

    Memory-efficient: we only store the fields needed for scoring,
    not the full phrase objects.
    """
    groups: dict[str, list[tuple[str, int, str, str]]] = defaultdict(list)

    for chunk_path in chunk_files:
        chunk_name = chunk_path.stem
        try:
            chunk_index = int(chunk_name[1:])
        except ValueError:
            chunk_index = 0

        with open(chunk_path, encoding="utf-8") as f:
            phrases = json.load(f)

        for phrase in phrases:
            phrase_id = phrase.get("i", "")
            if not phrase_id:
                continue

            x = phrase.get("x", "")
            ru = phrase.get("ru", "")
            en = phrase.get("en", "")

            groups[x].append((phrase_id, chunk_index, ru, en))

    return groups


def select_best_phrase(
    entries: list[tuple[str, int, str, str]],
) -> tuple[str, int, str, str]:
    """
    Select the best phrase from a group of partial duplicates.

    Selection criteria:
        1. Smallest score = len(ru) + len(en) (shortest translation wins)
        2. Tiebreaker: smallest ULID (earliest entry wins)

    Returns the (ulid, chunk_index, ru, en) tuple of the best phrase.
    """
    return min(entries, key=lambda e: (len(e[2]) + len(e[3]), e[0]))


def analyze_groups(
    groups: dict[str, list[tuple[str, int, str, str]]],
) -> tuple[
    list[str],
    list[dict],
    list[dict],
    int,
    int,
]:
    """
    Analyze grouped phrases and produce partial deduplication results.

    For each group with count >= 2:
        - Select the best phrase (shortest translation, earliest ULID)
        - Mark all others as invalid

    Returns:
        (invalid_ids, invalid_details, groups_summary,
         total_phrases, total_partial_duplicate_groups)
    """
    invalid_ids: list[str] = []
    invalid_details: list[dict] = []
    groups_summary: list[dict] = []
    total_phrases = 0
    total_partial_duplicate_groups = 0

    for x, entries in groups.items():
        count = len(entries)
        total_phrases += count

        if count <= 1:
            continue

        total_partial_duplicate_groups += 1

        best = select_best_phrase(entries)
        kept_id = best[0]
        kept_score = len(best[2]) + len(best[3])

        removed_entries = [
            (ulid, chunk_idx, ru, en)
            for ulid, chunk_idx, ru, en in entries
            if ulid != kept_id
        ]

        for removed_id, chunk_index, ru, en in removed_entries:
            invalid_ids.append(removed_id)
            invalid_details.append(
                {
                    "i": removed_id,
                    "chunk": chunk_index,
                    "reason": "partial_duplicate",
                    "kept_id": kept_id,
                    "x": x[:80] if x else "",
                }
            )

        groups_summary.append(
            {
                "text": x,
                "total_copies": count,
                "kept_id": kept_id,
                "kept_score": kept_score,
                "removed_ids": [rid for rid, _, _, _ in removed_entries],
            }
        )

    return (
        invalid_ids,
        invalid_details,
        groups_summary,
        total_phrases,
        total_partial_duplicate_groups,
    )


def build_report(
    invalid_ids: list[str],
    invalid_details: list[dict],
    groups_summary: list[dict],
    total_phrases: int,
    total_partial_duplicate_groups: int,
) -> dict:
    """Build the output report dict compatible with remove_invalid_phrases.py."""
    return {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "script_version": "1.0",
        "summary": {
            "total_phrases": total_phrases,
            "partial_duplicate_groups": total_partial_duplicate_groups,
            "phrases_to_remove": len(invalid_ids),
            "phrases_kept": total_phrases - len(invalid_ids),
        },
        "invalid_phrase_ids": invalid_ids,
        "invalid_phrases_details": invalid_details,
        "partial_duplicate_groups_summary": groups_summary,
    }


def print_verbose_summary(report: dict) -> None:
    """Print top-10 largest partial duplicate groups to stderr."""
    summary = report["partial_duplicate_groups_summary"]
    summary.sort(key=lambda g: g["total_copies"], reverse=True)

    top = summary[:10]
    print("\n=== Top-10 largest partial duplicate groups ===\n", file=sys.stderr)
    print(
        f"{'#':<4} {'Copies':>7}  {'Kept Score':>10}  {'Kept ID':<28}  {'Text'}",
        file=sys.stderr,
    )
    print("-" * 100, file=sys.stderr)

    for idx, group in enumerate(top, 1):
        text_display = group["text"][:50] + ("..." if len(group["text"]) > 50 else "")
        print(
            f"{idx:<4} {group['total_copies']:>7}  {group['kept_score']:>10}  "
            f"{group['kept_id']:<28}  {text_display}",
            file=sys.stderr,
        )


def main() -> None:
    args = parse_args()

    data_dir = Path(args.input)
    if not data_dir.exists():
        print(f"Error: Input directory not found: {data_dir}", file=sys.stderr)
        sys.exit(1)

    chunk_files = collect_chunk_files(data_dir)
    if not chunk_files:
        print(f"Error: No p*.json files found in {data_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Scanning {len(chunk_files)} chunk files in {data_dir}...", file=sys.stderr)

    groups = read_chunks_and_group(chunk_files)

    (
        invalid_ids,
        invalid_details,
        groups_summary,
        total_phrases,
        total_partial_duplicate_groups,
    ) = analyze_groups(groups)

    report = build_report(
        invalid_ids,
        invalid_details,
        groups_summary,
        total_phrases,
        total_partial_duplicate_groups,
    )

    # Print summary to stderr
    s = report["summary"]
    print(f"\nSummary:", file=sys.stderr)
    print(f"  Total phrases:              {s['total_phrases']}", file=sys.stderr)
    print(f"  Partial duplicate groups:   {s['partial_duplicate_groups']}", file=sys.stderr)
    print(f"  Phrases to remove:          {s['phrases_to_remove']}", file=sys.stderr)
    print(f"  Phrases kept:               {s['phrases_kept']}", file=sys.stderr)

    if args.verbose:
        print_verbose_summary(report)

    # Write report
    if args.output:
        output_path = Path(args.output)
        with open(output_path, "w", encoding="utf-8") as f:
            json.dump(report, f, ensure_ascii=False, indent=2)
        print(f"\nReport written to: {output_path}", file=sys.stderr)
    else:
        print("\nNo --output specified; report not saved.", file=sys.stderr)


if __name__ == "__main__":
    main()
