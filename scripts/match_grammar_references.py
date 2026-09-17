#!/usr/bin/env python3
"""Map every corpus grammar rule to ALL matching reference points.

References (fetched to /tmp/opencode, see session notes):
  - J-nihongo: 626 pages, catalog.json {file, slug, level, title}
  - Hanabira:  828 JSON points in grammar_N*.json

For each of our rules the matcher collects every reference point whose
Japanese core is similar enough (not just the single best), sorted by
score, so the generation pipeline can ground each rule in several
sources.

Matching model:
  - both sides are normalized to Japanese-only cores (romaji parens,
    gloss parens, waves, latin, digits stripped);
  - compound titles are split into alternatives on ・／/，、 and matched
    pairwise — 「きっと・たぶん・もしかしたら」 vs 「たぶん／ Perhaps」 hits;
  - score: exact=1.0, containment(>=4 chars)=0.9, kana-bigram Jaccard
    otherwise;
  - a reference enters the array when its best alternative score passes
    --threshold (default 0.45).

Usage:
    python scripts/match_grammar_references.py \
        [--corpus cdn/grammar/grammar_v2.json] \
        [--jnihongo /tmp/opencode/jnihongo/catalog.json] \
        [--hanabira /tmp/opencode/hanabira] \
        [--threshold 0.45] \
        [--out /tmp/opencode/ref_mapping.json]
"""

import argparse
import json
import re
import sys
from collections import Counter
from pathlib import Path

_ROMAJI_PAREN = re.compile(r"\([^)]*\)")
_ANY_PAREN = re.compile(r"[（(][^）)]*[）)]")
_SPLIT = re.compile(r"[・／/,、\s]+")
_NON_JP = re.compile(r"[^ぁ-ヿ㐀-䶿一-鿿]+")


def _core(title: str) -> str:
    s = _ROMAJI_PAREN.sub("", title)
    s = _ANY_PAREN.sub("", s)
    s = _NON_JP.sub("", s)
    return s


def alternatives(title: str) -> list[str]:
    """Alternative cores of a (possibly compound) title.

    The concatenated core is appended, so a compound reference
    (「～から、～まで」→ [から, まで, からまで]) can hit our fused pattern
    (「～から～まで」→ [からまで]).
    """
    s = _ROMAJI_PAREN.sub("", title)
    s = _ANY_PAREN.sub("", s)
    parts = [p for p in (_core(x) for x in _SPLIT.split(s)) if p]
    if not parts:
        return []
    uniq = list(dict.fromkeys(parts))
    if len(uniq) > 1:
        uniq.append("".join(uniq))
    return uniq


def _bigrams(s: str) -> set[str]:
    return {s[i : i + 2] for i in range(len(s) - 1)} if len(s) > 1 else {s}


def pair_score(a: str, b: str, mono_a: bool, mono_b: bool) -> float:
    if not a or not b:
        return 0.0
    # A side shorter than 3 kana (も, あれ) matches thousands of tails and
    # bigram-collides with unrelated points; short cores are only scorable
    # between two MONO titles.
    if min(len(a), len(b)) < 3 and not (mono_a and mono_b):
        return 0.0
    if a == b:
        return 1.0
    short, long_ = (a, b) if len(a) <= len(b) else (b, a)
    if len(short) >= 4 and short in long_:
        return 0.9
    ja, jb = _bigrams(a), _bigrams(b)
    return len(ja & jb) / len(ja | jb) if ja | jb else 0.0


def title_score(ours: str, ref_title: str) -> float:
    ours_alts, ref_alts = alternatives(ours), alternatives(ref_title)
    if not ours_alts or not ref_alts:
        return 0.0
    mono_ours, mono_ref = len(ours_alts) == 1, len(ref_alts) == 1
    return max(
        (pair_score(a, b, mono_ours, mono_ref) for a in ours_alts for b in ref_alts),
        default=0.0,
    )


def load_references(jnihongo: Path | None, hanabira: Path | None) -> list[dict]:
    refs = []
    if jnihongo and jnihongo.exists():
        for entry in json.loads(jnihongo.read_text(encoding="utf-8")):
            refs.append(
                {
                    "source": "jnihongo",
                    "title": entry["title"],
                    "level": entry["level"].upper(),
                    "file": entry["file"],
                }
            )
    if hanabira and hanabira.exists():
        for path in sorted(hanabira.glob("grammar_N*.json")):
            level = re.search(r"(N[1-5])", path.stem).group(1)
            for item in json.loads(path.read_text(encoding="utf-8")):
                refs.append(
                    {
                        "source": "hanabira",
                        "title": item.get("title", ""),
                        "level": level,
                        "file": path.name,
                    }
                )
    return refs


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, default=Path("cdn/grammar/grammar_v2.json"))
    parser.add_argument("--jnihongo", type=Path, default=Path("/tmp/opencode/jnihongo/catalog.json"))
    parser.add_argument("--hanabira", type=Path, default=Path("/tmp/opencode/hanabira"))
    parser.add_argument("--threshold", type=float, default=0.45)
    parser.add_argument("--out", type=Path, default=Path("/tmp/opencode/ref_mapping.json"))
    args = parser.parse_args()

    data = json.loads(args.corpus.read_text(encoding="utf-8"))
    refs = load_references(args.jnihongo, args.hanabira)
    print(f"references loaded: {len(refs)} (jnihongo+hanabira)")

    mapping = []
    for rule in data["grammar"]:
        pattern = rule["content"]["English"].get("pattern") or rule["content"]["English"]["title"]
        matches = []
        for ref in refs:
            score = title_score(pattern, ref["title"])
            if score >= args.threshold:
                matches.append({**ref, "score": round(score, 2)})
        matches.sort(key=lambda m: -m["score"])
        mapping.append(
            {
                "rule_id": rule["rule_id"],
                "level": rule["level"],
                "title": rule["content"]["English"]["title"],
                "pattern": pattern,
                "refs": matches,
            }
        )

    args.out.write_text(
        json.dumps(mapping, ensure_ascii=False, indent=1), encoding="utf-8"
    )

    # Coverage summary
    covered = [m for m in mapping if m["refs"]]
    both = [m for m in mapping if {r["source"] for r in m["refs"]} == {"jnihongo", "hanabira"}]
    dist = Counter(min(len(m["refs"]), 5) for m in mapping)
    by_level = {}
    for m in mapping:
        key = m["level"]
        by_level.setdefault(key, [0, 0])
        by_level[key][0 if m["refs"] else 1] += 1
    print(f"\nmapping: {len(mapping)} rules -> {args.out}")
    print(f"with >=1 ref: {len(covered)} ({100 * len(covered) // len(mapping)}%) | both sources: {len(both)} | no ref: {len(mapping) - len(covered)}")
    print("refs-per-rule distribution (5+ bucketed):", dict(sorted(dist.items())))
    print("coverage by level (covered / bare):")
    for lvl in sorted(by_level):
        print(f"  {lvl}: {by_level[lvl][0]} / {by_level[lvl][1]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
