#!/usr/bin/env python3
"""
Script to update JLPT JSON files from TSV text files.

This script reads TSV files with format: Kanji\tHiragana\tEnglish
and updates the corresponding JSON files by replacing the "words" array
with the Kanji column values.
"""

import json
from pathlib import Path


def parse_tsv_file(tsv_path: Path) -> list[str]:
    """
    Parse a TSV file and extract words from the first column (Kanji).

    Args:
        tsv_path: Path to the TSV file

    Returns:
        List of words from the Kanji column
    """
    words = []

    with open(tsv_path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    # Skip header line
    for line in lines[1:]:
        line = line.strip()
        if not line:
            continue

        # Split by tab and get first column (Kanji)
        parts = line.split("\t")
        if parts:
            kanji = parts[0].strip()
            if kanji:
                words.append(kanji)

    return words


def update_json_file(json_path: Path, words: list[str]) -> None:
    """
    Update a JSON file with new words array.

    Args:
        json_path: Path to the JSON file
        words: List of words to put in the "words" array
    """
    # Read existing JSON
    with open(json_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    # Update words array
    data["words"] = words

    # Write back with proper formatting
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent="\t")
        f.write("\n")  # Add trailing newline

    print(f"Updated {json_path.name}: {len(words)} words")


def main():
    """Main function to update all JLPT JSON files."""
    # Define the base directory
    base_dir = (
        Path(__file__).parent.parent
        / "origa_ui"
        / "public"
        / "domain"
        / "well_known_set"
    )

    # Define mappings: (tsv_file, json_file)
    file_mappings = [
        ("n1.new.txt", "jlpt_n1.json"),
        ("n2.new.txt", "jlpt_n2.json"),
        ("n3.new.txt", "jlpt_n3.json"),
        ("n4.new.txt", "jlpt_n4.json"),
        ("n5.new.txt", "jlpt_n5.json"),
    ]

    print(f"Base directory: {base_dir}")
    print()

    for tsv_filename, json_filename in file_mappings:
        tsv_path = base_dir / tsv_filename
        json_path = base_dir / json_filename

        # Check if files exist
        if not tsv_path.exists():
            print(f"ERROR: TSV file not found: {tsv_path}")
            continue

        if not json_path.exists():
            print(f"ERROR: JSON file not found: {json_path}")
            continue

        # Parse TSV and get words
        words = parse_tsv_file(tsv_path)

        # Update JSON file
        update_json_file(json_path, words)

    print()
    print("All JLPT JSON files have been updated successfully!")


if __name__ == "__main__":
    main()
