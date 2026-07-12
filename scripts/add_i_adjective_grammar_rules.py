#!/usr/bin/env python3
"""Add grammar rules for uncovered i-adjective conjugations.

Adds 3 rules to cdn/grammar/grammar.json:
  1. New rule "～く (adverbial)" — i-adjective ku-form (AdjectiveToKu).
  2. New rule "～かった (past)" — i-adjective past (AdjectiveToKatta).
  3. Patches existing rule "～て／～くて／～で" with a format_map so its
     i-adjective (AdjectiveToKute) and na-adjective (AdjectiveToDe) branches
     finally match (the rule existed but was dead — no format_map).

Verb format_map is intentionally NOT added to the te-form rule: verb te-form
is already covered by 37 other rules, and adding it here would create
over-matching and shift the grammar_label selected by the resolver's
longest-format scoring, breaking existing verb-te tests.

Idempotent: re-running detects the new rules by title and skips insertion.
"""
import json
import shutil
import sys
from pathlib import Path

GRAMMAR_PATH = Path(__file__).resolve().parents[1] / "cdn" / "grammar" / "grammar.json"

KU_TITLE_EN = "～く (adverbial)"
KU_TITLE_RU = "～く (наречие)"
KATTA_TITLE_EN = "～かった (past)"
KATTA_TITLE_RU = "～かった (прошедшее)"
TE_TITLE_EN_MARKER = "～て／～くて／～で"

# The exact format_map the te-form rule must end up carrying. Used both for
# applying the patch and for verifying it on re-runs (precise idempotency).
TE_EXPECTED_FORMAT_MAP = {
    "IAdjective": [{"AdjectiveToKute": {}}],
    "NaAdjective": [{"AdjectiveToDe": {}}],
}

KU_RULE_ID = "01KX9DBYXX0NSA9SA7SJ2WWKWV"
KATTA_RULE_ID = "01KX9DBYXXMTPRG5ARM7CT0TRW"


def make_rule(rule_id, level, en, ru, format_map=None, keywords=None):
    rule = {
        "rule_id": rule_id,
        "level": level,
        "content": {"English": en, "Russian": ru},
    }
    if format_map is not None:
        rule["format_map"] = format_map
    if keywords is not None:
        rule["keywords"] = keywords
    return rule


def main():
    raw = GRAMMAR_PATH.read_text(encoding="utf-8")
    if raw.startswith("\ufeff"):
        raw = raw.lstrip("\ufeff")
    data = json.loads(raw)
    rules = data["grammar"]

    existing_titles = {
        r["content"]["English"]["title"] for r in rules
    }

    added = []

    # 1. i-adjective ku-form (adverbial)
    if KU_TITLE_EN not in existing_titles:
        rules.append(make_rule(
            KU_RULE_ID,
            "N5",
            {
                "title": KU_TITLE_EN,
                "short_description": "I-adjective adverbial form",
                "explanation": (
                    "Dropping the final `い` of an i-adjective and adding `く` "
                    "turns it into an adverb that modifies a following verb. "
                    "This is the adverbial (連用形) form."
                ),
                "how_to_form": (
                    "| Word type | Rule | Example |\n"
                    "|-----------|------|---------|\n"
                    "| I-adjectives | Remove い → く | 早い → 早く |\n"
                    "| Exception: いい | よく | いい → よく |"
                ),
                "examples": (
                    "```\n早く歩いてください。\nPlease walk quickly.\n```\n\n"
                    "```\nたくさん食べました。\nI ate a lot.\n```"
                ),
                "nuances": (
                    "- ❌ Leaving the `い` and adding `く` (早い速く) → ✅ 早く\n"
                    "- 🔄 いい (good) irregularly becomes よく, not いく"
                ),
                "pro_tip": (
                    "The ku-form is the bridge from adjectives to adverbs. "
                    "Most i-adjectives behave regularly; only いい → よく is irregular."
                ),
            },
            {
                "title": KU_TITLE_RU,
                "short_description": "Наречная форма и-прилагательного",
                "explanation": (
                    "Убрав конечное `い` у и-прилагательного и добавив `く`, "
                    "получаем наречие, которое определяет последующий глагол. "
                    "Это наречная (連用形) форма."
                ),
                "how_to_form": (
                    "| Тип слова | Правило | Пример |\n"
                    "|-----------|---------|--------|\n"
                    "| И-прилагательные | Убрать い → く | 早い → 早く |\n"
                    "| Исключение: いい | よく | いい → よく |"
                ),
                "examples": (
                    "```\n早く歩いてください。\nПожалуйста, идите быстрее.\n```\n\n"
                    "```\nたくさん食べました。\nЯ много съел.\n```"
                ),
                "nuances": (
                    "- ❌ Оставлять `い` и добавлять `く` (早い速く) → ✅ 早く\n"
                    "- 🔄 いい (хороший) неправильно становится よく, а не いく"
                ),
                "pro_tip": (
                    "Ku-форма — мост от прилагательных к наречиям. "
                    "Большинство и-прилагательных регулярны; только いい → よく — исключение."
                ),
            },
            format_map={"IAdjective": [{"AdjectiveToKu": {}}]},
        ))
        added.append(KU_TITLE_EN)

    # 2. i-adjective past (katta-form)
    if KATTA_TITLE_EN not in existing_titles:
        rules.append(make_rule(
            KATTA_RULE_ID,
            "N5",
            {
                "title": KATTA_TITLE_EN,
                "short_description": "I-adjective past tense",
                "explanation": (
                    "To express the past tense of an i-adjective, drop the final "
                    "`い` and add `かった`. The negative past is `～なかった`."
                ),
                "how_to_form": (
                    "| Form | Rule | Example |\n"
                    "|------|------|---------|\n"
                    "| Affirmative past | Remove い → かった | 高い → 高かった |\n"
                    "| Negative past | Remove い → なかった | 高い → 高くなかった |"
                ),
                "examples": (
                    "```\n昨日の天気は良かった。\nYesterday's weather was good.\n```\n\n"
                    "```\nこの本は面白くなかった。\nThis book was not interesting.\n```"
                ),
                "nuances": (
                    "- ❌ Adding だった to an i-adjective (高いだった) → ✅ 高かった\n"
                    "- 🔄 いい → よかった (irregular past, not いかった)"
                ),
                "pro_tip": (
                    "The katta-form is the i-adjective equivalent of the verb た-form. "
                    "Remember いい → よかった as the only common irregular."
                ),
            },
            {
                "title": KATTA_TITLE_RU,
                "short_description": "Прошедшее время и-прилагательного",
                "explanation": (
                    "Чтобы выразить прошедшее время и-прилагательного, нужно убрать "
                    "конечное `い` и добавить `かった`. Отрицательное прошедшее — `～なかった`."
                ),
                "how_to_form": (
                    "| Форма | Правило | Пример |\n"
                    "|-------|---------|--------|\n"
                    "| Утвердительное прошедшее | Убрать い → かった | 高い → 高かった |\n"
                    "| Отрицательное прошедшее | Убрать い → なかった | 高い → 高くなかった |"
                ),
                "examples": (
                    "```\n昨日の天気は良かった。\nВчера погода была хорошей.\n```\n\n"
                    "```\nこの本は面白くなかった。\nЭта книга была неинтересной.\n```"
                ),
                "nuances": (
                    "- ❌ Добавлять だった к и-прилагательному (高いだった) → ✅ 高かった\n"
                    "- 🔄 いい → よかった (неправильное прошедшее, не いかった)"
                ),
                "pro_tip": (
                    "Katta-форма — аналог た-формы глагола для и-прилагательных. "
                    "Запомни いい → よかった как единственное частое исключение."
                ),
            },
            format_map={"IAdjective": [{"AdjectiveToKatta": {}}]},
        ))
        added.append(KATTA_TITLE_EN)

    # 3. Patch existing ～て／～くて／～で rule with an adjective-only format_map.
    te_rule = next(
        (r for r in rules if r["content"]["English"]["title"] == TE_TITLE_EN_MARKER),
        None,
    )
    if te_rule is None:
        print(f"ERROR: te-form rule {TE_TITLE_EN_MARKER!r} not found", file=sys.stderr)
        sys.exit(1)
    if te_rule.get("format_map") == TE_EXPECTED_FORMAT_MAP:
        print("te-form rule already carries the expected format_map, skipping patch")
    else:
        # IAdjective + NaAdjective only; verb te-form stays covered by other rules.
        te_rule["format_map"] = TE_EXPECTED_FORMAT_MAP
        added.append(f"(patched format_map on {TE_TITLE_EN_MARKER})")

    # Always create a timestamped backup before overwriting — grammar.json is
    # gitignored production data, so this script is the only mutation point.
    backup_path = GRAMMAR_PATH.with_suffix(
        f".json.bak.{__import__('time').strftime('%Y%m%d-%H%M%S')}"
    )
    shutil.copy2(GRAMMAR_PATH, backup_path)

    out = json.dumps(data, ensure_ascii=False, indent=2)
    GRAMMAR_PATH.write_text(out, encoding="utf-8")
    print(f"backup written: {backup_path.name}")
    print(f"added/patched: {added}")
    print(f"total rules now: {len(rules)}")


if __name__ == "__main__":
    main()
