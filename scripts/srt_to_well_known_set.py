import argparse
import json
import re
import subprocess  # nosec B404
import tempfile
from pathlib import Path
from urllib.parse import unquote, urlparse

import requests


def _clean_line(line: str) -> str:
    line = re.sub(r"<[^>]+>", "", line)
    line = re.sub(r"\{[^}]+\}", "", line)
    return line.strip()


def _extract_content_lines(lines: list) -> list:
    if len(lines) >= 3 and "-->" in lines[1]:
        return lines[2:]
    elif len(lines) >= 2 and "-->" in lines[0]:
        return lines[1:]
    elif len(lines) >= 3:
        return lines[2:]
    return []


def _process_block(block: str, split_lines: bool) -> list:
    lines = [line.strip() for line in block.split("\n") if line.strip()]
    if len(lines) < 3:
        return []

    content_lines = _extract_content_lines(lines)
    cleaned_lines = [_clean_line(line) for line in content_lines]
    cleaned_lines = [line for line in cleaned_lines if line]

    if split_lines:
        return cleaned_lines
    elif cleaned_lines:
        return [" ".join(cleaned_lines)]
    return []


def parse_srt(srt_content, split_lines=False):
    blocks = re.split(r"\n\s*\n", srt_content.strip())
    text_blocks = []

    for block in blocks:
        text_blocks.extend(_process_block(block, split_lines))

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
        )  # nosec B603
        if result.returncode != 0:
            print(f"Tokenizer error: {result.stderr}")
            return phrases

        # Tokenizer returns space-separated words
        words = result.stdout.strip().split(" ")
        return [w for w in words if w]
    finally:
        Path(temp_path).unlink()


def _download_srt(url: str) -> tuple:
    print(f"Downloading SRT from: {url}")
    try:
        response = requests.get(url, timeout=30)
        response.raise_for_status()

        content_disposition = response.headers.get("Content-Disposition")
        filename = None
        if content_disposition and "filename=" in content_disposition:
            filename = re.findall('filename="?([^";]+)"?', content_disposition)
            if filename:
                filename = unquote(filename[0])

        if not filename:
            filename = Path(urlparse(url).path).name

        if not filename or filename == "":
            filename = "downloaded_subtitles.srt"

        return filename, response.text, None
    except Exception as e:
        return None, None, e


def _read_local_file(input_path: Path) -> tuple:
    if not input_path.exists():
        return None, Exception(f"File {input_path} not found.")

    try:
        with open(input_path, "r", encoding="utf-8") as f:
            return f.read(), None
    except UnicodeDecodeError:
        try:
            with open(input_path, "r", encoding="utf-16") as f:
                return f.read(), None
        except Exception:
            with open(input_path, "r", encoding="latin-1") as f:
                return f.read(), None


def _unique_preserve_order(phrases: list) -> list:
    seen = set()
    result = []
    for p in phrases:
        if p not in seen:
            result.append(p)
            seen.add(p)
    return result


def _get_input_content(args) -> tuple:
    is_url = args.input.startswith(("http://", "https://"))

    if is_url:
        input_name, content, error = _download_srt(args.input)
        if error:
            print(f"Error downloading URL: {error}")
            return None, None
        return input_name, content

    input_path = Path(args.input)
    content, error = _read_local_file(input_path)
    if error:
        print(f"Error reading file: {error}")
        return None, None
    return input_path.name, content


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

    input_name, content = _get_input_content(args)
    if not input_name:
        return

    stem = Path(input_name).stem
    set_id = args.id or stem.lower().replace(" ", "_")
    set_id = re.sub(r"[^a-z0-9_]", "_", set_id)

    title_ru = args.title_ru or f"Колода из субтитров: {stem}"
    title_en = args.title_en or f"Deck from subtitles: {stem}"
    desc_ru = args.desc_ru or f"Автоматически сгенерированная колода из {input_name}"
    desc_en = args.desc_en or f"Automatically generated deck from {input_name}"

    phrases = parse_srt(content, split_lines=args.split_lines)

    if args.tokenize:
        print("Tokenizing and extracting vocabulary words...")
        words = run_tokenizer(phrases)
    else:
        words = _unique_preserve_order(phrases)

    if not words:
        print("Warning: No text blocks found in the SRT file.")
        return

    deck = {
        "level": args.level,
        "content": {
            "Russian": {"title": title_ru, "description": desc_ru},
            "English": {"title": title_en, "description": desc_en},
        },
        "words": words,
    }

    root_dir = Path(__file__).parent.parent
    output_dir = root_dir / "origa_ui" / "public" / "domain" / "well_known_set"
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
