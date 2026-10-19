#!/usr/bin/env python3
"""Build the reference-audio pool for example TTS voice cloning.

Selects neutral-tone phrases from the phrases corpus, converts them to
24 kHz mono wav, and writes a deterministic manifest. The pool feeds
``gen_batch.py`` (one ref per item, ``item_id % pool_size``).

Criteria (tuned on the 156k-phrase anime corpus):
    - audio 4–12 s (12–36 KB opus at ~24 kbps)
    - text 12–35 chars
    - neutrality: no ！？…っ♪☆★, no latin/cyrillic (score ≤ 3)

Usage (from repo root, system python3 + ffmpeg):
    python3 tts/build_ref_pool.py --size 80 --out tts-demo/ref_pool
"""
from __future__ import annotations

import argparse
import json
import random
import re
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
PHRASES_DIR = REPO / "cdn" / "phrases"


def neutrality(text: str) -> int:
    score = 0
    score += text.count("！") * 5 + text.count("？") * 4 + text.count("…") * 3
    score += text.count("っ") * 2
    score += text.count("♪") * 3 + text.count("☆") * 3 + text.count("★") * 3
    score += len(re.findall(r"[A-Za-zА-Яа-яЁё]", text)) * 2
    return score


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--size", type=int, default=80, help="pool size")
    parser.add_argument("--seed", type=int, default=528, help="deterministic shuffle")
    parser.add_argument("--out", type=str, required=True, help="output dir")
    args = parser.parse_args()

    rows: list[dict] = []
    for chunk in sorted((PHRASES_DIR / "data").glob("p*.json")):
        rows.extend(json.loads(chunk.read_text()))
    print(f"phrases: {len(rows)}")

    cands = []
    for row in rows:
        audio = PHRASES_DIR / "audio" / f"{row['i']}.opus"
        if not audio.is_file():
            continue
        size = audio.stat().st_size
        text = row["x"]
        if 12_000 < size < 36_000 and 12 <= len(text) <= 35 and neutrality(text) <= 3:
            cands.append((row["i"], text, size))

    print(f"neutral candidates: {len(cands)}")
    rng = random.Random(args.seed)
    rng.shuffle(cands)
    pool = sorted(cands[: args.size])

    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    manifest = []
    for idx, (pid, text, _size) in enumerate(pool):
        dst = out / f"{idx:03d}.wav"
        src = PHRASES_DIR / "audio" / f"{pid}.opus"
        subprocess.run(
            ["ffmpeg", "-y", "-loglevel", "error", "-i", str(src),
             "-ar", "24000", "-ac", "1", str(dst)],
            check=True,
        )
        dur = float(subprocess.run(
            ["ffprobe", "-v", "error", "-show_entries", "format=duration",
             "-of", "csv=p=0", str(dst)],
            capture_output=True, text=True, check=True,
        ).stdout.strip())
        manifest.append({"idx": idx, "phrase_id": pid, "text": text, "dur": dur})

    (out / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=1), encoding="utf-8"
    )
    durs = [m["dur"] for m in manifest]
    print(f"pool: {len(manifest)} refs, dur {min(durs):.1f}–{max(durs):.1f}s (med "
          f"{sorted(durs)[len(durs) // 2]:.1f}s) → {out}")


if __name__ == "__main__":
    main()
