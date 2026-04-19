#! scripts/.venv/Scripts/python.exe
r"""Extract audio from HF dataset cache (joujiboi/japanese-anime-speech-v2).

Uses pyarrow directly to read Arrow files from the HuggingFace cache.
Avoids the datasets library which crashes with torchcodec DLL on Windows.

Usage:
    scripts/.venv/Scripts/python.exe scripts/extract_audio.py
    scripts/.venv/Scripts/python.exe scripts/extract_audio.py --cache-dir D:\hf_cache
    scripts/.venv/Scripts/python.exe scripts/extract_audio.py --phrases my_phrases.json
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path

import pyarrow
import pyarrow.ipc as ipc

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
DEFAULT_PHRASES = PROJECT_ROOT / "phrase_dataset" / "selected_phrases.json"
DEFAULT_OUTPUT = PROJECT_ROOT / "origa_ui" / "public" / "phrase" / "audio"
INDEX_FILE = SCRIPT_DIR / "transcription_index.json"


def default_cache_dir() -> Path:
    """Return the default HF datasets cache directory."""
    return Path.home() / ".cache" / "huggingface" / "datasets"


def find_arrow_files(cache_dir: Path) -> list[Path]:
    """Find Arrow files inside joujiboi/japanese-anime-speech-v2 cache dirs."""
    if not cache_dir.exists():
        return []

    results: list[Path] = []
    for arrow_path in cache_dir.rglob("*.arrow"):
        path_str = arrow_path.as_posix().lower()
        if "joujiboi" in path_str or "japanese-anime-speech" in path_str:
            results.append(arrow_path)

    return sorted(results)


def load_index() -> dict[str, str]:
    """Load transcription -> arrow_file mapping from disk."""
    if not INDEX_FILE.exists():
        return {}
    with INDEX_FILE.open("r", encoding="utf-8") as f:
        return json.load(f)


def save_index(index: dict[str, str]) -> None:
    """Persist transcription index to disk."""
    with INDEX_FILE.open("w", encoding="utf-8") as f:
        json.dump(index, f, ensure_ascii=False, indent=2)


def read_arrow_table(arrow_path: Path) -> pyarrow.Table:
    """Read an Arrow file using RecordBatchStreamReader."""
    with arrow_path.open("rb") as f:
        reader = ipc.open_stream(f)
        return reader.read_all()


def build_index_from_arrow(
    arrow_files: list[Path],
    existing_index: dict[str, str],
) -> dict[str, str]:
    """Scan Arrow files for transcriptions not yet in the index."""
    total = len(arrow_files)
    new_entries = 0

    for i, arrow_path in enumerate(arrow_files, 1):
        arrow_str = arrow_path.as_posix()
        already_has = any(v == arrow_str for v in existing_index.values())
        if already_has:
            continue

        print(f"  [{i}/{total}] Indexing {arrow_path.name} ...", end="", flush=True)
        t0 = time.monotonic()
        table = read_arrow_table(arrow_path)
        col = table.column("transcription")
        file_new = 0

        for row_idx in range(len(col)):
            val = col[row_idx].as_py()
            if val is None:
                continue
            text = str(val).strip()
            if text and text not in existing_index:
                existing_index[text] = arrow_str
                file_new += 1

        elapsed = time.monotonic() - t0
        new_entries += file_new
        print(f" {len(col)} rows, {file_new} new ({elapsed:.1f}s)")

    if new_entries:
        save_index(existing_index)
        print(f"Index updated: +{new_entries} transcriptions")

    return existing_index


def extract_audio_bytes(arrow_path: Path, transcription: str) -> bytes | None:
    """Extract audio bytes for a matching transcription from an Arrow file."""
    table = read_arrow_table(arrow_path)
    transcription_col = table.column("transcription")
    audio_col = table.column("audio")

    for row_idx in range(len(transcription_col)):
        val = transcription_col[row_idx].as_py()
        if val is None:
            continue
        if str(val).strip() == transcription:
            raw = audio_col[row_idx].as_py()
            return _decode_audio(raw)

    return None


def _decode_audio(raw: object) -> bytes | None:
    """Handle both dict{bytes: ...} and raw bytes formats."""
    if isinstance(raw, bytes):
        return raw
    if isinstance(raw, dict):
        if "bytes" in raw:
            return raw["bytes"]
    if isinstance(raw, memoryview):
        return bytes(raw)
    if hasattr(raw, "to_pybytes"):
        return raw.to_pybytes()
    return None


def load_phrases(path: Path) -> list[dict]:
    """Load selected phrases JSON file."""
    if not path.exists():
        print(f"ERROR: Phrases file not found: {path}", file=sys.stderr)
        sys.exit(1)

    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)

    # Handle both {"phrases": [...]} and [...] formats
    if isinstance(data, dict):
        phrases = data.get("phrases")
        if phrases is None:
            print("ERROR: JSON must have phrases key or be an array", file=sys.stderr)
            sys.exit(1)
        return phrases

    if isinstance(data, list):
        return data

    print("ERROR: Expected JSON array or object with phrases key", file=sys.stderr)
    sys.exit(1)


def find_audio_for_phrase(
    phrase: dict,
    index: dict[str, str],
) -> tuple[bytes | None, str]:
    """Try to find audio for a phrase. Returns (audio_bytes, status)."""
    text = phrase.get("text", "").strip()
    if not text:
        return None, "no_text"

    arrow_path_str = index.get(text)
    if arrow_path_str is None:
        return None, "not_indexed"

    arrow_path = Path(arrow_path_str)
    if not arrow_path.exists():
        return None, "arrow_missing"

    audio = extract_audio_bytes(arrow_path, text)
    if audio is None:
        return None, "transcription_not_found"

    return audio, "found"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Extract audio from HF dataset cache for selected phrases"
    )
    parser.add_argument(
        "--phrases",
        type=Path,
        default=DEFAULT_PHRASES,
        help="Path to selected_phrases.json",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_OUTPUT,
        help="Output directory",
    )
    parser.add_argument(
        "--cache-dir",
        type=Path,
        default=None,
        help="HF datasets cache directory",
    )
    parser.add_argument(
        "--rebuild-index",
        action="store_true",
        help="Force rebuild the transcription index from scratch",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    phrases = load_phrases(args.phrases)
    print(f"Loaded {len(phrases)} phrases from {args.phrases}")

    cache_dir = args.cache_dir or default_cache_dir()
    print(f"Searching for Arrow files in: {cache_dir}")

    arrow_files = find_arrow_files(cache_dir)
    if not arrow_files:
        print("ERROR: No Arrow files found. Check cache directory.", file=sys.stderr)
        sys.exit(1)

    print(f"Found {len(arrow_files)} Arrow file(s)")

    index = {} if args.rebuild_index else load_index()
    print(f"Index: {len(index)} transcriptions cached")

    if len(index) == 0 or args.rebuild_index:
        print("Building transcription index ...")
        index = build_index_from_arrow(arrow_files, index)
        print(f"Index complete: {len(index)} transcriptions")
    else:
        indexed_files = set(index.values())
        unindexed = [p for p in arrow_files if p.as_posix() not in indexed_files]
        if unindexed:
            print(f"Found {len(unindexed)} unindexed Arrow file(s), updating index ...")
            index = build_index_from_arrow(arrow_files, index)

    output_dir: Path = args.output
    output_dir.mkdir(parents=True, exist_ok=True)

    counts = {"found": 0, "skipped": 0, "missing": 0}
    total = len(phrases)
    t_start = time.monotonic()

    for i, phrase in enumerate(phrases, 1):
        phrase_id = phrase.get("id")
        text = phrase.get("text", "")
        output_file = output_dir / f"{phrase_id:05d}.mp3"

        if output_file.exists() and output_file.stat().st_size > 0:
            counts["skipped"] += 1
            if i % 100 == 0 or i == total:
                elapsed = time.monotonic() - t_start
                print(
                    f"  [{i}/{total}] skipped (exists) | "
                    f"f={counts['found']} s={counts['skipped']} m={counts['missing']} "
                    f"({elapsed:.1f}s)"
                )
            continue

        audio, status = find_audio_for_phrase(phrase, index)

        if audio is None:
            counts["missing"] += 1
            print(f"  [{i}/{total}] MISSING id={phrase_id}: {text[:50]} ({status})")
            continue

        output_file.write_bytes(audio)
        counts["found"] += 1

        if counts["found"] % 50 == 0 or i == total:
            elapsed = time.monotonic() - t_start
            print(
                f"  [{i}/{total}] extracted id={phrase_id} | "
                f"f={counts['found']} s={counts['skipped']} m={counts['missing']} "
                f"({elapsed:.1f}s)"
            )

    elapsed = time.monotonic() - t_start
    print()
    print("=" * 60)
    print(f"  Total:   {total}")
    print(f"  Found:   {counts['found']}")
    print(f"  Skipped: {counts['skipped']}")
    print(f"  Missing: {counts['missing']}")
    print(f"  Time:    {elapsed:.1f}s")
    print(f"  Output:  {output_dir}")
    print("=" * 60)


if __name__ == "__main__":
    main()
