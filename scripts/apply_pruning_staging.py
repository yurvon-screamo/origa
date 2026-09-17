#!/usr/bin/env python3
"""Apply the owner-approved pruning + merge to a STAGING copy of the corpus.

The production corpus is NOT touched: user FSRS cards reference rule_ids,
and the Rust-side migration mechanics come later ("раст в конце"). This
script works on a staging copy that becomes the generation base, and emits
the migration map the Rust stage will consume:

    staging/grammar_v2.pruned.json  — corpus after deletions + merges
    staging/migration_map.json      — {deleted rule_id: successor rule_id | null}
                                        (null = no successor, card is dropped)

Deletions (owner-approved 2026-09-17):
  - 11 anchor-free pseudo/phrase/overview points (B + 辞書形/条件形)
  - 3 form categories whose detector actions re-anchor to concrete
    patterns (可能形→（ら）れる-potential, 受身形→（ら）れる-passive,
    使役形→（さ）せる-family)
  - 7 lexicon points (D: demonstratives, なかなか, きっと・たぶん…)
  - 意向形 KEPT (owner: "терять правила нельзя" on the volitional nuance)

Merges (5 twin pairs): the surviving rule keeps the lower level (met
earlier in study); the twin's keywords/format_map/related_patterns move
over when missing there; both rule_ids enter the migration map.

Usage:
    python scripts/apply_pruning_staging.py \
        [--corpus cdn/grammar/grammar_v2.json] \
        [--staging /tmp/opencode/staging]
"""

import argparse
import json
import sys
from pathlib import Path

# rule_ids resolved from patterns at runtime by this script (see PATTERNS).
DELETE_PATTERNS = {
    # B: textbook phrases / overview
    "そうです", "そうですか", "そうですね", "～は場所です", "今～時～分です",
    "～で行く", "何をしますか", "～語で何ですか",
    # B wave 2: phrase/topic lessons surfaced during the detection audit
    "ありました", "何か心配なこと", "こ・そ", "～時間・約束・用事",
    "数量の表現", "数量＋名詞", "数量＋で", "数量＋は", "数量＋も",
    "過去・現在＋名詞・な形容詞", "過去・現在＋い形容詞",
    # C: form categories with re-anchored detection
    "可能形", "受身形", "使役形", "辞書形", "条件形",
    # C wave 2: 「の文／使い方」 variants and honorific overview topics
    "可能形の文", "使役形の文", "命令形の使い方",
    "敬語", "敬語の種類", "敬語と文体", "敬語の敬語の一致",
    # D: lexicon (owner: dictionary translation suffices)
    "これ・それ・あれ", "この～・その～・あの～", "ここ・そこ・あそこ",
    "どこか・何か", "なかなか", "きっと・たぶん・もしかしたら",
}
# ～で N4 "four functions overview": identified by sd marker, not pattern
# (the N5 ～で stays).

MERGE_PAIRS = [
    # (survivor_pattern_hint, twin_pattern_hint) — same normalized pattern
    "～（さ）せられる",
    "～た～",
    "～される",
    "～たところで",
    "～たら、～た",
]


def core(p: str) -> str:
    import re

    return re.sub(r"[～〜\s]", "", p)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, default=Path("cdn/grammar/grammar_v2.json"))
    parser.add_argument("--staging", type=Path, default=Path("/tmp/opencode/staging"))
    args = parser.parse_args()

    data = json.loads(args.corpus.read_text(encoding="utf-8"))
    rules = data["grammar"]

    by_core: dict[str, list[int]] = {}
    by_exact: dict[str, list[int]] = {}
    for i, r in enumerate(rules):
        pattern = r["content"]["English"].get("pattern", "")
        by_core.setdefault(core(pattern), []).append(i)
        by_exact.setdefault(pattern, []).append(i)

    # ── deletions ──
    delete_ids: set[str] = set()
    for i, r in enumerate(rules):
        pattern = r["content"]["English"].get("pattern", "")
        sd = r["content"]["English"].get("short_description", "")
        if pattern in DELETE_PATTERNS:
            delete_ids.add(r["rule_id"])
        elif pattern == "～で" and "overview" in sd.lower():
            delete_ids.add(r["rule_id"])

    # ── merges ──
    migration: dict[str, str | None] = {rid: None for rid in delete_ids}
    merged_away: set[str] = set()
    for hint in MERGE_PAIRS:
        # EXACT pattern match: core() collapses ～た (past tense) and ～た～
        # (attributive) onto the same core, but they are different rules.
        group = by_exact.get(hint, [])
        if len(group) != 2:
            print(
                f"WARN: merge group {hint!r} has {len(group)} members, expected 2",
                file=sys.stderr,
            )
            continue
        a, b = rules[group[0]], rules[group[1]]
        # survivor: lower level (met earlier), anchors preferred
        level_rank = {"N5": 0, "N4": 1, "N3": 2, "N2": 3, "N1": 4}

        def rank(r: dict) -> tuple:
            anchors = 1 if (r.get("keywords") or r.get("format_map")) else 0
            return (-anchors, level_rank.get(r["level"], 9))

        survivor, twin = (a, b) if rank(a) <= rank(b) else (b, a)
        # Move the twin's detector anchors and related patterns over when
        # the survivor lacks them.
        if twin.get("keywords") and not survivor.get("keywords"):
            survivor["keywords"] = twin["keywords"]
        if twin.get("format_map") and not survivor.get("format_map"):
            survivor["format_map"] = twin["format_map"]
        for rel in twin.get("related_patterns_ref", []) or []:
            pass  # refs are by rule_id inside content; handled below
        for loc in ("English", "Russian"):
            twin_rel = twin["content"][loc].get("related_patterns", [])
            sur_rel = survivor["content"][loc].get("related_patterns", [])
            if isinstance(twin_rel, list) and isinstance(sur_rel, list):
                seen = {x.get("rule_id") for x in sur_rel if isinstance(x, dict)}
                survivor["content"][loc]["related_patterns"] = sur_rel + [
                    x for x in twin_rel if isinstance(x, dict) and x.get("rule_id") not in seen
                ]
        migration[twin["rule_id"]] = survivor["rule_id"]
        merged_away.add(twin["rule_id"])

    delete_all = delete_ids | merged_away
    pruned = [r for r in rules if r["rule_id"] not in delete_all]
    print(
        f"deleted: {len(delete_ids)} rules | merged away: {len(merged_away)} | "
        f"kept: {len(pruned)} (was {len(rules)})"
    )
    with_successor = sum(1 for v in migration.values() if v)
    print(f"migration map: {len(migration)} entries ({with_successor} with successor)")

    pruned_data = {"schema": data.get("schema"), "grammar": pruned}
    out_corpus = args.staging / "grammar_v2.pruned.json"
    out_map = args.staging / "migration_map.json"
    out_corpus.write_text(
        json.dumps(pruned_data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    out_map.write_text(
        json.dumps(migration, ensure_ascii=False, indent=1), encoding="utf-8"
    )
    print(f"staged: {out_corpus}\n        {out_map}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
