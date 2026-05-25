#!/usr/bin/env python3
"""Re-patch pitch index using NHK's own ordering (authoritative).

v1 used furigana reading order (wrong — not frequency-based).
v2 uses NHK entry order (authoritative dictionary ordering).
Parallel mp3->opus conversion using ThreadPoolExecutor + ffmpeg pipe output.

Usage: python scripts/patch_nhk_pitch_v2.py
"""

import hashlib
import json
import os
import re
import shutil
import sqlite3
import subprocess
import sys
import time
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

ROOT = Path(r"D:\origa_worktree\origa")
DB_PATH = ROOT / "tmp" / "pitch_sources" / "entries.db"
FURIGANA_PATH = ROOT / "cdn" / "dictionaries" / "JmdictFurigana.txt"
INDEX_PATH = ROOT / "cdn" / "pitch" / "index.json"
INDEX_BAK = ROOT / "cdn" / "pitch" / "index.json.bak"
AUDIO_DIR = ROOT / "cdn" / "pitch" / "audio"
NHK_MEDIA = ROOT / "tmp" / "nhk_extract" / "nhk16" / "media"

PITCH_RE = re.compile(r"\[(\d+)\]")
HASH_LEN = 16
WORKERS = 8


def has_kanji(text: str) -> bool:
    return any(0x4E00 <= ord(c) <= 0x9FFF or 0x3400 <= ord(c) <= 0x4DBF for c in text)


def extract_pitch(display: str | None) -> int | None:
    if not display:
        return None
    m = PITCH_RE.search(display)
    return int(m.group(1)) if m else None


def convert_one(fp: str) -> tuple[str, str | None]:
    """Convert mp3 to opus via ffmpeg pipe output, return (fp, opus_hash.opus or None)."""
    mp3_name = Path(fp).name
    mp3_file = NHK_MEDIA / mp3_name
    if not mp3_file.exists():
        return (fp, None)

    try:
        proc = subprocess.run(
            [
                "ffmpeg", "-loglevel", "error",
                "-i", str(mp3_file),
                "-c:a", "libopus", "-b:a", "32k", "-vn",
                "-f", "opus", "pipe:1",
            ],
            capture_output=True, timeout=15,
        )
        if proc.returncode != 0 or not proc.stdout:
            return (fp, None)

        sha = hashlib.sha256(proc.stdout).hexdigest()[:HASH_LEN]
        opus_name = f"{sha}.opus"
        out_path = AUDIO_DIR / opus_name
        if not out_path.exists():
            out_path.write_bytes(proc.stdout)
        return (fp, opus_name)
    except Exception:
        return (fp, None)


def main():
    t0 = time.time()
    print("=== NHK Pitch Patcher v2 (NHK ordering, parallel) ===\n")

    # 1. Restore original index from backup (undo v1 patches)
    print("Restoring original index from backup...")
    if INDEX_BAK.exists():
        shutil.copy2(INDEX_BAK, INDEX_PATH)
        print("  Restored from backup")
    else:
        print("  No backup found, using current index")

    # 2. Load original index
    with open(INDEX_PATH, encoding="utf-8") as f:
        old = json.load(f)
    old_entries = old["entries"]
    print(f"  {len(old_entries)} entries loaded (v{old['v']})")

    # 3. Parse furigana — SET of valid readings per word
    print("Parsing furigana...")
    furigana_readings: dict[str, set[str]] = {}
    furigana_order: dict[str, list[str]] = {}
    with open(FURIGANA_PATH, encoding="utf-8-sig") as f:
        for line in f:
            parts = line.strip().split("|")
            if len(parts) < 2:
                continue
            word, reading = parts[0], parts[1]
            furigana_readings.setdefault(word, set()).add(reading)
            lo = furigana_order.setdefault(word, [])
            if reading not in lo:
                lo.append(reading)
    print(f"  {len(furigana_readings)} words")

    # 4. Load NHK entries from DB, GROUPED BY expression, ordered by id
    print("Loading NHK entries (ordered by id)...")
    conn = sqlite3.connect(str(DB_PATH))
    nhk_by_expr: dict[str, list[tuple[str, str, int | None]]] = defaultdict(list)
    for expr, reading, fp, display in conn.execute(
        "SELECT expression, reading, file, display "
        "FROM entries WHERE source='nhk16' AND reading IS NOT NULL "
        "ORDER BY id"
    ):
        pitch = extract_pitch(display)
        nhk_by_expr[expr].append((reading, fp, pitch))
    conn.close()
    print(f"  {len(nhk_by_expr)} expressions with NHK data")

    # 5. Determine patches using NHK ordering
    print("\nDetermining patches (using NHK ordering)...")
    patches: dict[str, tuple[str, int | None]] = {}
    kana_patches: dict[str, tuple[str, int | None]] = {}

    for expr, entries in nhk_by_expr.items():
        if not has_kanji(expr):
            continue
        if expr not in furigana_readings:
            continue

        valid_readings = furigana_readings[expr]

        for reading, fp, pitch in entries:
            if reading in valid_readings:
                if expr not in patches:
                    patches[expr] = (fp, pitch)
                if reading not in kana_patches:
                    kana_patches[reading] = (fp, pitch)

    print(f"  {len(patches)} kanji words to patch")
    print(f"  {len(kana_patches)} kana readings")

    # Show test words
    for w in ["空く", "本", "東", "前", "貴方", "風", "一寸"]:
        if w in patches:
            fp, p = patches[w]
            readings = furigana_order.get(w, [])
            chosen_reading = "?"
            for r in readings:
                if r in kana_patches and kana_patches[r][0] == fp:
                    chosen_reading = r
                    break
            print(f"  {w}: NHK chose reading={chosen_reading}, p={p}")
        else:
            print(f"  {w}: NO NHK data")

    # 6. Collect unique mp3 files needed
    needed_files: set[str] = set()
    for fp, _ in patches.values():
        needed_files.add(fp)
    for fp, _ in kana_patches.values():
        needed_files.add(fp)
    print(f"\n  {len(needed_files)} unique NHK audio files needed")

    # 7. Convert mp3 -> opus (PARALLEL, pipe output — no temp files)
    print(f"\nConverting mp3 -> opus ({WORKERS} workers)...")
    file_map: dict[str, str] = {}
    converted = 0
    failed = 0
    total = len(needed_files)

    with ThreadPoolExecutor(max_workers=WORKERS) as pool:
        futures = {pool.submit(convert_one, fp): fp for fp in needed_files}
        for i, future in enumerate(as_completed(futures), 1):
            fp, opus_name = future.result()
            if opus_name:
                file_map[fp] = opus_name
                converted += 1
            else:
                failed += 1
                if failed <= 10:
                    print(f"  FAIL: {Path(fp).name}")

            if i % 5000 == 0:
                elapsed = time.time() - t0
                rate = i / elapsed
                eta = (total - i) / rate if rate > 0 else 0
                print(
                    f"  {i}/{total} (ok={converted}, fail={failed})"
                    f" — {rate:.0f}/s, ETA {eta:.0f}s"
                )

    print(f"  Converted: {converted}, failed: {failed}")

    # 8. Build new index
    print("\nApplying patches...")
    new_entries = dict(old_entries)
    updated = 0

    for word, (fp, pitch) in patches.items():
        if fp in file_map:
            new_entries[word] = {"f": file_map[fp], "p": pitch}
            updated += 1

    for reading, (fp, pitch) in kana_patches.items():
        if fp in file_map:
            new_entries[reading] = {"f": file_map[fp], "p": pitch}

    # Backup original before writing
    if not INDEX_BAK.exists():
        shutil.copy2(INDEX_PATH, INDEX_BAK)
        print(f"  Created backup: {INDEX_BAK.name}")

    # Write
    sorted_entries = dict(sorted(new_entries.items()))
    with open(INDEX_PATH, "w", encoding="utf-8") as f:
        json.dump(
            {"v": 2, "total": len(sorted_entries), "entries": sorted_entries},
            f, ensure_ascii=False, separators=(",", ":"),
        )

    elapsed = time.time() - t0

    # Summary
    print(f"\n{'='*50}")
    print(f"  Total entries:    {len(sorted_entries)} (was {len(old_entries)})")
    print(f"  Kanji patched:    {updated}")
    print(f"  Audio converted:  {converted}")
    print(f"  Failed:           {failed}")
    print(f"  Elapsed:          {elapsed:.1f}s")
    print(f"{'='*50}")

    # Verify test words
    print("\nTest words (NEW vs OLD):")
    for word in ["空く", "本", "東", "前", "貴方", "風", "一寸"]:
        old_e = old_entries.get(word, {})
        new_e = new_entries.get(word, {})
        changed = "CHANGED" if old_e.get("f") != new_e.get("f") else "same"
        readings = furigana_order.get(word, [])
        chosen = "?"
        if word in patches:
            fp_w, _ = patches[word]
            for r in readings:
                if r in kana_patches and kana_patches[r][0] == fp_w:
                    chosen = r
                    break
        print(f"  {word} [{chosen}]: p={new_e.get('p')} [{changed}]")

    print("\nDONE.")


if __name__ == "__main__":
    main()
