#!/usr/bin/env python3
"""Rebuild pitch audio index v3 with composite keys.

Reads entries.db (nhk16 + daijisen), combines with JmdictFurigana readings,
converts mp3->opus via ffmpeg, produces cdn/pitch/index.json v3 + audio/*.opus.

Persistent hash cache at tmp/pitch_sources/hash_cache.json allows resume.

Usage:
    python scripts/rebuild_pitch_index.py
"""

import hashlib
import json
import re
import shutil
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(r"D:\origa_worktree\origa")

DB_PATH = ROOT / "tmp" / "pitch_sources" / "entries.db"
FURIGANA_PATH = ROOT / "cdn" / "dictionaries" / "JmdictFurigana.txt"
INDEX_PATH = ROOT / "cdn" / "pitch" / "index.json"
AUDIO_DIR = ROOT / "cdn" / "pitch" / "audio"
HASH_CACHE_PATH = ROOT / "tmp" / "pitch_sources" / "hash_cache.json"

NHK_MEDIA = ROOT / "tmp" / "nhk_extract" / "nhk16" / "media"
DAIJISEN_MEDIA = ROOT / "tmp" / "daijisen_extract" / "daijisen" / "media"

SOURCE_PRIORITY: dict[str, int] = {"nhk16": 0, "daijisen": 1}
VALID_SOURCES = {"nhk16", "daijisen"}

PITCH_RE = re.compile(r"\[(\d+)\]")
HASH_LEN = 16
PROGRESS_INTERVAL = 1000
CACHE_SAVE_INTERVAL = 5000

MEDIA_DIRS: dict[str, Path] = {
    "nhk16": NHK_MEDIA,
    "daijisen": DAIJISEN_MEDIA,
}

TEST_WORDS = ["役", "空く", "本", "東", "前", "貴方", "風", "一寸", "開く", "下", "間"]


def _has_kanji(text: str) -> bool:
    for ch in text:
        cp = ord(ch)
        if 0x4E00 <= cp <= 0x9FFF or 0x3400 <= cp <= 0x4DBF or 0xF900 <= cp <= 0xFAFF:
            return True
    return False


def _extract_pitch(display: str | None) -> int | None:
    if not display:
        return None
    m = PITCH_RE.search(display)
    return int(m.group(1)) if m else None


def check_prerequisites() -> None:
    print("Checking prerequisites...")
    try:
        subprocess.run(["ffmpeg", "-version"], capture_output=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("ERROR: ffmpeg not found.", file=sys.stderr)
        sys.exit(1)

    for path in [DB_PATH, FURIGANA_PATH]:
        if not path.exists():
            print(f"ERROR: {path} not found", file=sys.stderr)
            sys.exit(1)

    for source, media_dir in MEDIA_DIRS.items():
        if not media_dir.exists():
            print(f"ERROR: {media_dir} not found (source: {source})", file=sys.stderr)
            sys.exit(1)

    print("Prerequisites OK.")


def parse_furigana() -> dict[str, list[str]]:
    """Load JmdictFurigana.txt -> {word: [reading1, reading2, ...]} (ordered)."""
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


def load_db_lookups() -> tuple[
    dict[tuple[str, str], tuple[str, str, int | None]],
    dict[str, tuple[str, str, int | None]],
]:
    """Load entries.db -> er_lookup and kana_lookup."""
    print("Loading entries from database...")
    conn = sqlite3.connect(str(DB_PATH))

    ph = ",".join("?" for _ in VALID_SOURCES)
    order_case = (
        "CASE source "
        "WHEN 'nhk16' THEN 0 WHEN 'daijisen' THEN 1 "
        "WHEN 'shinmeikai8' THEN 2 WHEN 'jpod' THEN 3 END"
    )

    er_lookup: dict[tuple[str, str], tuple[str, str, int | None]] = {}
    cur = conn.execute(
        f"SELECT expression, reading, source, file, display "
        f"FROM entries "
        f"WHERE source IN ({ph}) AND reading IS NOT NULL "
        f"ORDER BY {order_case}, id",
        list(VALID_SOURCES),
    )
    total = 0
    for expr, reading, src, fp, display in cur:
        total += 1
        key = (expr, reading)
        if key not in er_lookup:
            er_lookup[key] = (src, fp, _extract_pitch(display))
    print(f"  {total} rows -> {len(er_lookup)} unique (expression, reading) pairs")

    kana_lookup: dict[str, tuple[str, str, int | None]] = {}
    cur2 = conn.execute(
        f"SELECT expression, source, file, display "
        f"FROM entries "
        f"WHERE expression = reading AND source IN ({ph}) "
        f"ORDER BY {order_case}, id",
        list(VALID_SOURCES),
    )
    for word, src, fp, display in cur2:
        if _has_kanji(word):
            continue
        if word not in kana_lookup:
            kana_lookup[word] = (src, fp, _extract_pitch(display))
    print(f"  {len(kana_lookup)} kana-only entries")

    conn.close()
    return er_lookup, kana_lookup


def build_old_index_mapping(
    furigana: dict[str, list[str]],
    er_lookup: dict[tuple[str, str], tuple[str, str, int | None]],
    kana_lookup: dict[str, tuple[str, str, int | None]],
) -> dict[str, tuple[str, str, int | None]]:
    """Replicate old script logic to produce word -> (source, mp3_file, pitch)."""
    print("  Replicating old index mapping for cache...")

    def upsert(
        store: dict[str, tuple[str, str, int | None]],
        word: str,
        source: str,
        fp: str,
        pitch: int | None,
    ) -> None:
        if word not in store:
            store[word] = (source, fp, pitch)
        elif SOURCE_PRIORITY.get(source, 99) < SOURCE_PRIORITY.get(store[word][0], 99):
            store[word] = (source, fp, pitch)

    result: dict[str, tuple[str, str, int | None]] = {}

    for word, readings in furigana.items():
        if not _has_kanji(word):
            continue
        kanji_set = False
        for reading in readings:
            key = (word, reading)
            if key in er_lookup:
                src, fp, pitch = er_lookup[key]
                if not kanji_set:
                    upsert(result, word, src, fp, pitch)
                    kanji_set = True
                upsert(result, reading, src, fp, pitch)

    for word, (src, fp, pitch) in kana_lookup.items():
        if word not in result:
            result[word] = (src, fp, pitch)

    for word, readings in furigana.items():
        for reading in readings:
            if reading not in result:
                key = (reading, reading)
                if key in er_lookup:
                    src, fp, pitch = er_lookup[key]
                    result[reading] = (src, fp, pitch)

    print(f"  Old mapping: {len(result)} entries")
    return result


def build_mp3_opus_cache(
    current_entries: dict[str, dict],
    old_mapping: dict[str, tuple[str, str, int | None]],
) -> dict[tuple[str, str], str]:
    """Build (source, mp3_file) -> opus_hash from current index."""
    print("Building mp3->opus cache from current index...")
    cache: dict[tuple[str, str], str] = {}
    matched = 0

    for word, (source, mp3_file, pitch) in old_mapping.items():
        if word in current_entries:
            entry = current_entries[word]
            if entry.get("p") == pitch:
                opus_hash = entry["f"].replace(".opus", "")
                cache[(source, mp3_file)] = opus_hash
                matched += 1

    print(f"  Cache: {len(cache)} unique (source, mp3_file) -> opus mappings, "
          f"{matched} word matches")
    return cache


def load_hash_cache() -> dict[str, str]:
    """Load persistent hash cache from disk. Key: 'source|mp3_file', value: opus_hash."""
    if not HASH_CACHE_PATH.exists():
        return {}
    with open(HASH_CACHE_PATH, encoding="utf-8") as fh:
        data = json.load(fh)
    print(f"  Loaded persistent cache: {len(data)} entries")
    return data


def save_hash_cache(cache: dict[str, str]) -> None:
    """Save persistent hash cache to disk."""
    HASH_CACHE_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(HASH_CACHE_PATH, "w", encoding="utf-8") as fh:
        json.dump(cache, fh, ensure_ascii=False, separators=(",", ":"))


def resolve_mp3_path(source: str, db_file: str) -> Path | None:
    """Convert DB file reference to actual filesystem path."""
    media_dir = MEDIA_DIRS.get(source)
    if not media_dir:
        return None
    filename = db_file.removeprefix("media/")
    full_path = media_dir / filename
    return full_path if full_path.exists() else None


def convert_mp3_to_opus(mp3_path: Path) -> tuple[str | None, str]:
    """Convert a single mp3 file to opus. Returns (opus_hash, error_message)."""
    tmp_opus = Path(tempfile.mktemp(suffix=".opus"))
    try:
        cmd = [
            "ffmpeg", "-loglevel", "error",
            "-i", str(mp3_path),
            "-c:a", "libopus", "-b:a", "32k",
            "-vn", "-y", str(tmp_opus),
        ]
        proc = subprocess.run(cmd, capture_output=True)
        if proc.returncode != 0:
            stderr = proc.stderr.decode("utf-8", errors="replace")[:300]
            return None, f"ffmpeg failed for {mp3_path.name}: {stderr}"

        opus_data = tmp_opus.read_bytes()
        sha = hashlib.sha256(opus_data).hexdigest()[:HASH_LEN]
        final_path = AUDIO_DIR / f"{sha}.opus"

        if not final_path.exists():
            final_path.write_bytes(opus_data)

        return sha, ""
    except Exception as exc:
        return None, str(exc)
    finally:
        tmp_opus.unlink(missing_ok=True)


def process_all_audio(
    er_lookup: dict[tuple[str, str], tuple[str, str, int | None]],
    kana_lookup: dict[str, tuple[str, str, int | None]],
    cache: dict[tuple[str, str], str],
) -> dict[tuple[str, str], str]:
    """Convert all needed mp3->opus, using cache where possible."""
    print("Processing audio files...")
    AUDIO_DIR.mkdir(parents=True, exist_ok=True)

    all_pairs: set[tuple[str, str]] = set()
    for src, fp, _ in er_lookup.values():
        all_pairs.add((src, fp))
    for src, fp, _ in kana_lookup.values():
        all_pairs.add((src, fp))

    # Load persistent cache and merge with in-memory cache
    persistent = load_hash_cache()
    file_map: dict[tuple[str, str], str] = {}
    for key_str, opus_hash in persistent.items():
        parts = key_str.split("|", 1)
        if len(parts) == 2:
            file_map[(parts[0], parts[1])] = opus_hash

    # Merge in-memory cache (from current index) — may have newer data
    for pair, opus_hash in cache.items():
        file_map[pair] = opus_hash

    print(f"  Total unique (source, mp3_file) pairs: {len(all_pairs)}")
    cached_count = sum(1 for p in all_pairs if p in file_map)
    print(f"  Cache hits (persistent + index): {cached_count}")

    to_convert = [p for p in all_pairs if p not in file_map]
    print(f"  Need to convert: {len(to_convert)}")

    if not to_convert:
        print("  All audio already cached!")
        return file_map

    failed = 0
    converted = 0
    for idx, (source, mp3_file) in enumerate(to_convert):
        mp3_path = resolve_mp3_path(source, mp3_file)
        if mp3_path is None:
            failed += 1
            if failed <= 20:
                print(f"  WARNING: mp3 not found: {source}/{mp3_file}")
            continue

        opus_hash, err = convert_mp3_to_opus(mp3_path)
        if err:
            failed += 1
            if failed <= 20:
                print(f"  WARNING: {err}")
        elif opus_hash:
            file_map[(source, mp3_file)] = opus_hash
            converted += 1

        # Periodic cache save for resume capability
        if (idx + 1) % CACHE_SAVE_INTERVAL == 0:
            print(f"  Progress: {idx + 1}/{len(to_convert)} "
                  f"(converted: {converted}, failed: {failed})")
            persistent_out = {
                f"{s}|{f}": h for (s, f), h in file_map.items()
            }
            save_hash_cache(persistent_out)

    # Final save
    persistent_out = {f"{s}|{f}": h for (s, f), h in file_map.items()}
    save_hash_cache(persistent_out)

    print(f"  Audio done: {len(file_map)} total mappings, "
          f"{converted} new conversions, {failed} failures")
    return file_map


def build_v3_entries(
    er_lookup: dict[tuple[str, str], tuple[str, str, int | None]],
    kana_lookup: dict[str, tuple[str, str, int | None]],
    furigana: dict[str, list[str]],
    file_map: dict[tuple[str, str], str],
) -> dict[str, dict[str, str | int]]:
    """Build v3 index entries with composite, kana, and kanji keys."""
    print("Building v3 index entries...")
    entries: dict[str, dict[str, str | int]] = {}
    no_audio = 0

    def add_entry(
        index_key: str,
        source: str,
        mp3_file: str,
        pitch: int | None,
    ) -> None:
        nonlocal no_audio
        pair = (source, mp3_file)
        if pair not in file_map:
            no_audio += 1
            return
        opus_hash = file_map[pair]
        entries[index_key] = {
            "f": f"{opus_hash}.opus",
            "p": pitch if pitch is not None else 0,
        }

    # Pass 1: Composite keys "expression|reading" for ALL (expression, reading) pairs
    for (expr, reading), (source, mp3_file, pitch) in er_lookup.items():
        if expr != reading:
            add_entry(f"{expr}|{reading}", source, mp3_file, pitch)

    # Pass 2: Kana keys "reading" — first expression (alphabetically) for each reading
    reading_best_expr: dict[str, str] = {}
    reading_best_data: dict[str, tuple[str, str, int | None]] = {}
    for (expr, reading), (source, mp3_file, pitch) in er_lookup.items():
        if reading not in reading_best_expr or expr < reading_best_expr[reading]:
            reading_best_expr[reading] = expr
            reading_best_data[reading] = (source, mp3_file, pitch)

    for reading, (source, mp3_file, pitch) in reading_best_data.items():
        add_entry(reading, source, mp3_file, pitch)

    # Kana-only entries (expression == reading, no kanji in word)
    for word, (source, mp3_file, pitch) in kana_lookup.items():
        if word not in entries:
            add_entry(word, source, mp3_file, pitch)

    # Pass 3: Kanji keys "expression" — primary reading from furigana
    for word, readings in furigana.items():
        if not _has_kanji(word):
            continue
        for reading in readings:
            pair = (word, reading)
            if pair in er_lookup:
                source, mp3_file, pitch = er_lookup[pair]
                add_entry(word, source, mp3_file, pitch)
                break

    # Pass 4: Kana readings from furigana not yet in the index
    for word, readings in furigana.items():
        for reading in readings:
            if reading not in entries:
                pair = (reading, reading)
                if pair in er_lookup:
                    src, fp, pitch = er_lookup[pair]
                    add_entry(reading, src, fp, pitch)

    print(f"  Entries: {len(entries)}, skipped (no audio): {no_audio}")
    return entries


def verify_index(
    entries: dict[str, dict[str, str | int]],
    furigana: dict[str, list[str]],
    er_lookup: dict[tuple[str, str], tuple[str, str, int | None]],
) -> bool:
    """Verify test words have correct entries."""
    print("Verifying test words...")
    all_ok = True

    for word in TEST_WORDS:
        print(f"\n  {word}:")
        kanji_entry = entries.get(word)
        if kanji_entry:
            print(f"    kanji '{word}': f={kanji_entry['f']}, p={kanji_entry['p']}")
        else:
            print(f"    kanji '{word}': MISSING")
            all_ok = False

        if word in furigana:
            for reading in furigana[word]:
                composite = f"{word}|{reading}"
                comp_entry = entries.get(composite)
                kana_entry = entries.get(reading)

                pair = (word, reading)
                if pair in er_lookup:
                    src, fp, pitch = er_lookup[pair]
                    if comp_entry:
                        status = "OK" if comp_entry["p"] == (pitch if pitch is not None else 0) else "MISMATCH"
                        print(f"    composite '{composite}': "
                              f"f={comp_entry['f']}, p={comp_entry['p']} "
                              f"(expected p={pitch}) [{status}]")
                        if status == "MISMATCH":
                            all_ok = False
                    else:
                        print(f"    composite '{composite}': MISSING")
                        all_ok = False

                if kana_entry:
                    print(f"    kana '{reading}': f={kana_entry['f']}, p={kana_entry['p']}")
                else:
                    print(f"    kana '{reading}': MISSING (may be expected)")

        if word in furigana and kanji_entry:
            for reading in furigana[word]:
                pair = (word, reading)
                if pair in er_lookup:
                    comp_entry = entries.get(f"{word}|{reading}")
                    if comp_entry and comp_entry["f"] == kanji_entry["f"]:
                        print(f"    kanji '{word}' uses reading '{reading}' (primary)")
                        break

    return all_ok


def deploy(entries: dict[str, dict[str, str | int]]) -> None:
    """Write index.json v3 with backup."""
    print("Deploying index.json v3...")

    if INDEX_PATH.exists():
        backup = INDEX_PATH.with_suffix(".json.bak")
        print(f"  Backup: {INDEX_PATH.name} -> {backup.name}")
        shutil.copy2(INDEX_PATH, backup)

    sorted_entries = dict(sorted(entries.items()))
    INDEX_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(INDEX_PATH, "w", encoding="utf-8") as fh:
        json.dump(
            {"v": 3, "total": len(sorted_entries), "entries": sorted_entries},
            fh,
            ensure_ascii=False,
            separators=(",", ":"),
        )
    print(f"  Written {INDEX_PATH} ({len(sorted_entries)} entries)")


def main() -> None:
    print("=" * 60)
    print("Origa Pitch Audio Index Rebuilder (v3 with composite keys)")
    print("=" * 60)

    try:
        check_prerequisites()
        furigana = parse_furigana()
        er_lookup, kana_lookup = load_db_lookups()

        # Build cache from current index
        print("\nLoading current index for cache...")
        current_index: dict[str, dict] = {}
        if INDEX_PATH.exists():
            with open(INDEX_PATH, encoding="utf-8") as fh:
                current_index = json.load(fh).get("entries", {})
            print(f"  Current index: {len(current_index)} entries")

        old_mapping = build_old_index_mapping(furigana, er_lookup, kana_lookup)
        cache = build_mp3_opus_cache(current_index, old_mapping)

        # Process audio (with persistent cache for resume)
        file_map = process_all_audio(er_lookup, kana_lookup, cache)

        # Build v3 index
        entries = build_v3_entries(er_lookup, kana_lookup, furigana, file_map)

        # Verify
        all_ok = verify_index(entries, furigana, er_lookup)

        # Deploy
        deploy(entries)

        # Summary
        print("\n" + "=" * 50)
        print("SUMMARY")
        print("=" * 50)
        print(f"  Total entries:  {len(entries)}")
        print(f"  Cache hits:     {len(cache)}")
        print(f"  Verification:   {'PASS' if all_ok else 'ISSUES FOUND'}")
        print("=" * 50)
        print(f"\nDONE. {len(entries)} entries -> {INDEX_PATH}")

        if not all_ok:
            print("\nWARNING: Some verification checks failed. See above for details.")

    except KeyboardInterrupt:
        print("\nInterrupted by user. Cache saved for resume.")
        sys.exit(130)


if __name__ == "__main__":
    main()
