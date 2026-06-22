"""Apply curated polysemic-kanji description fixes to cdn/dictionary/kanji.json.

Context: 字 ("character" in English) was rendered as RU "характер" (personality),
which captures only one English sense and is wrong for the kanji sense (letter /
symbol / sign). Similar LLM mistranslations exist for ~21 kanji where the English
gloss is polysemic and the Russian translator picked the wrong sense.

Run:
    python scripts/fix_polysemic_kanji.py --kanji cdn/dictionary/kanji.json
    python scripts/fix_polysemic_kanji.py --kanji cdn/dictionary/kanji.json --dry-run
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# Curated, hand-verified fixes. Each value is a (description_en, description_ru)
# pair derived from kanjidic / jisho.org / standard Russian japanist lexicons.
# Only entries where the previous RU gloss was the wrong sense of a polysemic
# EN word are listed — most kanji are already correct and intentionally omitted.
POLYSEMIC_FIXES: dict[str, tuple[list[str], list[str]]] = {
    # 字: EN "character" (letter vs personality) → RU previously "характер" (wrong)
    "字": (["character", "letter", "symbol"], ["знак", "символ", "буква", "иероглиф"]),
    # 点: EN "point" (vs "place") → RU previously "место" (place — wrong)
    "点": (["point", "dot", "score"], ["точка", "пункт", "капля", "немного"]),
    # 州: EN "state" (administrative vs condition) → RU previously "состояние" (condition — wrong)
    "州": (["state", "province"], ["штат", "провинция", "область"]),
    # 津: EN "port" (ferry/harbor vs input-hole) → RU previously "входное отверстие" (wrong)
    "津": (["port", "ferry crossing", "harbor"], ["переправа", "порт", "гавань", "брод"]),
    # 軽: EN "light" (weight vs illumination) → RU previously "свет" (illumination — wrong)
    "軽": (["light (weight)", "easy"], ["лёгкий", "легковесный", "несерьёзный"]),
    # 封: EN "seal" (envelope vs animal) → RU previously "тюлень" (the animal — wrong)
    "封": (["seal", "envelope"], ["печать", "запечатывать", "конверт"]),
    # 箋: EN "note" (slip of paper vs musical/composition) → RU previously "композиция" (wrong)
    "箋": (["note", "letter", "label"], ["записка", "письмо", "ярлык"]),
    # 底: EN "bottom" (noun vs adjective) → RU previously "нижний" (adjective — wrong)
    "底": (["bottom", "base"], ["дно", "основание", "основа", "низ"]),
    # 幹: EN "trunk" (tree vs main-part) → RU previously "основная часть" (too vague)
    "幹": (["trunk (tree)", "cadres"], ["ствол", "кадры", "основной"]),
    # 丁: EN "block" (city block vs verb to block) → RU previously "блокировать" (verb — wrong)
    "丁": (["city block", "piece", "fourth"], ["квартал", "кусочек", "четвёрка"]),
    # 胞: EN "cell" (biological vs placenta-only) → RU previously "плацента" (too narrow)
    "胞": (["cell", "placenta", "sibling"], ["клетка", "послед", "брат или сестра"]),
    # 衷: EN "heart" (inner feelings vs secret) → RU previously "сокровенный" (wrong sense)
    "衷": (["inner heart", "feelings"], ["внутренние чувства", "сердце", "искренний"]),
    # 玩: EN "play" (verb vs toy-noun) → RU previously "игрушка" (toy — wrong)
    "玩": (["to play with", "enjoy"], ["играть", "забавляться", "наслаждаться"]),
    # 彩: EN "color" (noun vs verb) → RU previously "раскрасить" (verb — wrong)
    "彩": (["color", "painting"], ["цвет", "окраска", "колорит"]),
    # 植: EN "plant" (verb vs noun-only) → RU previously "растение" (noun-only — incomplete)
    "植": (["to plant", "plant"], ["сажать", "насаждать", "растение"]),
    # 申: EN "state" (to state vs noun) → RU previously "сказать" (right verb sense, but EN ambiguous)
    "申": (["to say (humble)", "monkey (zodiac)"], ["говорить (вежливо)", "обезьяна (зодиак)"]),
    # 述: EN "state" (to state) → RU previously "сказать" (too generic for 述=to expound)
    "述": (["to state", "to expound"], ["излагать", "высказывать", "рассказывать"]),
    # 候: EN "season" (narrow) → RU previously "климат" (narrow) — broaden to season/weather/climate
    "候": (["season", "weather", "climate"], ["сезон", "погода", "климат", "ожидать"]),
    # 兆: EN "sign" (narrow) → RU previously "знак" — broaden with omen/trillion senses
    "兆": (["sign", "omen", "trillion"], ["знак", "признак", "предзнаменование", "триллион"]),
    # 符: EN "sign" (narrow) → RU previously "знак" — broaden with talisman/token senses
    "符": (["sign", "token", "talisman"], ["знак", "талисман", "жетон", "соответствие"]),
    # 房: EN "room" (narrow) → RU previously "комната" — broaden with chamber/branch senses
    "房": (["room", "chamber"], ["комната", "покой", "ответвление", "секция"]),
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--kanji",
        required=True,
        help="Path to cdn/dictionary/kanji.json",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show which entries would be updated without writing",
    )
    return parser.parse_args()


def load_kanji(path: Path) -> dict:
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def apply_fixes(data: dict) -> list[tuple[str, list[str], list[str], list[str], list[str]]]:
    """Apply fixes in place. Returns list of (kanji, old_en, new_en, old_ru, new_ru)."""
    kanji_list = data.get("kanji", [])
    changed: list[tuple[str, list[str], list[str], list[str], list[str]]] = []
    seen: set[str] = set()
    for entry in kanji_list:
        literal = entry.get("kanji")
        if literal not in POLYSEMIC_FIXES:
            continue
        new_en, new_ru = POLYSEMIC_FIXES[literal]
        old_en = list(entry.get("description_en", []))
        old_ru = list(entry.get("description_ru", []))
        if old_en == new_en and old_ru == new_ru:
            seen.add(literal)
            continue
        entry["description_en"] = new_en
        entry["description_ru"] = new_ru
        changed.append((literal, old_en, new_en, old_ru, new_ru))
        seen.add(literal)
    missing = set(POLYSEMIC_FIXES) - seen
    if missing:
        print(f"WARNING: {len(missing)} kanji from fix-table not found in data: {sorted(missing)}")
    return changed


def main() -> int:
    args = parse_args()
    path = Path(args.kanji)
    if not path.exists():
        print(f"Error: {path} not found")
        return 1
    data = load_kanji(path)
    changed = apply_fixes(data)
    if not changed:
        print("No changes needed — all fixes already applied.")
        return 0
    print(f"Applied {len(changed)} polysemic-kanji fixes:")
    for literal, old_en, new_en, old_ru, new_ru in changed:
        print(f"  {literal}: EN {old_en} -> {new_en}")
        print(f"         RU {old_ru} -> {new_ru}")
    if args.dry_run:
        print("\n--dry-run: no files modified.")
        return 0
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, separators=(",", ":"))
    print(f"\nWrote {len(changed)} fixes to {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
