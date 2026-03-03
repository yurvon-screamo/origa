import os


def count_lines(filepath):
    try:
        with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
            return sum(1 for _ in f)
    except Exception as e:
        print(f"Error reading {filepath}: {e}")
        return 0


def scan_directory(directory, threshold):
    long_files = []
    for root, dirs, files in os.walk(directory):
        # Skip common directories that might contain many files or non-source code
        if "node_modules" in dirs:
            dirs.remove("node_modules")
        if "target" in dirs:
            dirs.remove("target")
        if "dist" in dirs:
            dirs.remove("dist")

        source_extensions = {".rs", ".js", ".ts", ".py", ".css", ".html", ".toml"}

        for file in files:
            if not any(file.endswith(ext) for ext in source_extensions):
                continue

            filepath = os.path.join(root, file)
            line_count = count_lines(filepath)
            if line_count > threshold:
                long_files.append((filepath, line_count))

    return sorted(long_files, key=lambda x: x[1], reverse=True)


if __name__ == "__main__":
    target_dir = "origa_ui\src\pages"
    limit = 120

    print(f"Scanning directory: {target_dir}")
    print(f"Threshold: {limit} lines\n")

    results = scan_directory(target_dir, limit)

    if results:
        print(f"{'Line Count':<12} | {'File Path'}")
        print("-" * 60)
        for filepath, count in results:
            print(f"{count:<12} | {filepath}")
        print(f"\nTotal files found: {len(results)}")
    else:
        print("No files found exceeding the threshold.")
