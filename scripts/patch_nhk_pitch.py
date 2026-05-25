#!/usr/bin/env python3
"""Patch existing pitch audio index with NHK data.

Keeps ALL existing entries. Only UPDATES entries where NHK has authoritative
audio with the correct reading (matched via JmdictFurigana).
Sequential processing — no parallelism, gentle on resources.

Usage: python scripts/patch_nhk_pitch.py
"""

import hashlib
import json
import re
import shutil
import sqlite3
import subprocess
import sys
from pathlib import Path

ROOT = Path(r"D:\origa_worktree\origa")
DB_PATH = ROOT / "tmp" / "pitch_sources" / "entries.db"
FURIGANA_PATH = ROOT / "cdn" / "dictionaries" / "JmdictFurigana.txt"
INDEX_PATH = ROOT / "cdn" / "pitch" / "index.json"
AUDIO_DIR = ROOT / "cdn" / "pitch" / "audio"
NHK_MEDIA = ROOT / "tmp" / "nhk_extract" / "nhk16" / "media"
TEMP_DIR = ROOT / "tmp" / "nhk_patch"

PITCH_RE = re.compile(r"\[(\d+)\]")
HASH_LEN = 16


def has_kanji(text: str) -> bool:
    return any(0x4E00 <= ord(c) <= 0x9FFF or 0x3400 <= ord(c) <= 0x4DBF for c in text)


def extract_pitch(display: str | None) -> int | None:
    if not display:
        return None
    m = PITCH_RE.search(display)
    return int(m.group(1)) if m else None


def main():
    print("=== NHK Pitch Patcher ===\n")

    # 1. Load existing index
    print("Loading existing index...")
    with open(INDEX_PATH, encoding="utf-8") as f:
        old = json.load(f)
    old_entries = old["entries"]
    print(f"  {len(old_entries)} existing entries, version={old['v']}")

    # 2. Parse furigana — {kanji: [reading1, reading2, ...]}
    print("Parsing furigana...")
    furigana: dict[str, list[str]] = {}
    with open(FURIGANA_PATH, encoding="utf-8-sig") as f:
        for line in f:
            parts = line.strip().split("|")
            if len(parts) < 2:
                continue
            word, reading = parts[0], parts[1]
            readings = furigana.setdefault(word, [])
            if reading not in readings:
                readings.append(reading)
    print(f"  {len(furigana)} words")

    # 3. Load NHK entries from DB — (expression, reading) -> (file, pitch)
    print("Loading NHK entries from DB...")
    conn = sqlite3.connect(str(DB_PATH))
    nhk_lookup: dict[tuple[str, str], tuple[str, int | None]] = {}
    for expr, reading, fp, display in conn.execute(
        "SELECT expression, reading, file, display "
        "FROM entries WHERE source='nhk16' AND reading IS NOT NULL "
        "ORDER BY id"
    ):
        key = (expr, reading)
        if key not in nhk_lookup:
            nhk_lookup[key] = (fp, extract_pitch(display))
    conn.close()
    print(f"  {len(nhk_lookup)} NHK (expression, reading) pairs")

    # 4. Determine patches
    print("Determining patches...")
    patches: dict[str, tuple[str, int | None]] = {}  # word -> (mp3_path, pitch)
    kana_patches: dict[str, tuple[str, int | None]] = {}  # reading -> same

    for word, readings in furigana.items():
        if not has_kanji(word):
            continue
        kanji_done = False
        for reading in readings:
            key = (word, reading)
            if key in nhk_lookup:
                fp, pitch = nhk_lookup[key]
                if not kanji_done:
                    patches[word] = (fp, pitch)
                    kanji_done = True
                if reading not in kana_patches:
                    kana_patches[reading] = (fp, pitch)

    print(f"  {len(patches)} kanji words to patch")
    print(f"  {len(kana_patches)} kana readings to add/update")

    # Show test words
    for w in ["空く", "本", "東", "前", "貴方", "風", "一寸"]:
        if w in patches:
            fp, p = patches[w]
            print(f"  TEST {w}: will patch (file={Path(fp).name}, p={p})")
        else:
            print(f"  TEST {w}: NO NHK data, keeping existing")

    # 5. Collect unique mp3 files needed
    needed: set[str] = set()
    for fp, _ in patches.values():
        needed.add(fp)
    for fp, _ in kana_patches.values():
        needed.add(fp)
    print(f"\n  {len(needed)} unique NHK audio files to convert")

    # 6. Convert mp3 -> opus (sequential)
    TEMP_DIR.mkdir(parents=True, exist_ok=True)
    file_map: dict[str, str] = {}  # mp3_path -> opus_hash_filename
    converted = 0
    failed = 0

    print("Converting (sequential, no parallelism)...")
    for i, fp in enumerate(needed):
        try:
            mp3_name = Path(fp).name
            mp3_file = NHK_MEDIA / mp3_name
            if not mp3_file.exists():
                failed += 1
                if failed <= 10:
                    print(f"  SKIP: {mp3_name} not found on disk")
                continue

            mp3_tmp = TEMP_DIR / f"{i}.mp3"
            opus_tmp = TEMP_DIR / f"{i}.opus"
            shutil.copy2(mp3_file, mp3_tmp)

            proc = subprocess.run(
                ["ffmpeg", "-loglevel", "error", "-i", str(mp3_tmp),
                 "-c:a", "libopus", "-b:a", "32k", "-vn", "-y", str(opus_tmp)],
                capture_output=True, timeout=10,
            )
            if proc.returncode != 0:
                failed += 1
                mp3_tmp.unlink(missing_ok=True)
                continue

            opus_data = opus_tmp.read_bytes()
            sha = hashlib.sha256(opus_data).hexdigest()[:HASH_LEN]
            opus_name = f"{sha}.opus"
            out_path = AUDIO_DIR / opus_name
            if not out_path.exists():
                out_path.write_bytes(opus_data)

            file_map[fp] = opus_name
            converted += 1
            mp3_tmp.unlink(missing_ok=True)
            opus_tmp.unlink(missing_ok=True)

            if (converted + failed) % 2000 == 0:
                print(f"  {converted + failed}/{len(needed)} (ok={converted}, fail={failed})")

        except Exception as e:
            failed += 1
            if failed <= 10:
                print(f"  ERR: {fp}: {e}")

    print(f"  Done: {converted} converted, {failed} failed")

    # 7. Apply patches
    print("\nApplying patches to index...")
    new_entries = dict(old_entries)
    updated = 0

    for word, (fp, pitch) in patches.items():
        if fp in file_map:
            new_entries[word] = {"f": file_map[fp], "p": pitch}
            updated += 1

    for reading, (fp, pitch) in kana_patches.items():
        if fp in file_map:
            new_entries[reading] = {"f": file_map[fp], "p": pitch}

    # 8. Write index
    backup = INDEX_PATH.with_suffix(".json.bak")
    shutil.copy2(INDEX_PATH, backup)
    print(f"  Backup: {backup.name}")

    sorted_entries = dict(sorted(new_entries.items()))
    with open(INDEX_PATH, "w", encoding="utf-8") as f:
        json.dump(
            {"v": 2, "total": len(sorted_entries), "entries": sorted_entries},
            f, ensure_ascii=False, separators=(",", ":"),
        )
    print(f"  Written {len(sorted_entries)} entries")

    # 9. Summary
    print(f"\n{'='*50}")
    print(f"  Old entries:    {len(old_entries)}")
    print(f"  New entries:    {len(sorted_entries)}")
    print(f"  Kanji patched:  {updated}")
    print(f"  Audio added:    {converted}")
    print(f"  Failed:         {failed}")
    print(f"{'='*50}")

    # 10. Verify test words
    print("\nTest words:")
    for word in ["空く", "本", "東", "前", "貴方", "風", "一寸"]:
        old_e = old_entries.get(word, {})
        new_e = new_entries.get(word, {})
        changed = "CHANGED" if old_e.get("f") != new_e.get("f") else "same"
        print(f"  {word}: p={new_e.get('p')} file={new_e.get('f','N/A')} [{changed}]")

    # Cleanup
    if TEMP_DIR.exists():
        shutil.rmtree(TEMP_DIR, ignore_errors=True)

    print("\nDONE.")


if __name__ == "__main__":
    main()
