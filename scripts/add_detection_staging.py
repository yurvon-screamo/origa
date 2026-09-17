#!/usr/bin/env python3
"""Add keyword detection to previously undetectable rules (DATA stage).

Keywords semantics (detect_keyword_rules): list of groups; EVERY group
must have at least one keyword present in the text (AND across groups,
OR within a group). Single frequent kana (も, を, は) can never be a
keyword — substring matching would fire on most texts — so bare
particle/abc rules stay undetectable by design and rely on the judge +
manual review in the generation pipeline.

Decisions per rule are hand-made (N1 bookish constructions carry unique
markers; N5 gets compound AND-groups like へ+行く). Conflicts with
existing keywords are avoided (～こそ already owns ['こそ']).

Applies to the STAGING corpus and writes the map for review.
"""

import json
from pathlib import Path

STAGING = Path("/tmp/opencode/staging")
CORPUS = STAGING / "grammar_v2.pruned.json"

KEYWORDS = {
    # ── N1: bookish constructions with unique markers ──
    "01M1FD71RYNH2BZZ080AND78FB": [["うが"]],  # ～うが～うが
    "01M1FD71S17JD9Y0Y5QGK7XJS4": [["どんなに"], ["うが"]],  # どんなに～うが
    "01M1FD71S27MGMXBKZ45EY8PS5": [["であれ"]],  # ～であれ～であれ
    "01M1FD71S35D6WR31AHGR93XFA": [["にしろ", "にせよ"]],  # ～にしろ～にしろ
    "01M1FD71S5VXVT6PTED9JJNPAM": [["かれ"]],  # ～かれ～かれ
    "01M1FD71S6DJZZG7AAPB2FZ5GK": [["だの"]],  # ～だの～だの
    "01M1FD71S739WF8H2SX070Y473": [["といい"]],  # ～といい～といい
    "01M1FD71S96RD5PETH6ACMDZJC": [["につけ"]],  # ～につけ～につけ
    "01M1FD71SBF65XVHENMTH5ZCQ4": [["うと"], ["まいと"]],  # ～ようと～まいと (волитив любой группы: 降ろうと含ает うと)
    "01M1FD71SV4NP8XVQCQKQ2AXQ6": [["からする"]],  # ～からする
    "01M1FD71SW7DKDQGTY8BWRMGXF": [["たりとも"], ["ない", "ません"]],  # ～たりとも～ない
    "01M1FD71TEHEWKTTF3FMG2T8VW": [["べく"]],  # ～べく
    "01M1FD71VAFB9PBX3J0HJ5Q7TT": [["たら"], ["たで"]],  # ～たら～たで
    "01M1FD71VGNVBBXDQX1MHA7K30": [["がまま"]],  # ～がままに
    "01M1FD71W452F71VX963TNKJ2F": [["にあって"]],  # ～にあって
    "01M1FD71W61QV07XXD1Q1DFXCG": [["にして"]],  # ～にして
    "01M1FD71X2HKAE2T754C7ESQDW": [["ものを"]],  # ～ものを
    "01M1H04XBT4AR17G7K72WYBMN4": [["ほどの"]],  # ～ほどの
    "01M1H04XC9FVHMSXF54GBGDXWZ": [["ところを"]],  # ～ところを
    "01M1H04XCE7Q4DVQAF9X826783": [["もなんでもない", "でもなんでもない"]],  # ～（で）もなんでもない
    "01M1H04XCQVY1KS1DRHAGZ7NPA": [["のみ"]],  # ～のみ
    "01M1H04XCSAGCR1AJ3VTQ697W2": [["重ねて", "重ねた", "重ね"]],  # ～に～を重ねて
    "01M1H04XCVYWYEX8G0VJYM57WE": [["当の"]],  # 当の～
    "01M1H04XD117T9V4TCSBHNW9YJ": [["もしない", "もしなかっ", "もせず", "たりもしない"]],  # ～もしない
    "01M1H04XD44FSE5DD033SYW40N": [["抜きで", "抜きに"]],  # ～抜きで（は）／～抜きに
    "01M1H04XD725MCVG9WW6431BBS": [["を抜い", "抜きで", "抜きに", "抜いた"]],  # ～を抜いて／～抜いた
    "01M1H04XDKF8V9Z3F9V11SGSJX": [["というか"]],  # ～というか～というか
    # ── N4 ──
    "01G000000000000000M0000000": [["それは"]],  # それは～
    "01G000000000000000S0000000": [["のです", "んです", "のだ", "んだ", "の。", "のか", "の？", "のね", "のよ"]],  # ～の explanatory
    "01G000000000000000SR000000": [["とき", "時"]],  # ～とき＋助詞
    # ── N5: compound AND-groups ──
    "01G00000000000000034000000": [["へ"], ["行く", "行き", "来る", "来ま", "帰る", "帰り"]],  # ～へ行く・来る・帰る
    "01G00000000000000038000000": [["どこへも"], ["ない", "ません"]],  # どこへも～ない
    "01G00000000000000044000000": [["をします", "をしました", "をする", "をした"]],  # ～をします
    "01G0000000000000007W000000": [["ですか"]],  # ～ですか
    "01G0000000000000009G000000": [["へ"], ["に行く", "に行き", "に行っ"]],  # ～へ～に行く
    "01G000000000000000EG000000": [["でも"]],  # ～でも
    "01G000000000000000FM000000": [["てくれ"]],  # ～てくれます (все формы: てくれます/ました/る)
    "01G000000000000000GG000000": [["ても"]],  # ～ても
    "01G00000000000000064000000": [["があります", "がいます"]],  # ～が、～ existence
}

# Deliberately left undetectable (single frequent kana / no unique marker):
#   N5: ～は～です, ～も, ～か、～か, ～の, お～, ～と～×2, ～ね, ～よ, ～を,
#       ～で, ～か, ～い＋名詞, ～から, ～の～, ～や～, ～が～, ご～, ～に,
#       ～に～を～, ～を～, ～は, ～で～がある, ～と, ～を＋移動動詞, ～は～が～
#   N4: は, も, ～か～, ～さ, ～に, ～く／～に＋動詞, ～連用形
#   N1: ～うと～うと, ～とも～とも, ～つ～つ, ～こそ～が, ～なり～なり,
#       ～も～なら、～も～だ


def main() -> int:
    data = json.loads(CORPUS.read_text(encoding="utf-8"))
    rules = data["grammar"]
    by_id = {r["rule_id"]: r for r in rules}

    missing = [rid for rid in KEYWORDS if rid not in by_id]
    if missing:
        print(f"ERROR: unknown rule_ids: {missing}")
        return 1

    applied = 0
    for rid, kw in KEYWORDS.items():
        rule = by_id[rid]
        if rule.get("keywords"):
            print(f"WARN: {rid} already has keywords, skipping")
            continue
        rule["keywords"] = kw
        applied += 1

    CORPUS.write_text(
        json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    (STAGING / "keywords_map.json").write_text(
        json.dumps(KEYWORDS, ensure_ascii=False, indent=1), encoding="utf-8"
    )

    dead = [r for r in rules if not r.get("keywords") and not r.get("format_map")]
    print(f"applied keywords to {applied} rules")
    print(f"undetectable remaining: {len(dead)} (by-design, see docstring)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
