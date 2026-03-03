import json
import os


def find_missing_words():
    well_known_dir = "well_known_set"
    vocabulary_dir = "dictionary/vocabulary"

    # 1. Collect all well-known words
    well_known_words = set()
    for root, dirs, files in os.walk(well_known_dir):
        for file in files:
            if file.endswith(".json") and file != "well_known_sets_meta.json":
                filepath = os.path.join(root, file)
                try:
                    with open(filepath, "r", encoding="utf-8") as f:
                        data = json.load(f)
                        if "words" in data:
                            well_known_words.update(data["words"])
                except Exception as e:
                    print(f"Error reading {filepath}: {e}")

    print(f"Total unique words in well_known_set: {len(well_known_words)}")

    # 2. Collect all dictionary keys
    dictionary_keys = set()
    for file in os.listdir(vocabulary_dir):
        if file.startswith("chunk_") and file.endswith(".json"):
            filepath = os.path.join(vocabulary_dir, file)
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    data = json.load(f)
                    dictionary_keys.update(data.keys())
            except Exception as e:
                print(f"Error reading {filepath}: {e}")

    print(f"Total unique words in dictionary: {len(dictionary_keys)}")

    # 3. Find missing words
    missing_words = sorted(list(well_known_words - dictionary_keys))

    print(f"Total missing words: {len(missing_words)}")

    # 4. Save missing words
    output_file = "well_known_missing_words.json"
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(missing_words, f, ensure_ascii=False, indent=2)

    print(f"Missing words saved to {output_file}")
    if missing_words:
        print("First 10 missing words:")
        for w in missing_words[:10]:
            print(f"  {w}")


if __name__ == "__main__":
    find_missing_words()
