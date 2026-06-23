"""
Remove invalid phrases from the phrases dataset.

Reads the validation report, removes invalid phrases from:
1. Chunk JSON files (cdn/phrases/data/p*.json)
2. Phrase index file (cdn/phrases/phrase_index.json)
3. Audio files (cdn/phrases/audio/{id}.opus)

Usage:
    # Remove phrases listed in a validation report
    python scripts/remove_invalid_phrases.py --report validation_report.json --phrases cdn/phrases

    # Verify the stored hash matches what compute_hash would produce (no writes)
    python scripts/remove_invalid_phrases.py --verify-hash --phrases cdn/phrases

Requirements:
    pip install requests tqdm
"""

import json
import argparse
import os
import sys
import tempfile
import hashlib
from pathlib import Path
from datetime import datetime


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Remove invalid phrases from the phrases dataset"
    )
    parser.add_argument(
        "--report",
        help="Path to validation report JSON file (required unless --verify-hash)",
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
    parser.add_argument(
        "--verify-hash",
        action="store_true",
        help="Only verify that phrase_index.json `h` matches compute_hash(phrases); exit non-zero on mismatch",
    )
    args = parser.parse_args()
    if not args.verify_hash and not args.report:
        parser.error("--report is required unless --verify-hash is set")
    return args


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
        tmp_fd, tmp_path = tempfile.mkstemp(dir=chunk_path.parent, suffix=".tmp")
        with os.fdopen(tmp_fd, "w", encoding="utf-8") as f:
            json.dump(filtered, f, ensure_ascii=False, separators=(",", ":"))
        os.replace(tmp_path, chunk_path)

    return {"path": str(chunk_path), "removed": removed_count}


def remove_from_index(index_path: Path, invalid_ids: set[str], dry_run: bool) -> dict:
    """Remove invalid phrases from the phrase index file."""
    with open(index_path, encoding="utf-8") as f:
        data = json.load(f)

    phrases = data.get("phrases", [])
    if not phrases:
        print("Warning: phrase_index.json has no 'phrases' array")
        return {"path": str(index_path), "removed": 0, "new_total": 0}

    original_count = len(phrases)
    data["phrases"] = [p for p in phrases if p["i"] not in invalid_ids]
    removed_count = original_count - len(data["phrases"])

    # Recalculate total
    data["total"] = len(data["phrases"])
    data["h"] = compute_hash(data["phrases"])
    data["v"] = data.get("v", 1) + 1

    if not verify_hash_consistency(data):
        print("Error: hash consistency check failed after update!")

    if not dry_run:
        tmp_fd, tmp_path = tempfile.mkstemp(dir=index_path.parent, suffix=".tmp")
        with os.fdopen(tmp_fd, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, separators=(",", ":"))
        os.replace(tmp_path, index_path)

    return {
        "path": str(index_path),
        "removed": removed_count,
        "new_total": len(data["phrases"]),
    }


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


def compute_hash(phrases: list) -> str:
    """Compute SHA-256 of JSON-serialized index entries, matching Rust producer.

    The Rust producer (utils/src/commands/enrich_phrases_with_grammar.rs::compute_hash)
    serializes each entry via `serde_json::json!` + `serde_json::to_string`. serde_json's
    default Map preserves BTreeMap ordering, so the resulting JSON object keys are emitted
    in alphabetical order: c, g, i, t. Python's `json.dumps(sort_keys=True)` reproduces
    that exact byte sequence.

    Entries are sorted by id (matching Rust's `entries.sort_by(|a, b| a.id.cmp(&b.id))`).
    The grammar_rules array (`g`) is NOT re-sorted here: every existing phrase_index.json
    (v14 and later) already stores `g` sorted (the producer applies `grammar_strs.sort()`
    before writing), so re-sorting would be a no-op. Keeping this layer untouched preserves
    backward-compatibility with v14 hashes if a caller ever passes legacy unsorted `g`.
    """
    entries = [
        {"i": p["i"], "t": p.get("t", []), "c": p.get("c", 0), "g": p.get("g", [])}
        for p in phrases
    ]
    entries.sort(key=lambda e: e["i"])
    serialized = json.dumps(
        entries, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    )
    return hashlib.sha256(serialized.encode("utf-8")).hexdigest()


def verify_hash_consistency(data: dict) -> bool:
    """Verify that the stored hash matches the actual phrases."""
    expected = compute_hash(data["phrases"])
    actual = data.get("h", "")
    if expected != actual:
        print(
            f"Warning: hash mismatch! Expected {expected[:16]}..., got {actual[:16]}..."
        )
        return False
    return True


def main():
    args = parse_args()

    phrases_dir = Path(args.phrases)
    if not phrases_dir.exists():
        print(f"Error: Phrases directory not found: {phrases_dir}")
        return

    index_path = phrases_dir / "phrase_index.json"

    if args.verify_hash:
        with open(index_path, encoding="utf-8") as f:
            data = json.load(f)
        ok = verify_hash_consistency(data)
        stored_version = data.get("v")
        stored_total = data.get("total")
        actual_total = len(data.get("phrases", []))
        print(
            f"phrase_index.json: v={stored_version} total={stored_total} "
            f"actual_phrases={actual_total}"
        )
        if not ok:
            print("RESULT: hash mismatch (FAILED)")
            sys.exit(1)
        print("RESULT: hash matches compute_hash(phrases) (OK)")
        return

    report_path = Path(args.report)
    if not report_path.exists():
        print(f"Error: Report file not found: {report_path}")
        return

    invalid_ids = load_invalid_ids(report_path)
    print(f"Loaded {len(invalid_ids)} invalid phrase IDs from report")

    audio_dir = phrases_dir / "audio"
    chunk_files = collect_chunk_files(phrases_dir)

    print(f"Found {len(chunk_files)} chunk files")
    print(f"Audio dir: {audio_dir}")
    print(f"Index file: {index_path}")
    print()

    if args.dry_run:
        print("=== DRY RUN - no files will be modified ===\n")

    total_removed_chunks = 0
    for chunk_path in chunk_files:
        result = remove_from_chunk(chunk_path, invalid_ids, args.dry_run)
        total_removed_chunks += result["removed"]
        if result["removed"] > 0:
            print(f"  {result['path']}: removed {result['removed']} phrases")

    print()
    index_result = remove_from_index(index_path, invalid_ids, args.dry_run)
    print(
        f"  {index_result['path']}: removed {index_result['removed']} entries, new total: {index_result['new_total']}"
    )

    print()
    audio_result = remove_audio_files(audio_dir, invalid_ids, args.dry_run)
    print(
        f"  Audio: removed {audio_result['removed']} files, not found: {audio_result['not_found']}"
    )

    print()
    print(f"Total removed from chunks: {total_removed_chunks}")
    print(f"Total invalid IDs: {len(invalid_ids)}")

    if args.dry_run:
        print("\nThis was a dry run. No files were modified.")
    else:
        print(f"\nChanges applied at {datetime.now().isoformat()}")


if __name__ == "__main__":
    main()
