#!/usr/bin/env python3
"""Generate favicon.ico and apple-touch-icon.png for origa_landing.

Canonical source: tauri/icons/icon-512.png (512x512 RGBA).
Outputs (committed to origa_landing/public/):
  - favicon.ico         multi-size ICO (16x16, 32x32, 48x48)
  - apple-touch-icon.png 180x180 PNG (Apple Touch Icon spec)

Run: uv run --with pillow python scripts/generate_favicon.py
"""

from pathlib import Path

from PIL import Image

REPO_ROOT = Path(__file__).resolve().parent.parent
SOURCE = REPO_ROOT / "tauri" / "icons" / "icon-512.png"
PUBLIC_DIR = REPO_ROOT / "origa_landing" / "public"
ICO_PATH = PUBLIC_DIR / "favicon.ico"
APPLE_PATH = PUBLIC_DIR / "apple-touch-icon.png"

ICO_SIZES = [(16, 16), (32, 32), (48, 48)]
APPLE_SIZE = (180, 180)


def main() -> None:
    if not SOURCE.exists():
        raise SystemExit(f"source icon not found: {SOURCE}")

    source = Image.open(SOURCE).convert("RGBA")

    PUBLIC_DIR.mkdir(parents=True, exist_ok=True)

    source.save(
        ICO_PATH,
        format="ICO",
        sizes=ICO_SIZES,
    )

    apple = source.resize(APPLE_SIZE, Image.LANCZOS)
    apple.save(APPLE_PATH, format="PNG")

    print(f"wrote {ICO_PATH.relative_to(REPO_ROOT)} ({ICO_PATH.stat().st_size} bytes)")
    print(f"wrote {APPLE_PATH.relative_to(REPO_ROOT)} ({APPLE_PATH.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
