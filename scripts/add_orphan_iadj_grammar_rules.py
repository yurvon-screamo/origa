#!/usr/bin/env python3
"""Add grammar rules for orphan i-adjective FormatAction variants.

Diagnostic on residual conjugation gaps (4 040 unlabeled real-conjugations)
revealed that 4 FormatAction variants are defined and implemented in code
(`origa/src/dictionary/grammar.rs`) but exposed by NO grammar.json rule:

  - `AdjectiveToGaru`     — i-adj stem + がる ("to feel/seem that way"):
                             怖い → 怖がる. The te-variant (怖がって) is matched
                             via `formatted_conjugation_variants`.
  - `AdjectiveToKunai`    — i-adj negative: 高い → 高くない.
  - `AdjectiveToKunakatta` — i-adj negative past: 高い → 高くなかった.
  - `AdjectiveToKereba`   — i-adj conditional: 高い → 高ければ.

Each new rule carries a format_map on IAdjective only. Verb/NaAdjective keys
are intentionally NOT added — the verb forms (negative, past, conditional)
are already covered by dedicated verb rules, and adding them here would
create over-matching via longest-format scoring.

Idempotent: re-running detects the rules by title and skips insertion.
Always writes a timestamped backup before overwriting (grammar.json is
gitignored production data).
"""
import json
import shutil
import time
from pathlib import Path

GRAMMAR_PATH = Path(__file__).resolve().parents[1] / "cdn" / "grammar" / "grammar.json"

RULES = [
    {
        "rule_id": "01KXB1CYH9BT21BNG51G47YJBH",
        "title_en": "～がる (i-adjective feeling)",
        "title_ru": "～がる (чувство и-прилагательного)",
        "short_description_en": "i-adjective stem + がる: to feel/seem that way",
        "short_description_ru": "основа и-прилагательного + がる: испытывать чувство",
        "format_action": "AdjectiveToGaru",
    },
    {
        "rule_id": "01KXB1CYH9ZDY4GMZW4FQVE7BH",
        "title_en": "～くない (i-adjective negative)",
        "title_ru": "～くない (отрицание и-прилагательного)",
        "short_description_en": "i-adjective negative form: 高い → 高くない",
        "short_description_ru": "отрицательная форма и-прилагательного: 高い → 高くない",
        "format_action": "AdjectiveToKunai",
    },
    {
        "rule_id": "01KXB1CYH9R2G7VEQ4TBFNJNQ5",
        "title_en": "～くなかった (i-adjective negative past)",
        "title_ru": "～くなかった (отрицательное прошедшее и-прилагательного)",
        "short_description_en": "i-adjective negative past: 高い → 高くなかった",
        "short_description_ru": "отрицательное прошедшее и-прилагательного: 高い → 高くなかった",
        "format_action": "AdjectiveToKunakatta",
    },
    {
        "rule_id": "01KXB1CYH9NJNT9XRBR9ER06XN",
        "title_en": "～ければ (i-adjective conditional)",
        "title_ru": "～ければ (условие и-прилагательного)",
        "short_description_en": "i-adjective conditional: 高い → 高ければ",
        "short_description_ru": "условная форма и-прилагательного: 高い → 高ければ",
        "format_action": "AdjectiveToKereba",
    },
]


def make_bilingual(short_en, short_ru):
    return {
        "English": {
            "title": "",
            "short_description": short_en,
            "explanation": "",
            "how_to_form": "",
            "examples": "",
            "nuances": "",
            "pro_tip": "",
        },
        "Russian": {
            "title": "",
            "short_description": short_ru,
            "explanation": "",
            "how_to_form": "",
            "examples": "",
            "nuances": "",
            "pro_tip": "",
        },
    }


def make_rule(spec):
    content = make_bilingual(spec["short_description_en"], spec["short_description_ru"])
    content["English"]["title"] = spec["title_en"]
    content["Russian"]["title"] = spec["title_ru"]
    return {
        "rule_id": spec["rule_id"],
        "level": "N4",
        "content": content,
        "format_map": {"IAdjective": [{spec["format_action"]: {}}]},
    }


def main():
    raw = GRAMMAR_PATH.read_text(encoding="utf-8")
    if raw.startswith("\ufeff"):
        raw = raw.lstrip("\ufeff")
    data = json.loads(raw)
    rules = data["grammar"]

    existing_titles = {r["content"]["English"]["title"] for r in rules}
    added = []
    for spec in RULES:
        if spec["title_en"] in existing_titles:
            print(f"rule「{spec['title_en']}」already present, skipping")
            continue
        rules.append(make_rule(spec))
        added.append(spec["title_en"])

    if not added:
        print("nothing to add")
        return

    backup_path = GRAMMAR_PATH.with_suffix(
        f".json.bak.{time.strftime('%Y%m%d-%H%M%S')}"
    )
    shutil.copy2(GRAMMAR_PATH, backup_path)

    out = json.dumps(data, ensure_ascii=False, indent=2)
    GRAMMAR_PATH.write_text(out, encoding="utf-8")
    print(f"backup: {backup_path.name}")
    print(f"added: {added}")
    print(f"total rules now: {len(rules)}")


if __name__ == "__main__":
    main()
