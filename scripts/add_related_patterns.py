"""Add related_patterns to key grammar rule pairs (C5 fix).

Inserts a `related_patterns` field into both English and Russian content blocks
for a curated set of semantically linked rule pairs. Uses raw-text insertion
to preserve the original file formatting.

Each entry: (relative_path, english_text, russian_text).

Usage:
    python scripts/add_related_patterns.py
    python scripts/add_related_patterns.py --root cdn/grammar/rules --dry-run
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# 6 pairs = 12 rules. Each links to its semantic counterpart.
PAIRS: list[tuple[str, str, str]] = [
    # Pair 1: ことにする ↔ ことになる (active vs spontaneous decision)
    (
        "n3_lessons_01-04/rule_28_koto_ni_suru.json",
        "See also: ～ことになる — the decision-from-outside counterpart («it was decided that…»).",
        "См. также: ～ことになる — парный паттерн решения извне («решено, что…»).",
    ),
    (
        "n3_lessons_01-04/rule_30_koto_ni_naru.json",
        "See also: ～ことにする — the active counterpart («I decide to…»).",
        "См. также: ～ことにする — активный парный паттерн («я решаю…»).",
    ),
    # Pair 2: ざるを得ない ↔ ざる (derived stem)
    (
        "n2_lessons_17-20/rule_06_zaru_o_enai.json",
        "See also: ～ざる — the classical negative stem that ～ざるを得ない is built from.",
        "См. также: ～ざる — классическая отрицательная основа, на которой построено ～ざるを得ない.",
    ),
    (
        "n2_lessons_21-24/rule_33_zaru.json",
        "See also: ～ざるを得ない — the fixed expression «cannot help but…» built from this stem.",
        "См. также: ～ざるを得ない — устойчивое выражение «не могу не…», образованное от этой основы.",
    ),
    # Pair 3: おかげで ↔ せいで (positive vs negative cause)
    (
        "n3_lessons_09-12/rule_38_okage_de.json",
        "See also: ～せいで — the negative-cause counterpart («because of…» with a bad result).",
        "См. также: ～せいで — парный паттерн с негативной причиной («из-за…»).",
    ),
    (
        "n3_lessons_09-12/rule_40_sei_de.json",
        "See also: ～おかげで — the positive-cause counterpart («thanks to…» with a good result).",
        "См. также: ～おかげで — парный паттерн с позитивной причиной («благодаря…»).",
    ),
    # Pair 4: わけだ ↔ わけではない (affirmative vs negative)
    (
        "n2_lessons_13-16/rule_18_wake_da.json",
        "See also: ～わけではない — the negative counterpart («that's not to say…»).",
        "См. также: ～わけではない — отрицательный парный паттерн («это не значит, что…»).",
    ),
    (
        "n2_lessons_13-16/rule_25_wake_dewa_nai.json",
        "See also: ～わけだ — the affirmative counterpart («that means… / naturally»).",
        "См. также: ～わけだ — утвердительный парный паттерн («значит… / естественно»).",
    ),
    # Pair 5: てくる ↔ ていく in n3_lessons_05-08 (変化・プロセス)
    (
        "n3_lessons_05-08/rule_27_te_kuru.json",
        "See also: ～ていく (変化・プロセス) — the forward/future mirror of this pattern.",
        "См. также: ～ていく (変化・プロセス) — зеркальный парный паттерн (в будущее).",
    ),
    (
        "n3_lessons_05-08/rule_28_te_iku.json",
        "See also: ～てくる (変化・プロセス) — the backward/present mirror of this pattern.",
        "См. также: ～てくる (変化・プロセス) — зеркальный парный паттерн (к настоящему).",
    ),
    # Pair 6: てくる ↔ ていく in n3_lessons_09-12 (継続・方向性)
    (
        "n3_lessons_09-12/rule_23_te_kuru.json",
        "See also: ～ていく (継続・方向性) — the away/future counterpart of this pattern.",
        "См. также: ～ていく (継続・方向性) — парный паттерн (вдаль / в будущее).",
    ),
    (
        "n3_lessons_09-12/rule_24_te_iku.json",
        "See also: ～てくる (継続・方向性) — the toward/present counterpart of this pattern.",
        "См. также: ～てくる (継続・方向性) — парный паттерн (к настоящему / к говорящему).",
    ),
]


def insert_related_patterns(
    path: Path, en_text: str, ru_text: str, dry_run: bool = False
) -> tuple[bool, bool]:
    """Insert related_patterns into both language blocks via raw-text edit.

    Finds the pro_tip value in the raw text, then inserts a new
    related_patterns key right before the closing brace of each language block.
    Returns (was_modified, had_warning).
    """
    raw = path.read_text(encoding="utf-8")
    original_raw = raw
    data = json.loads(raw)
    content = data["content"]
    had_warning = False

    for lang, text in (("Russian", ru_text), ("English", en_text)):
        block = content[lang]
        if "related_patterns" in block:
            print(f"  SKIP {lang} — related_patterns already present", file=sys.stderr)
            continue

        pro_tip = block["pro_tip"]
        pro_tip_json = json.dumps(pro_tip, ensure_ascii=False)
        related_json = json.dumps(text, ensure_ascii=False)

        if pro_tip_json not in raw:
            print(f"  WARN: could not locate {lang} pro_tip in raw text", file=sys.stderr)
            had_warning = True
            continue

        # pro_tip is the last field; insert related_patterns after it.
        old = pro_tip_json + "\n    }"
        new = pro_tip_json + ",\n      \"related_patterns\": " + related_json + "\n    }"
        if old not in raw:
            print(f"  WARN: could not locate closing brace after {lang} pro_tip", file=sys.stderr)
            had_warning = True
            continue
        raw = raw.replace(old, new, 1)

    modified = raw != original_raw

    if modified and not dry_run:
        path.write_text(raw, encoding="utf-8")
    return (modified, had_warning)


def main() -> int:
    parser = argparse.ArgumentParser(description="Add related_patterns to key grammar rule pairs")
    parser.add_argument("--root", type=Path, default=Path("cdn/grammar/rules"), help="Root directory")
    parser.add_argument("--dry-run", action="store_true", help="Only report, do not write")
    args = parser.parse_args()

    modified = 0
    had_any_warning = False
    for rel_path, en_text, ru_text in PAIRS:
        path = args.root / rel_path
        if not path.exists():
            print(f"ERROR: {path} not found", file=sys.stderr)
            return 1
        print(f"Processing {rel_path}...")
        was_modified, warned = insert_related_patterns(path, en_text, ru_text, args.dry_run)
        if warned:
            had_any_warning = True
        if was_modified:
            modified += 1

    mode = "DRY RUN" if args.dry_run else "APPLIED"
    print(f"\n[{mode}] {modified} files modified (6 pairs, 12 rules).")
    return 1 if had_any_warning else 0


if __name__ == "__main__":
    sys.exit(main())
