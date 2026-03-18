import json
from pathlib import Path


def main():
    project_root = Path(__file__).parent.parent

    translations_path = project_root / "scripts" / "missing_words_for_translation.json"
    dictionary_path = (
        project_root
        / "origa_ui"
        / "public"
        / "dictionary"
        / "vocabulary"
        / "chunk_11.json"
    )

    print(f"Reading translations from {translations_path}...")
    with open(translations_path, "r", encoding="utf-8") as f:
        translations = json.load(f)

    print(f"Reading dictionary from {dictionary_path}...")
    with open(dictionary_path, "r", encoding="utf-8-sig") as f:
        dictionary = json.load(f)

    words_with_both = 0
    words_missing_russian = 0
    words_missing_english = 0
    words_already_exist = 0
    words_added = 0

    for word, trans in translations.items():
        russian = trans.get("russian_translation", "").strip()
        english = trans.get("english_translation", "").strip()

        if not russian:
            words_missing_russian += 1
            continue
        if not english:
            words_missing_english += 1
            continue

        words_with_both += 1

        if word in dictionary:
            words_already_exist += 1
            continue

        dictionary[word] = {
            "russian_translation": russian,
            "english_translation": english,
        }
        words_added += 1

    print(f"\nSaving updated dictionary to {dictionary_path}...")
    with open(dictionary_path, "w", encoding="utf-8") as f:
        json.dump(dictionary, f, ensure_ascii=False, indent=2)

    print(f"\nStatistics:")
    print(f"  Total words in translation file: {len(translations)}")
    print(f"  Words with both translations: {words_with_both}")
    print(f"  Words missing Russian translation: {words_missing_russian}")
    print(f"  Words missing English translation: {words_missing_english}")
    print(f"  Words already in dictionary: {words_already_exist}")
    print(f"  Words added to dictionary: {words_added}")
    print(f"  Total words in dictionary now: {len(dictionary)}")


if __name__ == "__main__":
    main()
