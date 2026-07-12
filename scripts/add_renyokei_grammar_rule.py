#!/usr/bin/env python3
"""Add the masu-stem (ren'yōkei) continuative grammar rule.

This rule carries ONLY content (title/short_description/explanation/...) — it
deliberately has NO format_map. The reason: a format_map entry with
{Verb: [VerbToStem]} would over-match in Path B (`detect_format_map_rules` →
`enrich_phrases_with_grammar`) because `format(食べる)="食べ"` and
`text.contains("食べ")` is true for 食べます/食べて/食べた.

The rule is matched only by the special-case resolver `resolve_masu_stem_match`
in `translation.rs` (analogous to how `resolve_sou_da_match` matches the
そうだ rule by title without going through format_map). Without a format_map,
`find_format_map_matches` filters it out via `rule.has_format_map()`, so
Path B is unaffected.

Idempotent: re-running detects the rule by title and skips insertion. Always
writes a timestamped backup before overwriting (grammar.json is gitignored
production data).
"""
import json
import shutil
import time
from pathlib import Path

GRAMMAR_PATH = Path(__file__).resolve().parents[1] / "cdn" / "grammar" / "grammar.json"

REN_TITLE_EN = "～連用形 (continuative/noun)"
REN_TITLE_RU = "～連用形 (continuative/noun)"
REN_RULE_ID = "01KXAEW17E1B4N421Y8PCPEYHK"


def make_rule():
    en = {
        "title": REN_TITLE_EN,
        "short_description": "Verb masu-stem used as continuative or noun",
        "explanation": (
            "The ren'yōkei (連用形, masu-stem) of a verb has two standalone uses "
            "when it is NOT followed by an auxiliary (ます/て/た/ば/...): it can "
            "join clauses as a continuative (走り、跳ぶ) or function as a noun "
            "(願い=request, 暮らし=living, 話=story). When the stem appears on its "
            "own — not as the base of another conjugation — it carries this label."
        ),
        "how_to_form": (
            "| Group | Rule | Example |\n"
            "|-------|------|---------|\n"
            "| Group 1 (う-verbs) | Replace final う-row sound with い-row | 行く → 行き |\n"
            "| Group 2 (る-verbs) | Remove る | 食べる → 食べ |\n"
            "| Group 3 (irregular) | する → し / くる → き | する → し |"
        ),
        "examples": (
            "```\n返事を願いします。\nI ask for a reply.\n```\n\n"
            "```\n暮らしは楽しい。\nLife is enjoyable.\n```\n\n"
            "```\n話を聞かせて。\nLet me hear the story.\n```"
        ),
        "nuances": (
            "- ❌ Treating a stem followed by ます/て/た as continuative → ✅ Those "
            "are polite/te/past forms covered by their own rules.\n"
            "- 🔄 Many stems have lexicalized as independent nouns (願い, 暮らし, "
            "話) — both readings are valid; the noun entry gives the translation."
        ),
        "pro_tip": (
            "If a verb stem stands on its own (next token is NOT ます/て/た/ば), "
            "it is either continuing the previous clause or has been noun-ified. "
            "Look up the stem in the dictionary for the noun meaning."
        ),
    }
    ru = {
        "title": REN_TITLE_RU,
        "short_description": "Masu-основа глагола как деепричастие или существительное",
        "explanation": (
            "Ren'yōkei (連用形, masu-основа) глагола имеет два самостоятельных "
            "употребления, когда за ней НЕ следует вспомогательный глагол "
            "(ます/て/た/ば/...): соединение предложений как деепричастие "
            "(走り、跳ぶ) или функционирование как существительное "
            "(願い=просьба, 暮らし=жизнь, 話=рассказ). Когда основа стоит "
            "отдельно — не как база другой спряжённой формы — она несёт эту метку."
        ),
        "how_to_form": (
            "| Группа | Правило | Пример |\n"
            "|--------|---------|--------|\n"
            "| Группа 1 (う-глаголы) | Заменить последний звук у-ряда на и-ряд | 行く → 行き |\n"
            "| Группа 2 (る-глаголы) | Убрать る | 食べる → 食べ |\n"
            "| Группа 3 (исключения) | する → し / くる → き | する → し |"
        ),
        "examples": (
            "```\n返事を願いします。\nЯ прошу ответа.\n```\n\n"
            "```\n暮らしは楽しい。\nЖизнь приятна.\n```\n\n"
            "```\n話を聞かせて。\nДай мне послушать рассказ.\n```"
        ),
        "nuances": (
            "- ❌ Трактовать основу перед ます/て/た как деепричастие → ✅ Это "
            "вежливая/te/прошедшая формы, у них свои правила.\n"
            "- 🔄 Многие основы лексикализировались как самостоятельные "
            "существительные (願い, 暮らし, 話) — оба прочтения допустимы; "
            "словарная статья существительного даёт перевод."
        ),
        "pro_tip": (
            "Если masu-основа глагола стоит отдельно (следующий токен НЕ "
            "ます/て/た/ば), она либо продолжает предыдущее предложение, либо "
            "превратилась в существительное. Поищи основу в словаре для "
            "значения-существительного."
        ),
    }
    return {
        "rule_id": REN_RULE_ID,
        "level": "N4",
        "content": {"English": en, "Russian": ru},
    }


def main():
    raw = GRAMMAR_PATH.read_text(encoding="utf-8")
    if raw.startswith("\ufeff"):
        raw = raw.lstrip("\ufeff")
    data = json.loads(raw)
    rules = data["grammar"]

    existing_titles = {r["content"]["English"]["title"] for r in rules}
    if REN_TITLE_EN in existing_titles:
        print(f"rule「{REN_TITLE_EN}」already present, skipping")
        return

    rules.append(make_rule())

    backup_path = GRAMMAR_PATH.with_suffix(
        f".json.bak.{time.strftime('%Y%m%d-%H%M%S')}"
    )
    shutil.copy2(GRAMMAR_PATH, backup_path)

    out = json.dumps(data, ensure_ascii=False, indent=2)
    GRAMMAR_PATH.write_text(out, encoding="utf-8")
    print(f"backup: {backup_path.name}")
    print(f"added rule「{REN_TITLE_EN}」 (id={REN_RULE_ID})")
    print(f"total rules now: {len(rules)}")


if __name__ == "__main__":
    main()
