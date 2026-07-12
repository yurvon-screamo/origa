#!/usr/bin/env python3
"""Add honorific suppletive grammar rules (なさい / ください / いらっしゃい).

These verbs (為さる, 下さる, いらっしゃる) have irregular imperative forms that
do NOT follow the regular godan imperative paradigm (為され / 下され /
いらっしゃれ). They are matched by lemma via the suppletive FormatAction
variants `VerbToNasai` / `VerbToKudasai` / `VerbToIrasshai` (defined in
`origa/src/dictionary/grammar.rs` and implemented in
`origa/src/domain/grammar/forms_verb/conjugations.rs`).

A wrong lemma returns Err from the variant; `find_format_map_matches` then
filters the rule out via `formatted_rule_matches_text`'s Err-as-no-match
(`origa/src/domain/grammar/mod.rs:195`).

The corpus audit POS verification gate confirmed the surface tokens
(なさい / ください / いらっしゃい) are all classified by Lindera as Verb, so the
format_map key is `Verb`. ございます has 0 standalone occurrences in the
corpus and is intentionally not added.

Idempotent: re-running detects the rules by title and skips insertion.
Always writes a timestamped backup before overwriting (grammar.json is
gitignored production data).
"""
import json
import shutil
import time
from pathlib import Path

GRAMMAR_PATH = Path(__file__).resolve().parents[1] / "cdn" / "grammar" / "grammar.json"

HONORIFIC_RULES = [
    {
        "rule_id": "01KXAX0VRPNSR58CP87HJPXYMM",
        "title_en": "為さる→なさい (imperative polite)",
        "title_ru": "為さる→なさい (вежливый императив)",
        "short_description_en": "Polite imperative of the honorific verb なさる",
        "short_description_ru": "Вежливый императив уважительного глагола なさる",
        "format_action": "VerbToNasai",
    },
    {
        "rule_id": "01KXAX0VRPXYAP6KYN9P7CJTPA",
        "title_en": "下さる→ください (imperative polite)",
        "title_ru": "下さる→ください (вежливый императив)",
        "short_description_en": "Polite imperative of the honorific verb くださる",
        "short_description_ru": "Вежливый императив уважительного глагола くださる",
        "format_action": "VerbToKudasai",
    },
    {
        "rule_id": "01KXAX0VRP192A16Z4NJ2WK9GA",
        "title_en": "いらっしゃる→いらっしゃい (imperative greeting)",
        "title_ru": "いらっしゃる→いらっしゃい (императив-приветствие)",
        "short_description_en": "Imperative of the honorific verb いらっしゃる (used in greetings)",
        "short_description_ru": "Императив уважительного глагола いらっしゃる (в приветствиях)",
        "format_action": "VerbToIrasshai",
    },
]


def make_rule(spec):
    return {
        "rule_id": spec["rule_id"],
        "level": "N4",
        "content": {
            "English": {
                "title": spec["title_en"],
                "short_description": spec["short_description_en"],
                "explanation": (
                    f"This is a suppletive imperative form of the honorific verb. "
                    f"It is matched by lemma only; the FormatAction `{spec['format_action']}` "
                    "returns Err for any other verb."
                ),
                "how_to_form": "",
                "examples": "",
                "nuances": "",
                "pro_tip": "",
            },
            "Russian": {
                "title": spec["title_ru"],
                "short_description": spec["short_description_ru"],
                "explanation": (
                    "Супплетивная императивная форма уважительного глагола. "
                    "Сматчивается только по лемме; FormatAction "
                    f"`{spec['format_action']}` возвращает Err для любого другого глагола."
                ),
                "how_to_form": "",
                "examples": "",
                "nuances": "",
                "pro_tip": "",
            },
        },
        "format_map": {"Verb": [{spec["format_action"]: {}}]},
    }


def main():
    raw = GRAMMAR_PATH.read_text(encoding="utf-8")
    if raw.startswith("\ufeff"):
        raw = raw.lstrip("\ufeff")
    data = json.loads(raw)
    rules = data["grammar"]

    existing_titles = {r["content"]["English"]["title"] for r in rules}
    added = []
    for spec in HONORIFIC_RULES:
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
