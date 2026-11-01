#!/usr/bin/env python3
"""Turn a generated mascot image into a clean code sprite matrix.

Pipeline: downscale to a tight pixel grid -> snap every pixel to the Origa
palette (palette.js is the single source of truth) -> flood-fill the
background from the corners to transparency -> emit ASCII matrix + preview.

Usage:
  python scripts/sprite_from_image.py <image> [--w 32] [--h 48] [--out records/from-image]

The ASCII matrix goes into mascot/origa.js (POSES), the PNG is a preview.
"""
import argparse
import re
import sys
from collections import deque
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent


def load_palette():
    """Parse hex colors out of mascot/palette.js (keeps it the only source)."""
    text = (ROOT / "mascot" / "palette.js").read_text(encoding="utf-8")
    pal = {}
    for m in re.finditer(r'(\w+):\s*"#([0-9a-fA-F]{6})"', text):
        pal[m.group(1)] = tuple(int(m.group(2)[i : i + 2], 16) for i in (0, 2, 4))
    if not pal:
        sys.exit("no colors found in mascot/palette.js")
    return pal


def dist(a, b):
    return sum((x - y) ** 2 for x, y in zip(a, b)) ** 0.5


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("image", type=Path)
    ap.add_argument("--w", type=int, default=0, help="target width (or use --cell)")
    ap.add_argument("--h", type=int, default=0, help="target height (or use --cell)")
    ap.add_argument(
        "--cell",
        type=int,
        default=0,
        help="source pixel-art cell size: downscale by exact integer blocks (best when the source has a regular grid)",
    )
    ap.add_argument("--out", type=Path, default=ROOT / "records" / "from-image")
    ap.add_argument("--zoom", type=int, default=8)
    ap.add_argument("--bg-threshold", type=float, default=34.0)
    args = ap.parse_args()

    pal = load_palette()
    entries = sorted(pal.items())
    names = [n for n, _ in entries]
    colors = [c for _, c in entries]

    img = Image.open(args.image).convert("RGB")
    if args.cell:
        # exact integer blocks: each source cell becomes one sprite pixel via
        # MAJORITY color (border cells keep hard edges instead of muddy blends)
        iw, ih = img.size
        if iw % args.cell or ih % args.cell:
            img = img.crop((0, 0, iw - iw % args.cell, ih - ih % args.cell))
        w_blocks, h_blocks = img.size[0] // args.cell, img.size[1] // args.cell
        px = img.load()
        small = Image.new("RGB", (w_blocks, h_blocks))
        sp = small.load()
        for by in range(h_blocks):
            for bx in range(w_blocks):
                tally = {}
                for dy in range(args.cell):
                    for dx in range(args.cell):
                        r, g, b = px[bx * args.cell + dx, by * args.cell + dy]
                        key = (r >> 4, g >> 4, b >> 4)  # 16-level cluster key
                        t = tally.setdefault(key, [0, [0, 0, 0]])
                        t[0] += 1
                        t[1][0] += r; t[1][1] += g; t[1][2] += b
                best_key = max(tally, key=lambda k: tally[k][0])
                cnt, acc = tally[best_key]
                sp[bx, by] = (acc[0] // cnt, acc[1] // cnt, acc[2] // cnt)
    else:
        if not args.w or not args.h:
            sys.exit("specify either --cell or --w/--h")
        small = img.resize((args.w, args.h), Image.LANCZOS)
    args.w, args.h = small.size

    # background reference: median of the four corner colors
    corners = [small.getpixel(p) for p in ((0, 0), (small.size[0] - 1, 0), (0, small.size[1] - 1), (small.size[0] - 1, small.size[1] - 1))]
    corners.sort(key=lambda c: sum(c))
    cream = corners[len(corners) // 2]

    # snap each pixel to the palette
    idx = [[0] * args.w for _ in range(args.h)]
    grid = [[None] * args.w for _ in range(args.h)]
    for y in range(args.h):
        for x in range(args.w):
            px = small.getpixel((x, y))
            best = min(range(len(colors)), key=lambda i: dist(px, colors[i]))
            idx[y][x] = best
            grid[y][x] = colors[best]

    # flood-fill the background from the four corners: pixels within a tight
    # radius of the corner tone that stay connected to the border are "background"
    def is_bglike(c):
        return dist(c, cream) < args.bg_threshold

    bg = [[False] * args.w for _ in range(args.h)]
    q = deque()
    for x in range(args.w):
        for y in (0, args.h - 1):
            if is_bglike(grid[y][x]) and not bg[y][x]:
                bg[y][x] = True
                q.append((x, y))
    for y in range(args.h):
        for x in (0, args.w - 1):
            if is_bglike(grid[y][x]) and not bg[y][x]:
                bg[y][x] = True
                q.append((x, y))
    while q:
        x, y = q.popleft()
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = x + dx, y + dy
            if 0 <= nx < args.w and 0 <= ny < args.h and not bg[ny][nx] and is_bglike(grid[ny][nx]):
                bg[ny][nx] = True
                q.append((nx, ny))

    # grow the cut into white pixels ADJACENT to the cut: the generated art
    # leaves a pale halo (background snapped to the white ramp entry) around
    # the figure. Growing along w-only borders eats the halo and stops at the
    # first non-white pixel (hair/hood are the barrier), so white INSIDE the
    # figure (drawstrings, eye whites) — not touching the halo — survives.
    for _ in range(24):
        grew = False
        for y in range(args.h):
            for x in range(args.w):
                if bg[y][x] or grid[y][x] != colors[names.index("w")]:
                    continue
                if any(
                    0 <= x + dx < args.w and 0 <= y + dy < args.h and bg[y + dy][x + dx]
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))
                ):
                    bg[y][x] = True
                    grew = True
        if not grew:
            break

    # trim to the sprite bounding box
    xs = [x for y in range(args.h) for x in range(args.w) if not bg[y][x]]
    ys = [y for y in range(args.h) for x in range(args.w) if not bg[y][x]]
    if not xs:
        sys.exit("sprite is empty: every pixel was classified as background")
    x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)

    # emit matrix rows ("." for background)
    lines = []
    for y in range(y0, y1 + 1):
        row = []
        for x in range(x0, x1 + 1):
            row.append("." if bg[y][x] else names[idx[y][x]])
        lines.append("".join(row))

    args.out.mkdir(parents=True, exist_ok=True)
    txt = args.out / "matrix.txt"
    txt.write_text("\n".join(lines) + "\n", encoding="utf-8")

    # preview PNG at integer zoom
    w, h = x1 - x0 + 1, y1 - y0 + 1
    out = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    for y in range(h):
        for x in range(w):
            if not bg[y0 + y][x0 + x]:
                c = grid[y0 + y][x0 + x]
                out.putpixel((x, y), (c[0], c[1], c[2], 255))
    big = out.resize((w * args.zoom, h * args.zoom), Image.NEAREST)
    png = args.out / "sprite.png"
    big.save(png)

    print(f"sprite: {w}x{h} (from {img.size[0]}x{img.size[1]})")
    print(f"matrix: {txt}")
    print(f"preview: {png} (zoom x{args.zoom})")
    print("---- paste into mascot/origa.js ----")
    print("[")
    for line in lines:
        print(f'  "{line}",')
    print("]")


if __name__ == "__main__":
    main()
