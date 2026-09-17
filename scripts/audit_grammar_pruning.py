#!/usr/bin/env python3
"""Pruning audit for the grammar corpus (DATA stage, no code changes).

Produces a staging file with every removal/merge candidate grouped by the
owner-approved categories, with per-rule context (pattern, sd, ref status,
detector anchors) so the list can be reviewed as data before anything is
applied to the corpus.

Categories (owner decisions, 2026-09-17):
  B — pseudo-points: textbook phrase-patterns that are not grammar points
      (～は場所です, 今～時～分です), plus "overview" points duplicating the
      concrete ones (～で four functions overview).
  C — form-category points: morphological umbrella terms as standalone
      rules (可能形, 意向形, 尊敬語...) — concrete usages already live in
      other rules.
  D — pure lexicon masquerading as grammar (なかなか, これ・それ・あれ).
  M — merge twins: same usage duplicated as two rules (owner-approved
      five pairs; kept here for the migration map).

SAFETY: a rule carrying tokenizer anchors (keywords/format_map that the
detector relies on) is flagged `anchor_locked` and must NOT be pruned
without re-anchoring the detector (～そうだ combined carries VerbToSou).

Usage:
    python scripts/audit_grammar_pruning.py \
        [--corpus cdn/grammar/grammar_v2.json] \
        [--refs /tmp/opencode/ref_mapping.json] \
        [--out /tmp/opencode/staging/pruning.json]
"""

import argparse
import json
import re
import sys
from pathlib import Path

# ── category seeds (patterns are JP cores; reviewed manually per hit) ──

B_PHRASE_PATTERNS = [
    r"^～?今～時～分です$",
    r"^～は場所です$",
    r"^～語で何ですか$",
    r"^～で行く$",
    r"^そう(です|ですね|ですか)$",
    r"^～は何ですか$",
    r"^何をしますか$",
]
# "combined" is NOT a marker: sd like «Combined with A — amplified effect»
# (～と相まって) legitimately describes the pattern's meaning.
B_SD_MARKERS = re.compile(r"overview|総覧|まとめ", re.I)

C_TERMS = [
    "可能形", "意向形", "命令形", "条件形", "受身形", "使役形",
    "尊敬語", "謙譲語", "辞書形", "ます形", "て形", "ない形",
    "た形", "普通形", "可能動詞", "丁寧語", "美化語",
]

D_LEXICON = [
    "なかなか", "これ・それ・あれ", "この～・その～・あの～",
    "ここ・そこ・あそこ", "どこか・何か", "きっと・たぶん・もしかしたら",
]

MERGE_TWINS = [
    # (keep_hint_regex, drop is the other same-pattern rule; manual pick)
    ("～（さ）せられる",),
    ("～た～",),
    ("～される",),
    ("～たところで",),
    ("～たら、～た",),
]


def core(p: str) -> str:
    return re.sub(r"[～〜\s]", "", p)


def categorize(rule: dict) -> tuple[str, str] | None:
    """Return (category, reason) or None for keep-rules."""
    en = rule["content"]["English"]
    pattern = en.get("pattern", "")
    sd = en.get("short_description", "")
    c = core(pattern)

    for term in D_LEXICON:
        if term in pattern:
            return "D", f"lexicon point ({term})"
    if any(re.match(rx, pattern) for rx in B_PHRASE_PATTERNS):
        return "B", "textbook phrase, not a grammar pattern"
    if B_SD_MARKERS.search(sd):
        return "B", "overview/combined point (sd marker)"
    if pattern in C_TERMS or c in C_TERMS:
        return "C", "morphological form-category"
    for (tw,) in MERGE_TWINS:
        if core(tw) == c:
            return None  # handled by the merge stage, not pruning
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, default=Path("cdn/grammar/grammar_v2.json"))
    parser.add_argument("--refs", type=Path, default=Path("/tmp/opencode/ref_mapping.json"))
    parser.add_argument("--out", type=Path, default=Path("/tmp/opencode/staging/pruning.json"))
    args = parser.parse_args()

    data = json.loads(args.corpus.read_text(encoding="utf-8"))
    refs = {}
    if args.refs.exists():
        for x in json.loads(args.refs.read_text(encoding="utf-8")):
            refs[x["rule_id"]] = bool(x["refs"])

    out = {"B": [], "C": [], "D": [], "anchor_locked": []}
    for rule in data["grammar"]:
        cat = categorize(rule)
        if not cat:
            continue
        category, reason = cat
        has_anchors = bool(rule.get("keywords") or rule.get("format_map"))
        entry = {
            "rule_id": rule["rule_id"],
            "level": rule["level"],
            "pattern": rule["content"]["English"].get("pattern", ""),
            "title": rule["content"]["English"]["title"],
            "sd": rule["content"]["English"]["short_description"][:90],
            "has_ref": refs.get(rule["rule_id"], False),
            "detector_anchors": has_anchors,
            "reason": reason,
        }
        if has_anchors:
            out["anchor_locked"].append({**entry, "category": category})
        else:
            out[category].append(entry)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8")

    print(f"staged -> {args.out}")
    for cat in ("B", "C", "D"):
        print(f"  {cat}: {len(out[cat])}")
    print(f"  anchor_locked (detector depends on them): {len(out['anchor_locked'])}")
    for e in out["anchor_locked"]:
        print(f"    [{e['category']}] {e['pattern']!r} — {e['reason']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
