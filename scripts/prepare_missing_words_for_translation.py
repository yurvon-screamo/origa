import json
from pathlib import Path


def extract_plain_list(md_path: Path) -> list[str]:
    content = md_path.read_text(encoding="utf-8")

    plain_list_start = content.find("## Plain List")
    if plain_list_start == -1:
        raise ValueError("Section '## Plain List' not found")

    next_section = content.find("\n## ", plain_list_start + 1)
    if next_section == -1:
        plain_list_content = content[plain_list_start:]
    else:
        plain_list_content = content[plain_list_start:next_section]

    lines = plain_list_content.split("\n")
    words = []
    for line in lines:
        line = line.strip()
        if line and not line.startswith("#"):
            words.append(line)

    return words


def main():
    project_root = Path(__file__).parent.parent
    md_path = project_root / "missing_vocabulary.md"
    output_path = project_root / "scripts" / "missing_words_for_translation.json"

    print(f"Reading {md_path}...")
    words = extract_plain_list(md_path)

    data = {
        word: {"russian_translation": "", "english_translation": ""} for word in words
    }

    print(f"Writing {output_path}...")
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)

    print(f"\nStatistics:")
    print(f"  Total words extracted: {len(words)}")
    print(f"  Output file: {output_path}")


if __name__ == "__main__":
    main()
