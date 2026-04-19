"""Convert MP3 audio files to Opus format (mono, configurable bitrate).

Requires: ffmpeg with libopus support.
Usage: python scripts/convert_audio_to_opus.py [--input DIR] [--bitrate N] [--keep-mp3]
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path

try:
    from tqdm import tqdm
except ImportError:
    print("tqdm is required. Install with: pip install tqdm")
    sys.exit(1)

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_ROOT = SCRIPT_DIR.parent
DEFAULT_AUDIO_DIR = PROJECT_ROOT / "origa_ui" / "public" / "phrase" / "audio"


def check_ffmpeg() -> None:
    try:
        subprocess.run(
            ["ffmpeg", "-version"],
            capture_output=True,
            check=True,
        )
    except FileNotFoundError:
        print(
            "ffmpeg is not installed or not in PATH.\n"
            "Install it from: https://ffmpeg.org/download.html\n"
            "On Windows: winget install ffmpeg\n"
            "On macOS:   brew install ffmpeg\n"
            "On Linux:   sudo apt install ffmpeg"
        )
        sys.exit(1)
    except subprocess.CalledProcessError as e:
        print(f"ffmpeg check failed: {e}")
        sys.exit(1)


def convert_file(mp3_path: Path, bitrate: int, keep_mp3: bool) -> tuple[bool, str, int]:
    """Convert a single MP3 file to Opus. Returns (success, filename, original_size)."""
    opus_path = mp3_path.with_suffix(".opus")
    original_size = mp3_path.stat().st_size

    cmd = [
        "ffmpeg",
        "-i", str(mp3_path),
        "-c:a", "libopus",
        "-b:a", f"{bitrate}k",
        "-ac", "1",
        "-vn",
        "-y",
        str(opus_path),
    ]

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        if result.returncode != 0:
            return False, mp3_path.name, original_size

        if not keep_mp3:
            mp3_path.unlink()

        return True, mp3_path.name, original_size
    except Exception as e:
        return False, f"{mp3_path.name}: {e}", original_size


def format_size(size_bytes: float) -> str:
    if size_bytes < 1024:
        return f"{size_bytes:.1f} B"
    if size_bytes < 1024 * 1024:
        return f"{size_bytes / 1024:.1f} KB"
    return f"{size_bytes / (1024 * 1024):.1f} MB"


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Convert MP3 files to Opus format for reduced size."
    )
    parser.add_argument(
        "--input",
        type=Path,
        default=DEFAULT_AUDIO_DIR,
        help=f"Directory with MP3 files (default: {DEFAULT_AUDIO_DIR})",
    )
    parser.add_argument(
        "--bitrate",
        type=int,
        default=16,
        help="Opus bitrate in kbps (default: 16)",
    )
    parser.add_argument(
        "--keep-mp3",
        action="store_true",
        default=False,
        help="Keep original MP3 files after conversion",
    )
    args = parser.parse_args()

    audio_dir: Path = args.input
    bitrate: int = args.bitrate
    keep_mp3: bool = args.keep_mp3

    if not audio_dir.is_dir():
        print(f"Directory not found: {audio_dir}")
        sys.exit(1)

    check_ffmpeg()

    mp3_files = sorted(audio_dir.glob("*.mp3"))
    if not mp3_files:
        print("No MP3 files found.")
        sys.exit(0)

    print(f"Found {len(mp3_files)} MP3 files in {audio_dir}")
    print(f"Bitrate: {bitrate} kbps, mono | {'Keep' if keep_mp3 else 'Delete'} MP3 after conversion")
    print()

    original_total = sum(f.stat().st_size for f in mp3_files)
    start_time = time.monotonic()

    converted = 0
    failed: list[str] = []

    with ProcessPoolExecutor() as executor:
        futures = {
            executor.submit(convert_file, mp3, bitrate, keep_mp3): mp3
            for mp3 in mp3_files
        }

        with tqdm(total=len(mp3_files), unit="file") as pbar:
            for future in as_completed(futures):
                success, name, _ = future.result()
                if success:
                    converted += 1
                else:
                    failed.append(name)
                pbar.update(1)

    elapsed = time.monotonic() - start_time

    opus_files = list(audio_dir.glob("*.opus"))
    opus_total = sum(f.stat().st_size for f in opus_files)
    reduction = (1 - opus_total / original_total) * 100 if original_total > 0 else 0

    print()
    print(f"Converted: {converted}/{len(mp3_files)}")
    print(f"Original size:  {format_size(original_total)}")
    print(f"Opus size:      {format_size(opus_total)}")
    print(f"Reduction:      {reduction:.1f}%")
    print(f"Time:           {elapsed:.1f}s")
    print(f"Rate:           {len(mp3_files) / elapsed:.1f} files/s")

    if failed:
        print(f"\nFailed ({len(failed)}):")
        for name in failed:
            print(f"  - {name}")


if __name__ == "__main__":
    main()
