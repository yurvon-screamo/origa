"""Export transcriptions from joujiboi/japanese-anime-speech-v2 to JSON.

Uses direct HTTP download of audio_transcription_list.txt instead of the
`datasets` library to avoid torchcodec/ffmpeg DLL issues on Windows.
"""

import json
import os
import sys
import urllib.request
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
OUTPUT_FILE = SCRIPT_DIR / "phrase_dataset_raw.json"
CACHE_FILE = SCRIPT_DIR / "audio_transcription_list.txt"
TRANSCRIPTIONS_URL = (
    "https://huggingface.co/datasets/joujiboi/japanese-anime-speech-v2"
    "/resolve/main/audio_transcription_list.txt"
)
PROGRESS_INTERVAL = 10_000
WRITE_CHUNK = 5_000
DOWNLOAD_CHUNK = 64 * 1024


def download_if_needed() -> None:
    """Download transcriptions file if not cached locally."""
    if CACHE_FILE.exists() and CACHE_FILE.stat().st_size > 0:
        size_mb = CACHE_FILE.stat().st_size / (1024 * 1024)
        print(f"Using cached file: {CACHE_FILE} ({size_mb:.1f} MB)")
        return

    print(f"Downloading transcriptions from:\n  {TRANSCRIPTIONS_URL}")

    try:
        req = urllib.request.Request(
            TRANSCRIPTIONS_URL, headers={"User-Agent": "origa-export/1.0"}
        )
        with urllib.request.urlopen(req, timeout=120) as resp:
            total = resp.headers.get("Content-Length")
            total_mb = int(total) / (1024 * 1024) if total else None
            if total_mb is not None:
                print(f"File size: {total_mb:.1f} MB")
            print(f"Saving to: {CACHE_FILE}")

            downloaded = 0
            with CACHE_FILE.open("wb") as f:
                while True:
                    chunk = resp.read(DOWNLOAD_CHUNK)
                    if not chunk:
                        break
                    f.write(chunk)
                    downloaded += len(chunk)
                    if total:
                        pct = downloaded / int(total) * 100
                        print(
                            f"\r  Downloaded: {downloaded / (1024 * 1024):.1f} MB ({pct:.0f}%)",
                            end="",
                            flush=True,
                        )
            print()
    except Exception as e:
        if CACHE_FILE.exists():
            CACHE_FILE.unlink()
        print(f"ERROR: Failed to download: {e}", file=sys.stderr)
        print("Try opening the URL in a browser and saving manually:", file=sys.stderr)
        print(f"  {TRANSCRIPTIONS_URL}", file=sys.stderr)
        sys.exit(1)

    final_mb = CACHE_FILE.stat().st_size / (1024 * 1024)
    print(f"Download complete: {final_mb:.1f} MB")


def parse_and_export() -> None:
    """Parse transcriptions file and export to JSON."""
    if not CACHE_FILE.exists():
        print(f"ERROR: Cache file not found: {CACHE_FILE}", file=sys.stderr)
        sys.exit(1)

    file_size_mb = CACHE_FILE.stat().st_size / (1024 * 1024)
    print(f"Parsing: {CACHE_FILE} ({file_size_mb:.1f} MB)")

    # Detect delimiter from first line
    sample_line = None
    with CACHE_FILE.open("r", encoding="utf-8") as f:
        for raw in f:
            line = raw.rstrip("\r\n")
            if line:
                sample_line = line
                break

    if sample_line is None:
        print("ERROR: Transcriptions file is empty", file=sys.stderr)
        sys.exit(1)

    delimiter = "\t" if "\t" in sample_line else None
    if delimiter is None:
        for candidate in ["|", ",", ";"]:
            if candidate in sample_line:
                delimiter = candidate
                break
    print(f"Detected delimiter: {'TAB' if delimiter == chr(9) else repr(delimiter)}")
    print(f"Sample line: {sample_line[:120]}...")

    print(f"Writing to: {OUTPUT_FILE}")
    entry_id = 0
    written = 0

    try:
        with CACHE_FILE.open("r", encoding="utf-8") as fin, OUTPUT_FILE.open(
            "w", encoding="utf-8"
        ) as fout:
            fout.write("[\n")
            first = True

            for raw in fin:
                line = raw.rstrip("\r\n")
                if not line or line.startswith("#"):
                    continue

                filepath = ""
                text = ""

                if delimiter:
                    parts = line.split(delimiter, maxsplit=1)
                    if len(parts) == 2:
                        filepath, text = parts[0].strip(), parts[1].strip()
                    elif len(parts) == 1:
                        filepath = parts[0].strip()
                else:
                    line = line.strip()
                    if " " in line:
                        filepath, text = line.split(" ", maxsplit=1)
                        text = text.strip()
                    else:
                        filepath = line

                if not text:
                    continue

                audio_ref = Path(filepath).name if filepath else f"audio_{entry_id}"

                entry = {"id": entry_id, "text": text, "audio_ref": audio_ref}
                entry_json = json.dumps(entry, ensure_ascii=False)

                if not first:
                    fout.write(",\n")
                else:
                    first = False
                fout.write(entry_json)

                entry_id += 1
                written += 1

                if written % WRITE_CHUNK == 0:
                    fout.flush()

                if written % PROGRESS_INTERVAL == 0:
                    print(f"  Progress: {written} entries written")

            fout.write("\n]")

    except Exception as e:
        print(f"ERROR: Failed to write output: {e}", file=sys.stderr)
        sys.exit(1)

    print(f"Done. {written} entries written to {OUTPUT_FILE}")


def main() -> None:
    download_if_needed()
    parse_and_export()


if __name__ == "__main__":
    main()
