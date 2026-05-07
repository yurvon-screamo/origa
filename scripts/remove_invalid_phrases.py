"""
Remove invalid phrases from the phrases dataset.

Reads the validation report, removes invalid phrases from:
1. Chunk JSON files (cdn/phrases/data/p*.json)
2. Phrase index file (cdn/phrases/phrase_index.json)
3. Audio files (cdn/phrases/audio/{id}.opus)

Usage:
    python scripts/remove_invalid_phrases.py --report validation_report.json --phrases cdn/phrases

Requirements:
    pip install requests tqdm
"""

import json
import argparse
import os
import hashlib
from pathlib import Path
from datetime import datetime


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Remove invalid phrases from the phrases dataset"
    )
    parser.add_argument(
        "--report",
        required=True,
        help="Path to validation report JSON file",
    )
    parser.add_argument(
        "--phrases",
        required=True,
        help="Path to phrases directory (containing data/, audio/, phrase_index.json)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview changes without modifying files",
    )
    return parser.parse_args()


def load_invalid_ids(report_path: Path) -> set[str]:
    """Load invalid phrase IDs from validation report."""
    with open(report_path, encoding="utf-8") as f:
        data = json.load(f)
    ids = set(data.get("invalid_phrase_ids", []))
    return ids


def collect_chunk_files(data_dir: Path) -> list[Path]:
    """Collect all chunk JSON files."""
    chunks_dir = data_dir / "data"
    return sorted(chunks_dir.glob("p*.json"))


def remove_from_chunk(chunk_path: Path, invalid_ids: set[str], dry_run: bool) -> dict:
    """Remove invalid phrases from a chunk file."""
    with open(chunk_path, encoding="utf-8") as f:
        phrases = json.load(f)

    original_count = len(phrases)
    filtered = [p for p in phrases if p["i"] not in invalid_ids]
    removed_count = original_count - len(filtered)

    if removed_count == 0:
        return {"path": str(chunk_path), "removed": 0}

    if not dry_run:
        with open(chunk_path, "w", encoding="utf-8") as f:
            json.dump(filtered, f, ensure_ascii=False, separators=(",", ":"))

    return {"path": str(chunk_path), "removed": removed_count}


def remove_from_index(index_path: Path, invalid_ids: set[str], dry_run: bool) -> dict:
    """Remove invalid phrases from the phrase index file."""
    with open(index_path, encoding="utf-8") as f:
        data = json.load(f)

    original_count = len(data["phrases"])
    data["phrases"] = [p for p in data["phrases"] if p["i"] not in invalid_ids]
    removed_count = original_count - len(data["phrases"])

    # Recalculate total
    data["total"] = len(data["phrases"])

    if not dry_run:
        with open(index_path, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, separators=(",", ":"))

    return {"path": str(index_path), "removed": removed_count, "new_total": len(data["phrases"])}


def remove_audio_files(audio_dir: Path, invalid_ids: set[str], dry_run: bool) -> dict:
    """Remove audio files for invalid phrases."""
    removed = 0
    not_found = 0

    for phrase_id in invalid_ids:
        audio_path = audio_dir / f"{phrase_id}.opus"
        if audio_path.exists():
            if not dry_run:
                audio_path.unlink()
            removed += 1
        else:
            not_found += 1

    return {"removed": removed, "not_found": not_found}


def main():
    args = parse_args()

    report_path = Path(args.report)
    phrases_dir = Path(args.phrases)

    if not report_path.exists():
        print(f"Error: Report file not found: {report_path}")
        return

    if not phrases_dir.exists():
        print(f"Error: Phrases directory not found: {phrases_dir}")
        return

    invalid_ids = load_invalid_ids(report_path)
    print(f"Loaded {len(invalid_ids)} invalid phrase IDs from report")

    index_path = phrases_dir / "phrase_index.json"
    audio_dir = phrases_dir / "audio"
    chunk_files = collect_chunk_files(phrases_dir)

    print(f"Found {len(chunk_files)} chunk files")
    print(f"Audio dir: {audio_dir}")
    print(f"Index file: {index_path}")
    print()

    if args.dry_run:
        print("=== DRY RUN - no files will be modified ===\n")

    # Remove from chunks
    total_removed_chunks = 0
    for chunk_path in chunk_files:
        result = remove_from_chunk(chunk_path, invalid_ids, args.dry_run)
        total_removed_chunks += result["removed"]
        if result["removed"] > 0:
            print(f"  {result['path']}: removed {result['removed']} phrases")

    # Remove from index
    print()
    index_result = remove_from_index(index_path, invalid_ids, args.dry_run)
    print(f"  {index_result['path']}: removed {index_result['removed']} entries, new total: {index_result['new_total']}")

    # Remove audio
    print()
    audio_result = remove_audio_files(audio_dir, invalid_ids, args.dry_run)
    print(f"  Audio: removed {audio_result['removed']} files, not found: {audio_result['not_found']}")

    print()
    print(f"Total removed from chunks: {total_removed_chunks}")
    print(f"Total invalid IDs: {len(invalid_ids)}")

    if args.dry_run:
        print("\nThis was a dry run. No files were modified.")
    else:
        print(f"\nChanges applied at {datetime.now().isoformat()}")


if __name__ == "__main__":
    main()
