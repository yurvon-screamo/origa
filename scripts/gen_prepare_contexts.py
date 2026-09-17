#!/usr/bin/env python3
"""S0: prepare generation contexts for every staging rule (local, free).

Steps:
 1. Rename the five term-titles into living constructions (owner request:
    a learner may know the rule but not the kana term — 意向形 etc). Only
    title/pattern change; rule_id, keywords, format_map stay untouched.
 2. For every rule, build staging/contexts/<rule_id>.json:
    rule meta, detection keywords, condensed reference facts (J-nihongo
    usage categories + formation + points, Hanabira explanation), and the
    current content as a baseline.

Condensing matters: a full J-nihongo page is ~10KB; the prompt needs the
semantic skeleton (usage names/descriptions + 2 examples each), not every
example — the model must write its OWN examples anyway.
"""

import json
import re
import sys
from pathlib import Path

STAGING = Path("/tmp/opencode/staging")
JNIHONGO = Path("/tmp/opencode/jnihongo")
HANABIRA = Path("/tmp/opencode/hanabira")

TERM_RENAMES = {
    # old pattern → (EN title, RU title, new pattern)
    "意向形": ("～（よ）う（volitional form）", "～（よ）う（волюнтив）", "～（よ）う"),
    "命令形": ("～ろ／～え（imperative）", "～ろ／～え（императив）", "～ろ／～え"),
    "尊敬語": ("お～になる／～れる・られる", "お～になる／～れる・られる", "お～になる／～れる・られる"),
    "謙譲語": ("お～する／～いたす", "お～する／～いたす", "お～する／～いたす"),
    "丁寧語": ("～ます／～です", "～ます／～です", "～ます／～です"),
}

# patterns after renaming (for the idempotent repair branch)
RENAMED_PATTERNS = ("～（よ）う", "～ろ／～え", "お～になる／～れる・られる",
                    "お～する／～いたす", "～ます／～です")

LOCALES = ("English", "Russian", "Korean", "Vietnamese")


def condense_jnihongo(fact: dict, max_examples_per_usage: int = 2) -> dict:
    usages = [
        {
            "name": u["name"],
            "desc_ja": u["desc_ja"],
            "examples": u["examples"][:max_examples_per_usage],
        }
        for u in fact.get("usages", [])
    ]
    return {
        "title": fact["title"],
        "summary_ja": fact.get("summary_ja", ""),
        "summary_en": fact.get("summary_en", ""),
        "usages": usages,
        "formation": fact.get("formation", []),
        "points": fact.get("points", []),
        "similar": [
            {"title": s["title"], "level": s["level"]} for s in fact.get("similar", [])[:4]
        ],
    }


def condense_hanabira(point: dict) -> dict:
    return {
        "title": point.get("title", ""),
        "short_explanation": point.get("short_explanation", ""),
        "formation": point.get("formation", ""),
        "examples": [
            {"jp": e.get("jp", ""), "en": e.get("en", "")}
            for e in point.get("examples", [])[:4]
        ],
    }


def main() -> int:
    corpus = json.loads((STAGING / "grammar_v2.pruned.json").read_text(encoding="utf-8"))
    refs = json.loads((STAGING / "ref_mapping_pruned.json").read_text(encoding="utf-8"))
    facts = json.loads((JNIHONGO / "facts.json").read_text(encoding="utf-8"))
    facts_by_slug = {f["slug"]: f for f in facts}

    han_points: list[dict] = []
    for path in sorted(HANABIRA.glob("grammar_N*.json")):
        for item in json.loads(path.read_text(encoding="utf-8")):
            han_points.append(item)

    by_id = {r["rule_id"]: r for r in corpus["grammar"]}
    refs_by_id = {x["rule_id"]: x for x in refs}

    # ── 1. term renames (idempotent: fresh pruned corpus OR already-renamed) ──
    renamed = []
    new_patterns = set()
    en_by_pattern = {new: en for en, _, new in TERM_RENAMES.values()} if TERM_RENAMES else {}
    ru_by_pattern = {new: ru for _, ru, new in TERM_RENAMES.values()} if TERM_RENAMES else {}
    for r in corpus["grammar"]:
        pat = r["content"]["English"].get("pattern")
        if pat in TERM_RENAMES:  # fresh corpus: old kana term present
            en_title, ru_title, new_pattern = TERM_RENAMES[pat]
            r["content"]["English"].update(title=en_title, pattern=new_pattern)
            r["content"]["Russian"].update(title=ru_title, pattern=new_pattern)
            renamed.append((r["rule_id"], pat, new_pattern))
            new_patterns.add(new_pattern)
        elif pat in ru_by_pattern:  # already renamed on a previous run
            r["content"]["English"]["title"] = en_by_pattern[pat]
            r["content"]["Russian"]["title"] = ru_by_pattern[pat]
    # uniqueness of new patterns
    all_patterns = [
        r["content"]["English"].get("pattern") for r in corpus["grammar"]
    ]
    for np_ in new_patterns:
        if all_patterns.count(np_) > 1:
            print(f"ERROR: renamed pattern {np_!r} collides", file=sys.stderr)
            return 1

    (STAGING / "grammar_v2.pruned.json").write_text(
        json.dumps(corpus, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"renamed {len(renamed)} term rules: {[x[1] + '→' + x[2] for x in renamed]}")

    # ── 2. contexts ──
    out_dir = STAGING / "contexts"
    out_dir.mkdir(exist_ok=True)
    n_refs = n_han = n_kw = 0
    for r in corpus["grammar"]:
        rid = r["rule_id"]
        x = refs_by_id.get(rid, {"refs": []})

        jnihongo_refs = []
        for ref in x["refs"]:
            if ref["source"] != "jnihongo":
                continue
            slug = ref["file"].split("/")[-1].removesuffix(".html")
            fact = facts_by_slug.get(slug)
            if fact:
                jnihongo_refs.append({"match": ref["title"], "score": ref["score"],
                                      **condense_jnihongo(fact)})
            if len(jnihongo_refs) >= 2:
                break

        han_ref = None
        best = 0.0
        for ref in x["refs"]:
            if ref["source"] != "hanabira":
                continue
            core = re.sub(r"\([^)]*\)", "", ref["title"]).strip()
            cands = [p for p in han_points if p.get("title", "") == ref["title"]] or \
                    [p for p in han_points if core and core in p.get("title", "")]
            if cands and ref["score"] > best:
                han_ref = condense_hanabira(cands[0])
                best = ref["score"]

        keywords = r.get("keywords") or []
        context = {
            "rule": {
                "rule_id": rid,
                "level": r["level"],
                "title": r["content"]["English"]["title"],
                "pattern": r["content"]["English"].get("pattern"),
            },
            "detection": {
                "keywords": keywords,
                "format_map_detected": bool(r.get("format_map")),
                "note": "every JP example MUST contain at least one keyword "
                        "from EVERY group (AND across groups)" if keywords else
                        ("form-detection via morphology; write natural examples "
                         "of the pattern" if r.get("format_map") else
                         "no detector for this rule; stay strictly on-pattern"),
            },
            "reference": {
                "jnihongo": jnihongo_refs or None,
                "hanabira": han_ref,
            },
            "current": {
                loc: {
                    "short_description": r["content"][loc]["short_description"],
                    "explanation": r["content"][loc]["explanation"][:1200],
                }
                for loc in ("English", "Russian")
            },
        }
        (out_dir / f"{rid}.json").write_text(
            json.dumps(context, ensure_ascii=False, indent=1), encoding="utf-8"
        )
        n_refs += bool(jnihongo_refs)
        n_han += bool(han_ref)
        n_kw += bool(keywords)

    print(f"contexts: {len(corpus['grammar'])} -> {out_dir}")
    print(f"  with jnihongo facts: {n_refs} | with hanabira: {n_han} | with keywords: {n_kw}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
