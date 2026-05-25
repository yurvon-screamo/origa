#!/usr/bin/env python3
"""Final pitch patcher: fix known issues + add daijisen coverage.

1. Fix NHK reading order issues (e.g., 本 should be ほん not もと)
2. Add daijisen entries for words without NHK coverage (e.g., 貴方, 一寸)
3. Sequential, gentle processing.

Usage: python scripts/patch_pitch_final.py
"""

import hashlib
import json
import re
import shutil
import sqlite3
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(r"D:\origa_worktree\origa")
DB_PATH = ROOT / "tmp" / "pitch_sources" / "entries.db"
FURIGANA_PATH = ROOT / "cdn" / "dictionaries" / "JmdictFurigana.txt"
INDEX_PATH = ROOT / "cdn" / "pitch" / "index.json"
AUDIO_DIR = ROOT / "cdn" / "pitch" / "audio"
NHK_MEDIA = ROOT / "tmp" / "nhk_extract" / "nhk16" / "media"
DAIJISEN_MEDIA = ROOT / "tmp" / "daijisen_extract" / "daijisen" / "media"
TEMP_DIR = ROOT / "tmp" / "pitch_final"

PITCH_RE = re.compile(r"\[(\d+)\]")
HASH_LEN = 16

# Manual overrides: word -> preferred reading
# These are words where NHK ordering picked the wrong reading
OVERRIDES: dict[str, str] = {
    "本": "ほん",
    "空く": "あく",
    "前": "まえ",
}


def has_kanji(text: str) -> bool:
    return any(0x4E00 <= ord(c) <= 0x9FFF or 0x3400 <= ord(c) <= 0x4DBF for c in text)


def extract_pitch(display: str | None) -> int | None:
    if not display:
        return None
    m = PITCH_RE.search(display)
    return int(m.group(1)) if m else None


def convert_mp3_to_opus(mp3_path: Path, work_id: int) -> str | None:
    """Convert mp3 to opus, return hash filename or None on failure."""
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
        out_path = AUDIO_DIR / opus_name
        if not out_path.exists():
            out_path.write_bytes(opus_data)
        return opus_name
    except Exception:
        return None
    finally:
        mp3_tmp.unlink(missing_ok=True)
        opus_tmp.unlink(missing_ok=True)


def main():
    print("=== Final Pitch Patcher (fixes + daijisen) ===\n")
    TEMP_DIR.mkdir(parents=True, exist_ok=True)

    # 1. Load current index
    with open(INDEX_PATH, encoding="utf-8") as f:
        data = json.load(f)
    entries = data["entries"]
    print(f"Current index: {len(entries)} entries, v{data['v']}")

    # 2. Parse furigana
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

    # 3. Load DB entries
    print("Loading entries from DB...")
    conn = sqlite3.connect(str(DB_PATH))

    # NHK: expression -> [(reading, file, pitch)] by id
    nhk_by_expr: dict[str, list[tuple[str, str, int | None]]] = defaultdict(list)
    for expr, reading, fp, display in conn.execute(
        "SELECT expression, reading, file, display "
        "FROM entries WHERE source='nhk16' AND reading IS NOT NULL ORDER BY id"
    ):
        nhk_by_expr[expr].append((reading, fp, extract_pitch(display)))

    # Daijisen: expression -> [(reading, file, pitch)] by id
    daijisen_by_expr: dict[str, list[tuple[str, str, int | None]]] = defaultdict(list)
    for expr, reading, fp, display in conn.execute(
        "SELECT expression, reading, file, display "
        "FROM entries WHERE source='daijisen' AND reading IS NOT NULL ORDER BY id"
    ):
        daijisen_by_expr[expr].append((reading, fp, extract_pitch(display)))

    conn.close()
    print(f"  NHK: {len(nhk_by_expr)} expressions")
    print(f"  Daijisen: {len(daijisen_by_expr)} expressions")

    # 4. Build correct mapping
    print("\nBuilding correct kanji -> audio mapping...")

    # For each kanji word: try override first, then NHK (with correct reading), then daijisen
    patches: dict[str, tuple[str, str, int | None]] = {}  # word -> (source, mp3_path, pitch)
    kana_patches: dict[str, tuple[str, str, int | None]] = {}
    stats = {"override": 0, "nhk": 0, "daijisen": 0, "skipped": 0}

    for word, readings in furigana_order.items():
        if not has_kanji(word):
            continue

        valid = furigana_readings[word]
        chosen_reading = None
        chosen_fp = None
        chosen_pitch = None
        chosen_source = None

        # Check override first
        if word in OVERRIDES:
            override_reading = OVERRIDES[word]
            if override_reading in valid:
                # Try NHK with this reading
                for reading, fp, pitch in nhk_by_expr.get(word, []):
                    if reading == override_reading:
                        chosen_reading = override_reading
                        chosen_fp = fp
                        chosen_pitch = pitch
                        chosen_source = "nhk16"
                        break
                # If no NHK, try daijisen
                if not chosen_fp:
                    for reading, fp, pitch in daijisen_by_expr.get(word, []):
                        if reading == override_reading:
                            chosen_reading = override_reading
                            chosen_fp = fp
                            chosen_pitch = pitch
                            chosen_source = "daijisen"
                            break
                if chosen_fp:
                    stats["override"] += 1

        # If no override or override failed, try NHK with any valid reading
        if not chosen_fp:
            for reading, fp, pitch in nhk_by_expr.get(word, []):
                if reading in valid:
                    chosen_reading = reading
                    chosen_fp = fp
                    chosen_pitch = pitch
                    chosen_source = "nhk16"
                    stats["nhk"] += 1
                    break

        # Fallback to daijisen
        if not chosen_fp:
            for reading, fp, pitch in daijisen_by_expr.get(word, []):
                if reading in valid:
                    chosen_reading = reading
                    chosen_fp = fp
                    chosen_pitch = pitch
                    chosen_source = "daijisen"
                    stats["daijisen"] += 1
                    break

        if not chosen_fp:
            stats["skipped"] += 1
            continue

        patches[word] = (chosen_source, chosen_fp, chosen_pitch)

        # Also add kana entry for the chosen reading
        if chosen_reading and chosen_reading not in kana_patches:
            kana_patches[chosen_reading] = (chosen_source, chosen_fp, chosen_pitch)

    # Also add kana entries for ALL readings that have audio
    for word, readings in furigana_order.items():
        if not has_kanji(word):
            continue
        for reading in readings:
            if reading in kana_patches:
                continue
            # Try NHK
            for r, fp, pitch in nhk_by_expr.get(word, []):
                if r == reading:
                    kana_patches[reading] = ("nhk16", fp, pitch)
                    break
            if reading in kana_patches:
                continue
            # Try daijisen
            for r, fp, pitch in daijisen_by_expr.get(word, []):
                if r == reading:
                    kana_patches[reading] = ("daijisen", fp, pitch)
                    break

    print(f"  Override: {stats['override']}, NHK: {stats['nhk']}, "
          f"Daijisen: {stats['daijisen']}, Skipped: {stats['skipped']}")
    print(f"  {len(patches)} kanji patches, {len(kana_patches)} kana patches")

    # 5. Collect unique mp3 files needed
    needed: dict[tuple[str, str], Path] = {}  # (source, mp3_path) -> disk_path
    for source, fp, _ in patches.values():
        needed.setdefault((source, fp), None)
    for source, fp, _ in kana_patches.values():
        needed.setdefault((source, fp), None)

    # Map source to media dir
    media_dirs = {"nhk16": NHK_MEDIA, "daijisen": DAIJISEN_MEDIA}
    needed_list = []
    for (source, fp) in needed:
        media_dir = media_dirs[source]
        mp3_name = Path(fp).name
        mp3_file = media_dir / mp3_name
        if mp3_file.exists():
            needed_list.append((source, fp, mp3_file))

    print(f"\n  {len(needed)} unique files, {len(needed_list)} found on disk")

    # 6. Convert files (sequential)
    print("Converting mp3 -> opus (sequential)...")
    file_map: dict[tuple[str, str], str] = {}  # (source, mp3_path) -> opus_hash
    converted = 0
    failed = 0
    already_exists = 0

    for i, (source, fp, mp3_file) in enumerate(needed_list):
        opus_name = convert_mp3_to_opus(mp3_file, i)
        if opus_name:
            file_map[(source, fp)] = opus_name
            # Check if it was already in the audio dir
            if (AUDIO_DIR / opus_name).exists():
                already_exists += 1
            converted += 1
        else:
            failed += 1

        if (converted + failed) % 2000 == 0:
            print(f"  {converted + failed}/{len(needed_list)} (ok={converted}, fail={failed})")

    print(f"  Converted: {converted} (already existed: {already_exists}), failed: {failed}")

    # 7. Apply patches
    print("\nApplying patches...")
    new_entries = dict(entries)
    updated_kanji = 0
    updated_kana = 0

    for word, (source, fp, pitch) in patches.items():
        key = (source, fp)
        if key in file_map:
            new_entries[word] = {"f": file_map[key], "p": pitch}
            updated_kanji += 1

    for reading, (source, fp, pitch) in kana_patches.items():
        key = (source, fp)
        if key in file_map:
            new_entries[reading] = {"f": file_map[key], "p": pitch}
            updated_kana += 1

    # 8. Write index
    sorted_entries = dict(sorted(new_entries.items()))
    with open(INDEX_PATH, "w", encoding="utf-8") as f:
        json.dump(
            {"v": 2, "total": len(sorted_entries), "entries": sorted_entries},
            f, ensure_ascii=False, separators=(",", ":"),
        )

    # 9. Summary
    print(f"\n{'='*50}")
    print(f"  Total entries:    {len(sorted_entries)} (was {len(entries)})")
    print(f"  Kanji patched:    {updated_kanji}")
    print(f"  Kana patched:     {updated_kana}")
    print(f"  Audio converted:  {converted}")
    print(f"  Failed:           {failed}")
    print(f"{'='*50}")

    # 10. Verify test words
    print("\nTest words:")
    for word in ["空く", "本", "東", "前", "貴方", "風", "一寸"]:
        old_e = entries.get(word, {})
        new_e = new_entries.get(word, {})
        changed = "CHANGED" if old_e.get("f") != new_e.get("f") else "same"
        # Find chosen reading
        if word in patches:
            source, fp, pitch = patches[word]
            # Find reading
            readings = furigana_order.get(word, [])
            chosen = "?"
            for r in readings:
                if r in kana_patches:
                    ks, kf, kp = kana_patches[r]
                    if ks == source and kf == fp:
                        chosen = r
                        break
            print(f"  {word} [{chosen}]: p={new_e.get('p')} f={new_e.get('f','N/A')[:20]}... [{changed}]")
        else:
            print(f"  {word}: NOT PATCHED [{old_e.get('p','?')}]")

    # Cleanup
    if TEMP_DIR.exists():
        shutil.rmtree(TEMP_DIR, ignore_errors=True)

    print("\nDONE.")


if __name__ == "__main__":
    main()
