#!/usr/bin/env python3
"""Targeted fix for specific problem words in pitch index.

Fixes: 本→ほん, 貴方→あなた, 一寸→ちょっと, and other misread words.
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
INDEX_PATH = ROOT / "cdn" / "pitch" / "index.json"
AUDIO_DIR = ROOT / "cdn" / "pitch" / "audio"
TEMP_DIR = ROOT / "tmp" / "pitch_fix"

PITCH_RE = re.compile(r"\[(\d+)\]")
HASH_LEN = 16

# Words that need fixing: (word, correct_reading)
FIXES = [
    ("本", "ほん"),
    ("貴方", "あなた"),
    ("一寸", "ちょっと"),
]

# Source media directories (in priority order)
MEDIA_DIRS = {
    "nhk16": ROOT / "tmp" / "nhk_extract" / "nhk16" / "media",
    "daijisen": ROOT / "tmp" / "daijisen_extract" / "daijisen" / "media",
}


def extract_pitch(display: str | None) -> int | None:
    if not display:
        return None
    m = PITCH_RE.search(display)
    return int(m.group(1)) if m else None


def convert_mp3(mp3_path: Path, work_id: int) -> str | None:
    mp3_tmp = TEMP_DIR / f"{work_id}.mp3"
    opus_tmp = TEMP_DIR / f"{work_id}.opus"
    try:
        shutil.copy2(mp3_path, mp3_tmp)
        proc = subprocess.run(
            ["ffmpeg", "-loglevel", "error", "-i", str(mp3_tmp),
             "-c:a", "libopus", "-b:a", "32k", "-vn", "-y", str(opus_tmp)],
            capture_output=True, timeout=10,
        )
        if proc.returncode != 0:
            return None
        opus_data = opus_tmp.read_bytes()
        sha = hashlib.sha256(opus_data).hexdigest()[:HASH_LEN]
        opus_name = f"{sha}.opus"
        out = AUDIO_DIR / opus_name
        if not out.exists():
            out.write_bytes(opus_data)
        return opus_name
    except Exception:
        return None
    finally:
        mp3_tmp.unlink(missing_ok=True)
        opus_tmp.unlink(missing_ok=True)


def main():
    print("=== Targeted Pitch Fixer ===\n")
    TEMP_DIR.mkdir(parents=True, exist_ok=True)

    with open(INDEX_PATH, encoding="utf-8") as f:
        data = json.load(f)
    entries = data["entries"]
    print(f"Current index: {len(entries)} entries")

    conn = sqlite3.connect(str(DB_PATH))

    for word, correct_reading in FIXES:
        print(f"\nFixing {word} → {correct_reading}...")

        # Find audio entry in DB (try sources in priority order)
        found = False
        for source in ["nhk16", "daijisen", "shinmeikai8", "jpod"]:
            rows = conn.execute(
                "SELECT file, display FROM entries "
                "WHERE expression=? AND reading=? AND source=? ORDER BY id LIMIT 1",
                (word, correct_reading, source),
            ).fetchall()
            if not rows:
                continue

            fp, display = rows[0]
            pitch = extract_pitch(display)
            media_dir = MEDIA_DIRS.get(source)
            if not media_dir:
                print(f"  {source}: found but no extracted media dir")
                continue

            mp3_name = Path(fp).name
            mp3_file = media_dir / mp3_name
            if not mp3_file.exists():
                print(f"  {source}: {mp3_name} not found on disk")
                continue

            print(f"  Found in {source}: {fp} (pitch={pitch})")
            opus_name = convert_mp3(mp3_file, hash(word) % 10000)
            if opus_name:
                entries[word] = {"f": opus_name, "p": pitch}
                print(f"  Converted: {opus_name} (pitch={pitch})")
                # Also update the kana reading entry
                entries[correct_reading] = {"f": opus_name, "p": pitch}
                print(f"  Also updated kana entry: {correct_reading}")
                found = True
                break
            else:
                print(f"  Conversion FAILED")

        if not found:
            print(f"  WARNING: Could not find audio for {word} ({correct_reading})")

    conn.close()

    # Write updated index
    sorted_entries = dict(sorted(entries.items()))
    with open(INDEX_PATH, "w", encoding="utf-8") as f:
        json.dump(
            {"v": 2, "total": len(sorted_entries), "entries": sorted_entries},
            f, ensure_ascii=False, separators=(",", ":"),
        )

    # Verify
    print(f"\n{'='*50}")
    print(f"  Total entries: {len(sorted_entries)}")
    print(f"\nTest words:")
    for w in ["空く", "本", "東", "前", "貴方", "風", "一寸"]:
        e = entries.get(w, {})
        print(f"  {w}: p={e.get('p','N/A')} f={e.get('f','N/A')[:20]}")

    if TEMP_DIR.exists():
        shutil.rmtree(TEMP_DIR, ignore_errors=True)

    print("\nDONE.")


if __name__ == "__main__":
    main()
