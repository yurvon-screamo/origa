#!/usr/bin/env python3
"""Assemble an animated WebP from a directory of PNG frames.

ffmpeg's webp muxer writes sub-rect deltas (each ANMF covers only the pixels
that changed between frames), which reliably crops static sprites whose top
rows never change — the mascot's head was being cut off. Pillow writes every
frame as a full canvas instead.

Usage: python scripts/webp_from_frames.py <frames_dir> <out> <fps> <loop 0|1> <lossless>
"""
import sys
from pathlib import Path

from PIL import Image

frames_dir = Path(sys.argv[1])
out = Path(sys.argv[2])
fps = int(sys.argv[3])
loop = int(sys.argv[4])  # 0 = infinite, 1 = play once
lossless = sys.argv[5] == "1"

pngs = sorted(frames_dir.glob("f*.png"))
if len(pngs) < 2:
    sys.exit(f"expected >=2 frames in {frames_dir}, found {len(pngs)}")

frames = [Image.open(p).convert("RGBA") for p in pngs]
first = frames[0]

kwargs = dict(
    save_all=True,
    append_images=frames[1:],
    duration=int(round(1000 / fps)),
    loop=loop,
    method=6,
)
if lossless:
    kwargs["lossless"] = True
else:
    kwargs["quality"] = 90

first.save(out, "WEBP", **kwargs)

# verify: reopen and confirm every frame has the same full canvas
check = Image.open(out)
n = getattr(check, "n_frames", 1)
sizes = set()
for i in range(n):
    check.seek(i)
    sizes.add(check.size)
assert sizes == {first.size}, f"webp frames have mixed sizes: {sizes}"
print(f"{out}: {n} frames, {first.size[0]}x{first.size[1]}, fps {fps}, loop {loop}, lossless={lossless}")
