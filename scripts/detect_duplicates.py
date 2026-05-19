"""
Detect exact duplicate phrases across all chunks.

Finds phrases with identical (x, ru, en) triple across chunk files,
keeps the one with the earliest ULID, and marks the rest for removal.

Output is compatible with remove_invalid_phrases.py.

Usage:
    python scripts/detect_duplicates.py --input cdn/phrases/data --output duplicate_report.json
    python scripts/detect_duplicates.py --input cdn/phrases/data --verbose

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
        description="Detect exact duplicate phrases across chunk files"
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Path to directory containing p*.json chunk files",
    )
    parser.add_argument(
        "--output",
        default=None,
        help="Path to write the duplicate report JSON (default: stdout summary only)",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Print top-20 largest duplicate groups to stderr",
    )
    return parser.parse_args()


def collect_chunk_files(data_dir: Path) -> list[Path]:
    """Collect all p*.json chunk files sorted by name."""
    return sorted(data_dir.glob("p*.json"))


def build_duplicate_key(phrase: dict) -> tuple[str, str, str]:
    """Build a deduplication key from (x, ru, en)."""
    return (
        phrase.get("x", ""),
        phrase.get("ru", ""),
        phrase.get("en", ""),
    )


def read_chunks_and_group(
    chunk_files: list[Path],
) -> dict[tuple[str, str, str], list[tuple[str, int]]]:
    """
    Read all chunks and group phrase IDs by deduplication key.

    Returns a mapping: key -> list of (ulid, chunk_index).
    chunk_index is the numeric part of the chunk filename (e.g. 0 for p0000.json).

    Memory-efficient: we only store (ulid, chunk_index) per phrase,
    not the full phrase objects.
    """
    groups: dict[tuple[str, str, str], list[tuple[str, int]]] = defaultdict(list)

    for chunk_path in chunk_files:
        chunk_name = chunk_path.stem
        # Extract numeric index from "p0000" -> 0
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
            key = build_duplicate_key(phrase)
            groups[key].append((phrase_id, chunk_index))

    return groups


def extract_chunk_index(chunk_files: list[Path], chunk_idx: int) -> int:
    """Return the numeric index from a chunk filename like p0042.json -> 42."""
    return chunk_idx


def analyze_groups(
    groups: dict[tuple[str, str, str], list[tuple[str, int]]],
) -> tuple[
    list[str],
    list[dict],
    list[dict],
    int,
    int,
]:
    """
    Analyze grouped phrases and produce deduplication results.

    For each group with count > 1: sort by ULID, keep the earliest,
    mark the rest as invalid.

    Returns:
        (invalid_ids, invalid_details, duplicate_groups_summary,
         total_phrases, total_duplicate_groups)
    """
    invalid_ids: list[str] = []
    invalid_details: list[dict] = []
    duplicate_groups_summary: list[dict] = []
    total_phrases = 0
    total_duplicate_groups = 0

    for key, entries in groups.items():
        count = len(entries)
        total_phrases += count

        if count <= 1:
            continue

        total_duplicate_groups += 1

        # Sort by ULID — ULIDs are lexicographically sortable by time
        entries.sort(key=lambda e: e[0])

        kept_id = entries[0][0]
        removed_entries = entries[1:]

        for removed_id, chunk_index in removed_entries:
            invalid_ids.append(removed_id)
            invalid_details.append(
                {
                    "i": removed_id,
                    "chunk": chunk_index,
                    "reason": "exact_duplicate",
                    "kept_id": kept_id,
                    "text_preview": key[0][:80] if key[0] else "",
                }
            )

        duplicate_groups_summary.append(
            {
                "text": key[0],
                "total_copies": count,
                "kept_id": kept_id,
                "removed_ids": [rid for rid, _ in removed_entries],
            }
        )

    return (
        invalid_ids,
        invalid_details,
        duplicate_groups_summary,
        total_phrases,
        total_duplicate_groups,
    )


def build_report(
    invalid_ids: list[str],
    invalid_details: list[dict],
    duplicate_groups_summary: list[dict],
    total_phrases: int,
    total_duplicate_groups: int,
) -> dict:
    """Build the output report dict compatible with remove_invalid_phrases.py."""
    return {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "script_version": "1.0",
        "summary": {
            "total_phrases": total_phrases,
            "duplicate_groups": total_duplicate_groups,
            "phrases_to_remove": len(invalid_ids),
            "phrases_kept": total_phrases - len(invalid_ids),
        },
        "invalid_phrase_ids": invalid_ids,
        "invalid_phrases_details": invalid_details,
        "duplicate_groups_summary": duplicate_groups_summary,
    }


def print_verbose_summary(report: dict) -> None:
    """Print top-20 largest duplicate groups to stderr."""
    summary = report["duplicate_groups_summary"]
    summary.sort(key=lambda g: g["total_copies"], reverse=True)

    top = summary[:20]
    print("\n=== Top-20 largest duplicate groups ===\n", file=sys.stderr)
    print(
        f"{'#':<4} {'Copies':>7}  {'Kept ID':<28}  {'Text'}",
        file=sys.stderr,
    )
    print("-" * 80, file=sys.stderr)
    for idx, group in enumerate(top, 1):
        text_display = group["text"][:50] + ("..." if len(group["text"]) > 50 else "")
        print(
            f"{idx:<4} {group['total_copies']:>7}  {group['kept_id']:<28}  {text_display}",
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
        duplicate_groups_summary,
        total_phrases,
        total_duplicate_groups,
    ) = analyze_groups(groups)

    report = build_report(
        invalid_ids,
        invalid_details,
        duplicate_groups_summary,
        total_phrases,
        total_duplicate_groups,
    )

    # Print summary to stderr
    s = report["summary"]
    print(f"\nSummary:", file=sys.stderr)
    print(f"  Total phrases:        {s['total_phrases']}", file=sys.stderr)
    print(f"  Duplicate groups:     {s['duplicate_groups']}", file=sys.stderr)
    print(f"  Phrases to remove:    {s['phrases_to_remove']}", file=sys.stderr)
    print(f"  Phrases kept:         {s['phrases_kept']}", file=sys.stderr)

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
