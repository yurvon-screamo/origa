import json
from pathlib import Path

BASE_DIR = (
    Path(__file__).parent.parent / "origa_ui" / "public" / "domain" / "well_known_set"
)
OUTPUT_FILE = BASE_DIR / "well_known_sets_meta.json"


def load_json(path: Path) -> dict:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def extract_meta(data: dict, set_id: str, set_type: str) -> dict:
    content = data.get("content", {})
    russian = content.get("Russian", {})
    english = content.get("English", {})

    return {
        "id": set_id,
        "set_type": set_type,
        "level": data.get("level", "N5"),
        "title_ru": russian.get("title", ""),
        "title_en": english.get("title", ""),
        "desc_ru": russian.get("description", ""),
        "desc_en": english.get("description", ""),
    }


def main():
    meta_list = []

    jlpt_files = [
        ("jltp_n5.json", "jlpt_n5", "N5"),
        ("jltp_n4.json", "jlpt_n4", "N4"),
        ("jltp_n3.json", "jlpt_n3", "N3"),
        ("jltp_n2.json", "jlpt_n2", "N2"),
        ("jltp_n1.json", "jlpt_n1", "N1"),
    ]

    for filename, set_id, level in jlpt_files:
        path = BASE_DIR / filename
        if path.exists():
            data = load_json(path)
            data["level"] = level
            meta_list.append(extract_meta(data, set_id, "Jlpt"))

    migii_configs = [
        ("n5", 20, "N5"),
        ("n4", 11, "N4"),
        ("n3", 31, "N3"),
        ("n2", 31, "N2"),
        ("n1", 56, "N1"),
    ]

    for level_id, count, level in migii_configs:
        for i in range(1, count + 1):
            filename = f"migii/{level_id}/migii_{level_id}_{i}.json"
            path = BASE_DIR / filename
            if path.exists():
                data = load_json(path)
                data["level"] = level
                set_id = f"migii_{level_id}_{i}"
                meta_list.append(extract_meta(data, set_id, "Migii"))

    spy_family_dir = BASE_DIR / "spy_family"
    if spy_family_dir.exists():
        for path in sorted(spy_family_dir.glob("*.json")):
            data = load_json(path)
            set_id = path.stem
            meta_list.append(extract_meta(data, set_id, "SpyFamily"))

    duolingo_dir = BASE_DIR / "duolingo"
    if duolingo_dir.exists():
        for path in sorted(duolingo_dir.rglob("*.json")):
            data = load_json(path)
            set_id = f"duolingo_{path.parent.name}_{path.stem}"
            meta_list.append(extract_meta(data, set_id, "Duolingo"))

    meta_list.sort(key=lambda x: x["id"])

    with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
        json.dump(meta_list, f, ensure_ascii=False, indent=2)

    print(f"Generated {len(meta_list)} meta entries to {OUTPUT_FILE}")


if __name__ == "__main__":
    main()
