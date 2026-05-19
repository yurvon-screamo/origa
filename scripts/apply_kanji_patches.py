#!/usr/bin/env python3
"""Apply patches to kanji.json from stdin (JSON array or JSON Lines)."""

import json
import sys
from pathlib import Path


KANJI_PATH = Path("cdn/dictionary/kanji.json")


def load_patches() -> list[dict]:
    """Load patches from stdin, supporting JSON array, single JSON object, or JSON Lines."""
    raw = sys.stdin.read().strip()
    if not raw:
        return []

    # Try as JSON array / single object
    try:
        parsed = json.loads(raw)
        if isinstance(parsed, dict):
            return [parsed]
        return parsed
    except json.JSONDecodeError:
        pass

    # Fall back to JSON Lines
    patches = []
    for line in raw.splitlines():
        line = line.strip()
        if line:
            patches.append(json.loads(line))
    return patches


def load_kanji_data() -> tuple[list[dict], dict[str, dict]]:
    """Load kanji.json and build a lookup by character."""
    with open(KANJI_PATH, "r", encoding="utf-8") as f:
        data = json.load(f)

    kanji_list = data["kanji"]
    lookup: dict[str, dict] = {}
    for entry in kanji_list:
        lookup[entry["kanji"]] = entry
    return kanji_list, lookup


def apply_patches(kanji_list: list[dict], lookup: dict[str, dict], patches: list[dict]) -> dict[str, int]:
    """Apply patches and return statistics."""
    stats = {"applied": 0, "skipped": 0, "not_found": 0, "mismatch": 0}

    # Deduplicate: group patches by kanji and merge issues
    kanji_patches: dict[str, list[dict]] = {}
    for patch in patches:
        kanji = patch["kanji"]
        if kanji not in kanji_patches:
            kanji_patches[kanji] = []
        kanji_patches[kanji].extend(patch["issues"])

    # Deduplicate issues within same kanji by (field, current, suggested)
    for kanji, issues in kanji_patches.items():
        seen = set()
        deduped = []
        for issue in issues:
            key = json.dumps(
                (issue["field"], issue["current"], issue["suggested"]),
                sort_keys=True,
                ensure_ascii=False,
            )
            if key not in seen:
                seen.add(key)
                deduped.append(issue)
        kanji_patches[kanji] = deduped

    for kanji, issues in kanji_patches.items():
        entry = lookup.get(kanji)
        if entry is None:
            print(f"Warning: kanji '{kanji}' not found in kanji.json — skipping", file=sys.stderr)
            stats["not_found"] += len(issues)
            continue

        for issue in issues:
            field = issue["field"]
            current = issue.get("current")
            suggested = issue.get("suggested")

            if field not in entry:
                print(
                    f"Warning: field '{field}' missing for kanji '{kanji}' — skipping",
                    file=sys.stderr,
                )
                stats["skipped"] += 1
                continue

            actual_value = entry[field]

            # For lists, check that current is a sub-multiset of actual_value.
            # Patches may specify only the problematic subset of entries.
            if isinstance(actual_value, list) and isinstance(current, list):
                remaining = list(actual_value)
                all_found = True
                for item in current:
                    try:
                        remaining.remove(item)
                    except ValueError:
                        all_found = False
                        break
                if not all_found:
                    print(
                        f"Warning: current value mismatch for kanji '{kanji}', "
                        f"field '{field}'. Expected subset {json.dumps(current)} "
                        f"not fully found in {json.dumps(actual_value)} — skipping",
                        file=sys.stderr,
                    )
                    stats["mismatch"] += 1
                    continue
            elif str(actual_value) != str(current):
                print(
                    f"Warning: current value mismatch for kanji '{kanji}', "
                    f"field '{field}'. Expected '{current}', "
                    f"got '{actual_value}' — skipping",
                    file=sys.stderr,
                )
                stats["mismatch"] += 1
                continue

            entry[field] = suggested
            stats["applied"] += 1

    return stats


def write_kanji_data(kanji_list: list[dict]) -> None:
    """Write updated kanji data back to file."""
    data = {"kanji": kanji_list}
    with open(KANJI_PATH, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
        f.write("\n")

    # Validate written file
    with open(KANJI_PATH, "r", encoding="utf-8") as f:
        json.load(f)
    print("Validation: kanji.json is valid JSON.", file=sys.stderr)


def run_quality_checks(kanji_list: list[dict]) -> dict:
    """Run quality checks on the kanji data."""
    results = {"empty_description_ru": 0, "empty_popular_words": 0}

    for entry in kanji_list:
        kanji = entry.get("kanji", "?")
        desc_ru = entry.get("description_ru", "")
        if isinstance(desc_ru, list):
            if not desc_ru:
                print(f"Warning: empty description_ru for '{kanji}'", file=sys.stderr)
                results["empty_description_ru"] += 1
        elif isinstance(desc_ru, str):
            if not desc_ru.strip():
                print(f"Warning: empty description_ru for '{kanji}'", file=sys.stderr)
                results["empty_description_ru"] += 1
        if not entry.get("popular_words"):
            print(f"Warning: empty popular_words for '{kanji}'", file=sys.stderr)
            results["empty_popular_words"] += 1

    return results


def main() -> None:
    patches = load_patches()
    if not patches:
        print("No patches provided.", file=sys.stderr)
        return

    kanji_list, lookup = load_kanji_data()
    stats = apply_patches(kanji_list, lookup, patches)
    write_kanji_data(kanji_list)

    qc = run_quality_checks(kanji_list)

    # Print statistics
    print(f"\nPatches applied:    {stats['applied']}")
    print(f"Patches skipped:    {stats['skipped']} (field missing)")
    print(f"Patches mismatched: {stats['mismatch']} (current value differs)")
    print(f"Kanji not found:    {stats['not_found']}")
    print(f"\nKanji entries total: {len(kanji_list)}")
    print(f"  with empty description_ru: {qc['empty_description_ru']}")
    print(f"  with empty popular_words:  {qc['empty_popular_words']}")


if __name__ == "__main__":
    main()
