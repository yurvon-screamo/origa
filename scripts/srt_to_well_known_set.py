import argparse
import json
import re
import subprocess
import tempfile
from pathlib import Path
from urllib.parse import unquote, urlparse

import requests


def parse_srt(srt_content, split_lines=False):
    """
    Simple SRT parser that extracts text blocks.
    Removes timestamps, indices, and common HTML/formatting tags.
    """
    # Split by double newline or more to get blocks
    blocks = re.split(r"\n\s*\n", srt_content.strip())
    text_blocks = []

    for block in blocks:
        lines = [l.strip() for l in block.split("\n") if l.strip()]
        if len(lines) >= 3:
            # Typical block structure check
            if "-->" in lines[1]:
                content_lines = lines[2:]
            elif "-->" in lines[0]:
                content_lines = lines[1:]
            else:
                content_lines = lines[2:]

            cleaned_lines = []
            for line in content_lines:
                # Remove HTML-like tags (e.g. <i>, <b>)
                line = re.sub(r"<[^>]+>", "", line)
                # Remove curly brace tags (common in ASS-to-SRT conversions or styling)
                line = re.sub(r"\{[^}]+\}", "", line)
                line = line.strip()
                if line:
                    cleaned_lines.append(line)

            if split_lines:
                text_blocks.extend(cleaned_lines)
            elif cleaned_lines:
                text_blocks.append(" ".join(cleaned_lines))

    return text_blocks


def run_tokenizer(phrases):
    """
    Runs the Rust tokenizer tool on the extracted phrases to get unique vocabulary words.
    """

    cmd = ["cargo", "run", "--bin", "tokenizer", "--"]

    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".txt", encoding="utf-8", delete=False
    ) as f:
        f.write("\n".join(phrases))
        temp_path = f.name

    try:
        result = subprocess.run(
            cmd + ["-f", temp_path], capture_output=True, text=True, encoding="utf-8"
        )
        if result.returncode != 0:
            print(f"Tokenizer error: {result.stderr}")
            return phrases

        # Tokenizer returns space-separated words
        words = result.stdout.strip().split(" ")
        return [w for w in words if w]
    finally:
        Path(temp_path).unlink()


def main():
    parser = argparse.ArgumentParser(
        description="Convert SRT subtitles to Origa WellKnownSet JSON deck."
    )
    parser.add_argument("input", help="Path or URL to the input .srt file")
    parser.add_argument(
        "--id", help="ID for the deck (default: filename without extension)"
    )
    parser.add_argument(
        "--level", default="N1", help="Japanese level (N1, N2, N3, N4, N5) - default N1"
    )
    parser.add_argument("--title-ru", help="Russian title for the deck")
    parser.add_argument("--title-en", help="English title for the deck")
    parser.add_argument("--desc-ru", help="Russian description")
    parser.add_argument("--desc-en", help="English description")
    parser.add_argument(
        "--split-lines",
        action="store_true",
        help="Treat each line in a subtitle block as a separate word",
    )
    parser.add_argument(
        "--tokenize",
        action="store_true",
        help="Run through tokenizer to extract unique vocabulary words (minification)",
    )

    args = parser.parse_args()

    is_url = args.input.startswith(("http://", "https://"))

    if is_url:
        print(f"Downloading SRT from: {args.input}")
        try:
            response = requests.get(args.input, timeout=30)
            response.raise_for_status()

            # Try to get filename from Content-Disposition header
            content_disposition = response.headers.get("Content-Disposition")
            filename = None
            if content_disposition and "filename=" in content_disposition:
                filename = re.findall('filename="?([^";]+)"?', content_disposition)
                if filename:
                    filename = unquote(filename[0])

            if not filename:
                # Fallback to URL path
                filename = Path(urlparse(args.input).path).name

            if not filename or filename == "":
                filename = "downloaded_subtitles.srt"

            input_name = filename
            content = response.text

        except Exception as e:
            print(f"Error downloading URL: {e}")
            return
    else:
        input_path = Path(args.input)
        if not input_path.exists():
            print(f"Error: File {input_path} not found.")
            return
        input_name = input_path.name

        # Read SRT with fallback encoding
        try:
            with open(input_path, "r", encoding="utf-8") as f:
                content = f.read()
        except UnicodeDecodeError:
            try:
                with open(input_path, "r", encoding="utf-16") as f:
                    content = f.read()
            except Exception:
                with open(input_path, "r", encoding="latin-1") as f:
                    content = f.read()
        except Exception as e:
            print(f"Error reading file: {e}")
            return

    # Prepare ID and Titles
    stem = Path(input_name).stem
    set_id = args.id or stem.lower().replace(" ", "_")
    # Filter set_id to be filesystem-friendly
    set_id = re.sub(r"[^a-z0-9_]", "_", set_id)

    title_ru = args.title_ru or f"Колода из субтитров: {stem}"
    title_en = args.title_en or f"Deck from subtitles: {stem}"
    desc_ru = args.desc_ru or f"Автоматически сгенерированная колода из {input_name}"
    desc_en = args.desc_en or f"Automatically generated deck from {input_name}"

    phrases = parse_srt(content, split_lines=args.split_lines)

    # Process through tokenizer if requested
    if args.tokenize:
        print("Tokenizing and extracting vocabulary words...")
        words = run_tokenizer(phrases)
    else:
        # Unique phrases preserving order
        seen = set()
        words = []
        for p in phrases:
            if p not in seen:
                words.append(p)
                seen.add(p)

    if not words:
        print("Warning: No text blocks found in the SRT file.")
        return

    # Construct the JSON structure compatible with origa_ui/public/domain/well_known_set/
    deck = {
        "level": args.level,
        "content": {
            "Russian": {"title": title_ru, "description": desc_ru},
            "English": {"title": title_en, "description": desc_en},
        },
        "words": words,
    }

    # Define output path
    root_dir = Path(__file__).parent.parent
    output_dir = root_dir / "origa_ui" / "public" / "domain" / "well_known_set"

    # Ensure directory exists
    output_dir.mkdir(parents=True, exist_ok=True)

    output_file = output_dir / f"{set_id}.json"

    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(deck, f, ensure_ascii=False, indent=2)

    print(f"Successfully converted {len(words)} entries.")
    print(f"Output saved to: {output_file.relative_to(root_dir)}")
    print("\nNext steps:")
    print(
        "1. Run 'python scripts/generate_well_known_meta.py' to update the global index."
    )
    print(f"2. Restart Origa UI to see the new deck '{title_en}' in the list.")


if __name__ == "__main__":
    main()
