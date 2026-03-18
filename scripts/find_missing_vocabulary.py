#!/usr/bin/env python3
"""Find vocabulary words from well-known sets that are missing from the dictionary."""

import json
from collections import defaultdict
from pathlib import Path


def load_well_known_sets(base_path: Path) -> dict[str, list[str]]:
    """Load all words from well-known set JSON files recursively."""
    word_to_sets: dict[str, list[str]] = defaultdict(list)
    sets_dir = base_path / "origa_ui" / "public" / "domain" / "well_known_set"

    if not sets_dir.exists():
        print(f"Warning: Directory not found: {sets_dir}")
        return word_to_sets

    for json_file in sets_dir.rglob("*.json"):
        if json_file.name == "well_known_sets_meta.json":
            continue

        set_name = json_file.stem
        try:
            with open(json_file, encoding="utf-8-sig") as f:
                data = json.load(f)

            words = data.get("words", [])
            for word in words:
                if word not in word_to_sets[set_name]:
                    word_to_sets[word].append(set_name)
        except json.JSONDecodeError as e:
            print(f"Warning: Could not process {json_file}: {e}")

    return word_to_sets


def load_dictionary(base_path: Path) -> set[str]:
    """Load all vocabulary words from dictionary chunks."""
    dictionary_words: set[str] = set()
    vocab_dir = base_path / "origa_ui" / "public" / "dictionary" / "vocabulary"

    if not vocab_dir.exists():
        print(f"Warning: Directory not found: {vocab_dir}")
        return dictionary_words

    for chunk_file in sorted(vocab_dir.glob("chunk_*.json")):
        try:
            with open(chunk_file, encoding="utf-8-sig") as f:
                data = json.load(f)
            dictionary_words.update(data.keys())
        except json.JSONDecodeError as e:
            print(f"Warning: Could not process {chunk_file}: {e}")

    return dictionary_words


def find_missing_words(
    word_to_sets: dict[str, list[str]], dictionary_words: set[str]
) -> dict[str, list[str]]:
    """Find words that are in sets but not in dictionary."""
    return {
        word: sets
        for word, sets in word_to_sets.items()
        if word not in dictionary_words
    }


def generate_report(
    missing_words: dict[str, list[str]],
    total_set_words: int,
    dict_words: int,
    output_path: Path,
) -> None:
    """Generate markdown report with missing vocabulary analysis."""
    sorted_missing = sorted(missing_words.items(), key=lambda x: (-len(x[1]), x[0]))

    lines = [
        "# Missing Vocabulary",
        "",
        "## Statistics",
        f"- Total unique words in sets: {total_set_words}",
        f"- Words in dictionary: {dict_words}",
        f"- Missing words: {len(missing_words)}",
        "",
        "## Missing Words List",
        "",
        "| Word | Found in sets |",
        "|------|---------------|",
    ]

    for word, sets in sorted_missing:
        sets_str = ", ".join(sorted(sets))
        lines.append(f"| {word} | {sets_str} |")

    if not sorted_missing:
        lines.append("_No missing words found._")

    lines.extend(
        [
            "",
            "## Plain List (for copy-paste)",
            "",
        ]
    )

    for word, _ in sorted_missing:
        lines.append(word)

    with open(output_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))

    print(f"Report saved to: {output_path}")


def main():
    base_path = Path(__file__).resolve().parent.parent
    output_path = base_path / "missing_vocabulary.md"

    print("Loading well-known sets...")
    word_to_sets = load_well_known_sets(base_path)
    total_set_words = len(word_to_sets)

    print("Loading dictionary...")
    dictionary_words = load_dictionary(base_path)

    print("Finding missing words...")
    missing_words = find_missing_words(word_to_sets, dictionary_words)

    print("Generating report...")
    generate_report(missing_words, total_set_words, len(dictionary_words), output_path)

    print(f"\nSummary:")
    print(f"  Unique words in sets: {total_set_words}")
    print(f"  Dictionary words: {len(dictionary_words)}")
    print(f"  Missing words: {len(missing_words)}")


if __name__ == "__main__":
    main()
