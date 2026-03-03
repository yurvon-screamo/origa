import json
import os
import subprocess
import tempfile
from pathlib import Path


def run_tokenizer(words):
    """
    Runs the Rust tokenizer tool on the provided words to get unique vocabulary base forms.
    """
    if not words:
        return []

    # Use a temporary file to pass the words to the tokenizer
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".txt", encoding="utf-8", delete=False
    ) as f:
        f.write("\n".join(words))
        temp_path = f.name

    try:
        # Run the tokenizer from the root of the repo using the built binary directly
        # to avoid cargo's startup overhead.
        root_dir = Path(__file__).parent.parent
        binary_path = root_dir / "target" / "debug" / "tokenizer.exe"
        if not binary_path.exists():
            # Try Unix variant just in case
            binary_path = root_dir / "target" / "debug" / "tokenizer"

        cmd = [str(binary_path), "-f", temp_path]
        result = subprocess.run(
            cmd, capture_output=True, text=True, encoding="utf-8", cwd=str(root_dir)
        )

        if result.returncode != 0:
            print(f"Error tokenizing: {result.stderr}", flush=True)
            return []

        # Tokenizer returns space-separated words
        output_words = result.stdout.strip().split(" ")
        return sorted(list(set(w for w in output_words if w)))
    finally:
        os.unlink(temp_path)


def process_file(file_path):
    print(f"Processing {file_path}...", end=" ", flush=True)
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        if "words" not in data:
            print("Skipping: no 'words' key found.", flush=True)
            return

        original_words = data["words"]
        tokenized_words = run_tokenizer(original_words)

        if tokenized_words:
            data["words"] = tokenized_words
            with open(file_path, "w", encoding="utf-8", newline="\n") as f:
                json.dump(data, f, ensure_ascii=False, indent="\t")
            print(
                f"Updated: {len(original_words)} -> {len(tokenized_words)} words.",
                flush=True,
            )
        else:
            print("Warning: Tokenizer returned no words.", flush=True)

    except Exception as e:
        print(f"Error processing {file_path}: {e}", flush=True)


def main():
    root_dir = Path(__file__).parent.parent
    well_known_set_dir = root_dir / "origa_ui" / "public" / "domain" / "well_known_set"

    # Walk through all json files in the directory and subdirectories
    for json_file in sorted(list(well_known_set_dir.rglob("*.json"))):
        # Skip the metadata file
        if json_file.name == "well_known_sets_meta.json":
            continue

        process_file(json_file)


if __name__ == "__main__":
    main()
