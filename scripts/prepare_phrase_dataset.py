#! scripts/.venv/Scripts/python.exe
"""Prepare phrase dataset for CDN distribution.

Reads phrase_dataset.json, generates deterministic ULIDs, writes
phrase_index.json and chunked data files (pNNNN.json).

Usage:
    python scripts/prepare_phrase_dataset.py
    python scripts/prepare_phrase_dataset.py --input phrase_dataset/phrase_dataset.json
    python scripts/prepare_phrase_dataset.py --chunk-size 500
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import time
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent

DEFAULT_INPUT = PROJECT_ROOT / "phrase_dataset" / "phrase_dataset.json"
DEFAULT_OUTPUT = PROJECT_ROOT / "phrase_dataset" / "dist"

PROGRESS_INTERVAL = 10_000

# Crockford's Base32 alphabet for ULID encoding
_CROCKFORD_BASE32 = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"


def encode_ulid(raw: bytes) -> str:
    """Encode 16 raw bytes into a 26-character Crockford's Base32 ULID string."""
    if len(raw) != 16:
        raise ValueError(f"Expected 16 bytes, got {len(raw)}")

    value = int.from_bytes(raw, byteorder="big")
    chars: list[str] = []
    for _ in range(26):
        chars.append(_CROCKFORD_BASE32[value & 0x1F])
        value >>= 5
    return "".join(reversed(chars))


def make_ulid(numeric_id: int) -> str:
    """Generate a deterministic ULID from a numeric id.

    Timestamp: 48 bits = 0 (fixed epoch).
    Randomness: 80 bits = SHA256(id)[:10 bytes].
    """
    hash_bytes = hashlib.sha256(str(numeric_id).encode()).digest()[:10]
    raw = b"\x00" * 6 + hash_bytes
    return encode_ulid(raw)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Prepare phrase dataset for CDN distribution"
    )
    parser.add_argument(
        "--input",
        type=Path,
        default=DEFAULT_INPUT,
        help="Path to phrase_dataset.json",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_OUTPUT,
        help="Output directory for dist files",
    )
    parser.add_argument(
        "--chunk-size",
        type=int,
        default=1000,
        help="Phrases per chunk file (default: 1000)",
    )
    return parser.parse_args()


def load_dataset(path: Path) -> list[dict]:
    """Load and validate the source dataset."""
    if not path.exists():
        print(f"ERROR: File not found: {path}", file=sys.stderr)
        sys.exit(1)

    print(f"Loading: {path}")
    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)

    if not isinstance(data, dict) or "phrases" not in data:
        print("ERROR: Expected JSON with 'phrases' key", file=sys.stderr)
        sys.exit(1)

    phrases: list[dict] = data["phrases"]
    stats: dict = data.get("stats", {})
    print(f"  Total phrases: {len(phrases)}")
    if stats:
        print(f"  Dataset stats: passed_filter={stats.get('passed_filter', '?')}")
    return phrases


def build_phrase_records(phrases: list[dict]) -> list[dict]:
    """Convert source phrases to index + data records with deterministic ULIDs.

    Returns list of dicts with keys: ulid, tokens, chunk_id, text.
    """
    seen_ulids: set[str] = set()
    records: list[dict] = []
    total = len(phrases)

    for idx, phrase in enumerate(phrases):
        ulid = make_ulid(phrase["id"])

        if ulid in seen_ulids:
            print(f"ERROR: Duplicate ULID '{ulid}' for id={phrase['id']}", file=sys.stderr)
            sys.exit(1)
        seen_ulids.add(ulid)

        records.append({
            "ulid": ulid,
            "tokens": phrase.get("tokens", []),
            "text": phrase.get("text", ""),
        })

        if (idx + 1) % PROGRESS_INTERVAL == 0:
            print(f"  Processed: {idx + 1}/{total}")

    return records


def assign_chunk_ids(records: list[dict], chunk_size: int) -> None:
    """Sort records by ULID and assign chunk_id in-place."""
    records.sort(key=lambda r: r["ulid"])
    for idx, rec in enumerate(records):
        rec["chunk_id"] = idx // chunk_size


def compute_hash(items: list[dict]) -> str:
    """Compute SHA256 hex digest of JSON-serialized index entries."""
    content = json.dumps(items, ensure_ascii=False, separators=(",", ":"))
    return hashlib.sha256(content.encode()).hexdigest()


def write_index(
    output_dir: Path, records: list[dict], total: int, hash_val: str
) -> int:
    """Write phrase_index.json. Returns file size in bytes."""
    index_entries = [
        {"i": rec["ulid"], "t": rec["tokens"], "c": rec["chunk_id"]}
        for rec in records
    ]

    index_data = {
        "v": 1,
        "h": hash_val,
        "total": total,
        "phrases": index_entries,
    }

    index_path = output_dir / "phrase_index.json"
    raw = json.dumps(index_data, ensure_ascii=False, separators=(",", ":"))
    index_path.write_text(raw, encoding="utf-8")
    return len(raw.encode("utf-8"))


def write_chunks(output_dir: Path, records: list[dict], chunk_size: int) -> list[int]:
    """Write chunk files to data/. Returns list of file sizes in bytes."""
    data_dir = output_dir / "data"
    data_dir.mkdir(parents=True, exist_ok=True)

    total = len(records)
    num_chunks = (total + chunk_size - 1) // chunk_size
    sizes: list[int] = []

    for chunk_idx in range(num_chunks):
        start = chunk_idx * chunk_size
        end = min(start + chunk_size, total)
        chunk_records = records[start:end]

        chunk_data = [
            {
                "i": rec["ulid"],
                "x": rec["text"],
                "ru": None,
                "en": None,
            }
            for rec in chunk_records
        ]

        filename = f"p{chunk_idx:04d}.json"
        chunk_path = data_dir / filename
        raw = json.dumps(chunk_data, ensure_ascii=False, separators=(",", ":"))
        chunk_path.write_text(raw, encoding="utf-8")
        sizes.append(len(raw.encode("utf-8")))

        if (chunk_idx + 1) % 20 == 0 or chunk_idx == num_chunks - 1:
            print(f"  Chunks: {chunk_idx + 1}/{num_chunks}")

    return sizes


def validate_output(
    records: list[dict],
    output_dir: Path,
    chunk_size: int,
) -> None:
    """Validate that all files were written correctly."""
    index_path = output_dir / "phrase_index.json"
    data_dir = output_dir / "data"

    with index_path.open("r", encoding="utf-8") as f:
        index_data = json.load(f)

    index_total = index_data["total"]
    if index_total != len(records):
        print(
            f"ERROR: Index total ({index_total}) != records ({len(records)})",
            file=sys.stderr,
        )
        sys.exit(1)

    num_chunks = (len(records) + chunk_size - 1) // chunk_size
    data_count = 0
    for chunk_idx in range(num_chunks):
        chunk_path = data_dir / f"p{chunk_idx:04d}.json"
        if not chunk_path.exists():
            print(f"ERROR: Missing chunk: {chunk_path}", file=sys.stderr)
            sys.exit(1)
        with chunk_path.open("r", encoding="utf-8") as f:
            chunk_data = json.load(f)
        data_count += len(chunk_data)

    if data_count != len(records):
        print(
            f"ERROR: Chunk data count ({data_count}) != records ({len(records)})",
            file=sys.stderr,
        )
        sys.exit(1)

    print("  Validation: PASSED")


def print_report(
    elapsed: float,
    record_count: int,
    index_bytes: int,
    chunk_sizes: list[int],
    output_dir: Path,
) -> None:
    """Print summary report of the run."""
    num_chunks = len(chunk_sizes)
    total_data_bytes = sum(chunk_sizes)

    print()
    print("=" * 60)
    print("  REPORT")
    print("=" * 60)
    print(f"  Records:        {record_count:,}")
    print(f"  Chunks:         {num_chunks}")
    print(f"  Index size:     {index_bytes / (1024 * 1024):.2f} MB")
    print(f"  Data size:      {total_data_bytes / (1024 * 1024):.2f} MB")
    print(f"  Total size:     {(index_bytes + total_data_bytes) / (1024 * 1024):.2f} MB")
    print(f"  Elapsed:        {elapsed:.1f}s")
    print(f"  Output dir:     {output_dir}")
    print("=" * 60)


def main() -> None:
    args = parse_args()
    t_start = time.monotonic()

    phrases = load_dataset(args.input)
    print(f"Building ULID records...")
    records = build_phrase_records(phrases)

    print(f"Sorting and assigning chunk ids (chunk_size={args.chunk_size})...")
    assign_chunk_ids(records, args.chunk_size)

    output_dir = args.output
    output_dir.mkdir(parents=True, exist_ok=True)

    index_items = [
        {"i": r["ulid"], "t": r["tokens"], "c": r["chunk_id"]} for r in records
    ]
    hash_val = compute_hash(index_items)

    print("Writing phrase_index.json...")
    index_bytes = write_index(output_dir, records, len(records), hash_val)

    print("Writing chunk files...")
    chunk_sizes = write_chunks(output_dir, records, args.chunk_size)

    print("Validating output...")
    validate_output(records, output_dir, args.chunk_size)

    elapsed = time.monotonic() - t_start
    print_report(elapsed, len(records), index_bytes, chunk_sizes, output_dir)


if __name__ == "__main__":
    main()
