#! scripts/.venv/Scripts/python.exe
r"""Extract audio from HF dataset cache (joujiboi/japanese-anime-speech-v2).

Uses pyarrow directly to read Arrow files from the HuggingFace cache.
Avoids the datasets library which crashes with torchcodec DLL on Windows.
Converts MP3 to Opus (16k mono) via ffmpeg pipe — no intermediate files.

Usage:
    scripts/.venv/Scripts/python.exe scripts/extract_audio.py
    scripts/.venv/Scripts/python.exe scripts/extract_audio.py --cache-dir D:\hf_cache
    scripts/.venv/Scripts/python.exe scripts/extract_audio.py --count 100
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path

import pyarrow
import pyarrow.ipc as ipc
from tqdm import tqdm

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent

DEFAULT_PHRASES = PROJECT_ROOT / "phrase_dataset" / "phrase_dataset.json"
DEFAULT_OUTPUT = PROJECT_ROOT / "phrase_dataset" / "audio"
INDEX_FILE = SCRIPT_DIR / "transcription_index.json"

PROGRESS_INTERVAL = 1000

_CROCKFORD_BASE32 = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"


def encode_ulid(raw: bytes) -> str:
    if len(raw) != 16:
        raise ValueError(f"Expected 16 bytes, got {len(raw)}")
    value = int.from_bytes(raw, byteorder="big")
    chars: list[str] = []
    for _ in range(26):
        chars.append(_CROCKFORD_BASE32[value & 0x1F])
        value >>= 5
    return "".join(reversed(chars))


def make_ulid(numeric_id: int) -> str:
    hash_bytes = hashlib.sha256(str(numeric_id).encode()).digest()[:10]
    raw = b"\x00" * 6 + hash_bytes
    return encode_ulid(raw)


def default_cache_dir() -> Path:
    return Path.home() / ".cache" / "huggingface" / "datasets"


def find_arrow_files(cache_dir: Path) -> list[Path]:
    if not cache_dir.exists():
        return []
    results: list[Path] = []
    for arrow_path in cache_dir.rglob("*.arrow"):
        path_str = arrow_path.as_posix().lower()
        if "joujiboi" in path_str or "japanese-anime-speech" in path_str:
            results.append(arrow_path)
    return sorted(results)


def load_index() -> dict[str, str]:
    if not INDEX_FILE.exists():
        return {}
    with INDEX_FILE.open("r", encoding="utf-8") as f:
        return json.load(f)


def save_index(index: dict[str, str]) -> None:
    with INDEX_FILE.open("w", encoding="utf-8") as f:
        json.dump(index, f, ensure_ascii=False, indent=2)


def read_arrow_table(arrow_path: Path) -> pyarrow.Table:
    with arrow_path.open("rb") as f:
        reader = ipc.open_stream(f)
        return reader.read_all()


def build_index_from_arrow(
    arrow_files: list[Path],
    existing_index: dict[str, str],
) -> dict[str, str]:
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


def convert_mp3_to_opus(mp3_bytes: bytes) -> bytes:
    result = subprocess.run(
        [
            "ffmpeg", "-i", "pipe:0",
            "-c:a", "libopus", "-b:a", "16k", "-ac", "1",
            "-vn", "-f", "opus", "pipe:1",
        ],
        input=mp3_bytes,
        capture_output=True,
        timeout=30,
    )
    if result.returncode != 0:
        return b""
    return result.stdout


def load_phrases(path: Path) -> list[dict]:
    if not path.exists():
        print(f"ERROR: Phrases file not found: {path}", file=sys.stderr)
        sys.exit(1)

    with path.open("r", encoding="utf-8") as f:
        data = json.load(f)

    if isinstance(data, dict):
        phrases = data.get("phrases")
        if phrases is None:
            print("ERROR: JSON must have 'phrases' key or be an array", file=sys.stderr)
            sys.exit(1)
        return phrases

    if isinstance(data, list):
        return data

    print("ERROR: Expected JSON array or object with 'phrases' key", file=sys.stderr)
    sys.exit(1)


def find_audio_for_phrase(
    phrase: dict,
    index: dict[str, str],
) -> tuple[bytes | None, str]:
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
        description="Extract audio from HF dataset cache, convert to Opus"
    )
    parser.add_argument(
        "--phrases",
        type=Path,
        default=DEFAULT_PHRASES,
        help="Path to phrase_dataset.json",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_OUTPUT,
        help="Output directory for .opus files",
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
    parser.add_argument(
        "--count",
        type=int,
        default=None,
        help="Limit number of phrases to process (for testing)",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    phrases = load_phrases(args.phrases)
    if args.count is not None:
        phrases = phrases[: args.count]
    total = len(phrases)
    print(f"Loaded {total} phrases from {args.phrases}")

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

    counts: dict[str, int] = {"found": 0, "skipped": 0, "missing": 0, "errors": 0}
    t_start = time.monotonic()

    pbar = tqdm(phrases, desc="Extracting", unit="file",
                bar_format="{l_bar}{bar}| {n_fmt}/{total_fmt} [{elapsed}<{remaining}, {rate_fmt}] {postfix}")

    for i, phrase in enumerate(phrases, 1):
        numeric_id = phrase.get("id")
        ulid = make_ulid(numeric_id)
        output_file = output_dir / f"{ulid}.opus"

        if output_file.exists() and output_file.stat().st_size > 0:
            counts["skipped"] += 1
            pbar.set_postfix_str(f"f={counts['found']} s={counts['skipped']} m={counts['missing']}")
            pbar.update(1)
            continue

        audio, status = find_audio_for_phrase(phrase, index)

        if audio is None:
            counts["missing"] += 1
            pbar.set_postfix_str(f"f={counts['found']} s={counts['skipped']} m={counts['missing']}")
            pbar.update(1)
            continue

        opus = convert_mp3_to_opus(audio)
        if not opus:
            counts["errors"] += 1
            pbar.write(f"FFMPEG_ERROR id={numeric_id} ulid={ulid}")
            pbar.update(1)
            continue

        output_file.write_bytes(opus)
        counts["found"] += 1
        pbar.set_postfix_str(f"f={counts['found']} s={counts['skipped']} m={counts['missing']}")
        pbar.update(1)

    pbar.close()
    elapsed = time.monotonic() - t_start
    _print_summary(total, counts, elapsed, output_dir)


def _print_progress(
    i: int,
    total: int,
    counts: dict[str, int],
    t_start: float,
    missing_id: int | None = None,
) -> None:
    elapsed = time.monotonic() - t_start
    suffix = f" (last missing: id={missing_id})" if missing_id else ""
    print(
        f"  [{i}/{total}] "
        f"f={counts['found']} s={counts['skipped']} "
        f"m={counts['missing']} e={counts['errors']} "
        f"({elapsed:.1f}s){suffix}"
    )


def _print_summary(
    total: int,
    counts: dict[str, int],
    elapsed: float,
    output_dir: Path,
) -> None:
    print()
    print("=" * 60)
    print(f"  Total:   {total}")
    print(f"  Found:   {counts['found']}")
    print(f"  Skipped: {counts['skipped']}")
    print(f"  Missing: {counts['missing']}")
    print(f"  Errors:  {counts['errors']}")
    print(f"  Time:    {elapsed:.1f}s")
    print(f"  Output:  {output_dir}")
    print("=" * 60)


if __name__ == "__main__":
    main()
