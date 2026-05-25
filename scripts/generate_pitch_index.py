#!/usr/bin/env python3
"""Regenerate pitch audio index from authoritative dictionary sources.

Reads entries.db + zip archives, combines with JmdictFurigana readings,
converts mp3→opus, produces cdn/pitch/index.json + cdn/pitch/audio/*.opus.

Usage:
    python scripts/generate_pitch_index.py
"""

import hashlib
import json
import re
import shutil
import sqlite3
import subprocess
import sys
import zipfile
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

ROOT = Path(r"D:\origa_worktree\origa")

SOURCES_DIR = ROOT / "tmp" / "pitch_sources"
DB_PATH = SOURCES_DIR / "entries.db"
FURIGANA_PATH = ROOT / "cdn" / "dictionaries" / "JmdictFurigana.txt"

OUTPUT_DIR = ROOT / "cdn" / "pitch"
AUDIO_DIR = OUTPUT_DIR / "audio"
INDEX_PATH = OUTPUT_DIR / "index.json"

TEMP_DIR = ROOT / "tmp" / "pitch_convert"

PRIORITY: dict[str, int] = {
    "nhk16": 0,
    "daijisen": 1,
    "shinmeikai8": 2,
    "jpod": 3,
}
ZIP_NAMES: dict[str, str] = {
    "nhk16": "nhk16.zip",
    "daijisen": "daijisen.zip",
    "shinmeikai8": "shinmeikai8.zip",
    "jpod": "jpod.zip",
}
SOURCES = list(PRIORITY.keys())

PITCH_RE = re.compile(r"\[(\d+)\]")
OPUS_BITRATE = "32k"
HASH_LEN = 16
PROGRESS_INTERVAL = 2000
MAX_WORKERS = 8


def check_prerequisites() -> None:
    print("Checking prerequisites...")
    try:
        subprocess.run(["ffmpeg", "-version"], capture_output=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("ERROR: ffmpeg not found.", file=sys.stderr)
        sys.exit(1)

    if not DB_PATH.exists():
        print(f"ERROR: {DB_PATH} not found", file=sys.stderr)
        sys.exit(1)
    if not FURIGANA_PATH.exists():
        print(f"ERROR: {FURIGANA_PATH} not found", file=sys.stderr)
        sys.exit(1)

    for name in ZIP_NAMES.values():
        if not (SOURCES_DIR / name).exists():
            print(f"WARNING: {name} not found, entries will be skipped")

    print("Prerequisites OK.")


def parse_furigana() -> dict[str, list[str]]:
    print("Parsing furigana dictionary...")
    result: dict[str, list[str]] = {}
    with open(FURIGANA_PATH, encoding="utf-8-sig") as fh:
        for line in fh:
            parts = line.strip().split("|")
            if len(parts) < 2:
                continue
            word, reading = parts[0], parts[1]
            readings = result.setdefault(word, [])
            if reading not in readings:
                readings.append(reading)
    print(f"  {len(result)} words loaded")
    return result


def extract_pitch(display: str | None) -> int | None:
    if not display:
        return None
    m = PITCH_RE.search(display)
    return int(m.group(1)) if m else None


def _has_kanji(text: str) -> bool:
    for ch in text:
        cp = ord(ch)
        if 0x4E00 <= cp <= 0x9FFF or 0x3400 <= cp <= 0x4DBF or 0xF900 <= cp <= 0xFAFF:
            return True
    return False


_ZIP_CACHE: dict[str, bool] = {}

def _zip_exists(source: str) -> bool:
    if source not in _ZIP_CACHE:
        _ZIP_CACHE[source] = (SOURCES_DIR / ZIP_NAMES[source]).exists()
    return _ZIP_CACHE[source]


def build_index_mapping(
    furigana: dict[str, list[str]],
) -> tuple[dict[str, tuple[str, str, int | None]], dict[str, int]]:
    """Build {word: (source, file_in_zip, pitch)}.

    KEY OPTIMIZATION: Load ALL entries from DB in two bulk queries,
    build in-memory lookup, then match against furigana.
    """
    print("Loading entries from database (bulk)...")
    conn = sqlite3.connect(str(DB_PATH))
    # No row_factory — plain tuples are 5-10x faster for bulk iteration

    ph = ",".join("?" for _ in SOURCES)

    # BULK QUERY 1: all entries with readings, ordered by source priority
    print("  Loading expression+reading entries...")
    er_lookup: dict[tuple[str, str], tuple[str, str, int | None]] = {}
    cur = conn.execute(
        f"SELECT expression, reading, source, file, display "
        f"FROM entries "
        f"WHERE source IN ({ph}) AND reading IS NOT NULL "
        f"ORDER BY CASE source "
        f"WHEN 'nhk16' THEN 0 WHEN 'daijisen' THEN 1 "
        f"WHEN 'shinmeikai8' THEN 2 WHEN 'jpod' THEN 3 END, id",
        SOURCES,
    )
    total = 0
    for expr, reading, src, fp, display in cur:
        total += 1
        key = (expr, reading)
        if key not in er_lookup and _zip_exists(src):
            er_lookup[key] = (src, fp, extract_pitch(display))
    print(f"  Loaded {total} rows -> {len(er_lookup)} unique (expression, reading) pairs")

    # BULK QUERY 2: kana-only entries (expression = reading)
    print("  Loading kana-only entries...")
    kana_lookup: dict[str, tuple[str, str, int | None]] = {}
    cur2 = conn.execute(
        f"SELECT expression, source, file, display "
        f"FROM entries "
        f"WHERE expression = reading AND source IN ({ph}) "
        f"ORDER BY CASE source "
        f"WHEN 'nhk16' THEN 0 WHEN 'daijisen' THEN 1 "
        f"WHEN 'shinmeikai8' THEN 2 WHEN 'jpod' THEN 3 END, id",
        SOURCES,
    )
    for word, src, fp, display in cur2:
        if _has_kanji(word):
            continue
        if word not in kana_lookup and _zip_exists(src):
            kana_lookup[word] = (src, fp, extract_pitch(display))
    print(f"  {len(kana_lookup)} kana-only entries")

    conn.close()

    # Build final index using in-memory lookups
    print("  Matching furigana -> audio...")
    result: dict[str, tuple[str, str, int | None]] = {}
    source_counts: dict[str, int] = {s: 0 for s in SOURCES}

    def upsert(word: str, source: str, fp: str, pitch: int | None) -> None:
        if word not in result:
            result[word] = (source, fp, pitch)
            source_counts[source] += 1
        elif PRIORITY[source] < PRIORITY[result[word][0]]:
            source_counts[result[word][0]] -= 1
            result[word] = (source, fp, pitch)
            source_counts[source] += 1

    # Kanji words from furigana
    for word, readings in furigana.items():
        if not _has_kanji(word):
            continue
        kanji_set = False
        for reading in readings:
            key = (word, reading)
            if key in er_lookup:
                src, fp, pitch = er_lookup[key]
                if not kanji_set:
                    upsert(word, src, fp, pitch)
                    kanji_set = True
                upsert(reading, src, fp, pitch)

    # Kana-only words
    for word, (src, fp, pitch) in kana_lookup.items():
        if word not in result:
            result[word] = (src, fp, pitch)
            source_counts[src] += 1

    # Also add readings from furigana that aren't yet in the index
    for word, readings in furigana.items():
        for reading in readings:
            if reading not in result:
                key = (reading, reading)
                if key in er_lookup:
                    src, fp, pitch = er_lookup[key]
                    result[reading] = (src, fp, pitch)
                    source_counts[src] += 1

    print(f"  Total entries: {len(result)}")
    return result, source_counts


def _convert_one(
    source: str,
    file_in_zip: str,
    work_id: int,
    tmp_dir: Path,
    target_dir: Path,
) -> tuple[str, str]:
    zip_path = SOURCES_DIR / ZIP_NAMES[source]
    mp3_tmp = tmp_dir / f"{work_id}_in.mp3"
    opus_tmp = tmp_dir / f"{work_id}_out.opus"

    try:
        with zipfile.ZipFile(zip_path, "r") as zf:
            mp3_data = zf.read(file_in_zip)
        mp3_tmp.write_bytes(mp3_data)

        proc = subprocess.run(
            [
                "ffmpeg", "-loglevel", "error",
                "-i", str(mp3_tmp),
                "-c:a", "libopus", "-b:a", OPUS_BITRATE,
                "-vn", "-y", str(opus_tmp),
            ],
            capture_output=True,
        )
        if proc.returncode != 0:
            stderr = proc.stderr.decode("utf-8", errors="replace")[:300]
            return ("", f"ffmpeg failed for {source}/{file_in_zip}: {stderr}")

        opus_data = opus_tmp.read_bytes()
        sha = hashlib.sha256(opus_data).hexdigest()[:HASH_LEN]
        opus_name = f"{sha}.opus"
        final_path = target_dir / opus_name

        if not final_path.exists():
            final_path.write_bytes(opus_data)

        return (opus_name, "")
    finally:
        mp3_tmp.unlink(missing_ok=True)
        opus_tmp.unlink(missing_ok=True)


def convert_all_audio(
    index_map: dict[str, tuple[str, str, int | None]],
    target_dir: Path,
) -> dict[tuple[str, str], str]:
    print("Converting audio files...")
    unique_pairs: set[tuple[str, str]] = {
        (s, f) for s, f, _ in index_map.values()
    }
    available = [(s, f) for s, f in unique_pairs if _zip_exists(s)]
    print(f"  Unique: {len(unique_pairs)}, available: {len(available)}")

    target_dir.mkdir(parents=True, exist_ok=True)
    TEMP_DIR.mkdir(parents=True, exist_ok=True)
    file_map: dict[tuple[str, str], str] = {}
    failed = 0

    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as executor:
        futures = {}
        for idx, (source, fp) in enumerate(available):
            future = executor.submit(
                _convert_one, source, fp, idx, TEMP_DIR, target_dir,
            )
            futures[future] = (source, fp)

        done = 0
        for future in as_completed(futures):
            source, fp = futures[future]
            done += 1
            try:
                opus_name, err = future.result()
                if err:
                    failed += 1
                    if failed <= 20:
                        print(f"  WARNING: {err}")
                else:
                    file_map[(source, fp)] = opus_name
            except Exception as exc:
                failed += 1
                if failed <= 20:
                    print(f"  WARNING: {source}/{fp}: {exc}")

            if done % PROGRESS_INTERVAL == 0:
                print(f"  Progress: {done}/{len(available)}")

    print(f"  Converted: {len(file_map)}, failed: {failed}")
    return file_map


def build_entries(
    index_map: dict[str, tuple[str, str, int | None]],
    file_map: dict[tuple[str, str], str],
) -> dict[str, dict[str, str | int | None]]:
    entries: dict[str, dict[str, str | int | None]] = {}
    skipped = 0
    for word, (source, fp, pitch) in index_map.items():
        if (source, fp) not in file_map:
            skipped += 1
            continue
        entries[word] = {"f": file_map[(source, fp)], "p": pitch}
    if skipped:
        print(f"  Skipped {skipped} entries with missing audio")
    return entries


def deploy(
    entries: dict[str, dict[str, str | int | None]],
    new_audio_dir: Path,
) -> None:
    print("Deploying...")
    if INDEX_PATH.exists():
        backup = INDEX_PATH.with_suffix(".json.bak")
        print(f"  Backup: {INDEX_PATH.name} -> {backup.name}")
        shutil.copy2(INDEX_PATH, backup)

    sorted_entries = dict(sorted(entries.items()))
    INDEX_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(INDEX_PATH, "w", encoding="utf-8") as fh:
        json.dump(
            {"v": 2, "total": len(sorted_entries), "entries": sorted_entries},
            fh,
            ensure_ascii=False,
            separators=(",", ":"),
        )
    print(f"  Written {INDEX_PATH} ({len(sorted_entries)} entries)")

    if AUDIO_DIR.exists():
        old_audio = AUDIO_DIR.parent / "audio_old"
        if old_audio.exists():
            shutil.rmtree(old_audio)
        AUDIO_DIR.rename(old_audio)
    new_audio_dir.rename(AUDIO_DIR)

    old_audio = AUDIO_DIR.parent / "audio_old"
    if old_audio.exists():
        shutil.rmtree(old_audio)

    print("  Deploy complete.")


def print_summary(
    total_entries: int,
    total_audio: int,
    source_counts: dict[str, int],
) -> None:
    print()
    print("=" * 50)
    print("SUMMARY")
    print("=" * 50)
    print(f"  Total entries:  {total_entries}")
    print(f"  Audio files:    {total_audio}")
    print("  By source:")
    for source in SOURCES:
        print(f"    {source:15s} {source_counts.get(source, 0):>6d}")
    print("=" * 50)


def main() -> None:
    print("=" * 60)
    print("Origa Pitch Audio Index Generator")
    print("=" * 60)
    try:
        check_prerequisites()
        furigana = parse_furigana()
        index_map, source_counts = build_index_mapping(furigana)

        temp_audio = TEMP_DIR / "audio"
        if temp_audio.exists():
            shutil.rmtree(temp_audio)

        file_map = convert_all_audio(index_map, temp_audio)
        entries = build_entries(index_map, file_map)
        deploy(entries, temp_audio)

        print_summary(len(entries), len(file_map), source_counts)
        print(f"\nDONE. {len(entries)} entries -> {INDEX_PATH}")
    except KeyboardInterrupt:
        print("\nInterrupted by user.")
        sys.exit(130)
    finally:
        if TEMP_DIR.exists():
            shutil.rmtree(TEMP_DIR, ignore_errors=True)


if __name__ == "__main__":
    main()
