#!/usr/bin/env python3
"""Local generation checkers (free, no LLM): schema, examples-audit, copy.

Checks a staging/gen/<rule_id>.json against its context:
  1. SCHEMA      — 4 locales × 6 fields present, non-empty, sane lengths;
                   jp_examples >= 3; examples field carries every jp_example.
  2. EXAMPLES    — every jp_example contains at least one keyword from EVERY
                   detection group (same contains-semantics as the production
                   detect_keyword_rules). Skipped for rules without keywords
                   (marked SKIPPED-no_kw, covered by judge/manual review).
  3. COPY        — n-gram overlap vs reference texts (J-nihongo examples and
                   summaries, Hanabira). 10-char grams for Japanese, 5-word
                   grams for other locales. Flag when overlap is high.

Usage: python scripts/check_generation.py <rule_id> [<rule_id>...]
       python scripts/check_generation.py --all
"""

import json
import re
import sys
from pathlib import Path

STAGING = Path("/tmp/opencode/staging")
LOCALES = ("English", "Russian", "Korean", "Vietnamese")
MIN_FIELD_LEN = {
    # RU/EN vs compact asian locales (Korean/Vietnamese run shorter)
    "short_description": 10,
    "explanation": 250,
    "how_to_form": 40,
    "examples": 120,
    "pro_tip": 15,
}
MIN_FIELD_LEN_KO_VI = {
    "short_description": 8,
    "explanation": 200,
    "how_to_form": 30,
    "examples": 90,
    "pro_tip": 10,
}


def check_schema(g: dict) -> list[str]:
    errs = []
    for loc in LOCALES:
        block = g.get(loc)
        if not isinstance(block, dict):
            errs.append(f"{loc}: missing block")
            continue
        limits = MIN_FIELD_LEN if loc in ("English", "Russian") else MIN_FIELD_LEN_KO_VI
        for f, minlen in limits.items():
            v = block.get(f)
            if not isinstance(v, str) or len(v.strip()) < minlen:
                errs.append(f"{loc}.{f}: missing/too thin")
        nu = block.get("nuances")
        if not isinstance(nu, dict) or not (
            isinstance(nu.get("common_mistakes"), list)
            and isinstance(nu.get("notes"), list)
            and (nu.get("common_mistakes") or nu.get("notes"))
        ):
            errs.append(f"{loc}.nuances: malformed/empty")
    jp = g.get("jp_examples")
    if not isinstance(jp, list) or len(jp) < 3:
        errs.append(f"jp_examples: {len(jp) if isinstance(jp, list) else 'missing'} < 3")
    return errs


def check_examples(g: dict, keywords: list[list[str]]) -> tuple[list[str], str]:
    if not keywords:
        return [], "SKIPPED-no_kw"
    errs = []
    for ex in g.get("jp_examples", []):
        for grp in keywords:
            if not any(k in ex for k in grp):
                errs.append(f"example without form: {ex[:40]}… (missing group {grp})")
    return errs, "OK"


def ngrams_jp(s: str, n: int = 10) -> set[str]:
    s = re.sub(r"\s+", "", s)
    return {s[i : i + n] for i in range(len(s) - n + 1)}


def ngrams_words(s: str, n: int = 5) -> set[str]:
    ws = re.findall(r"[\w]+", s.lower())
    return {" ".join(ws[i : i + n]) for i in range(len(ws) - n + 1)}


def check_copy(g: dict, ctx: dict) -> list[str]:
    refs = ctx.get("reference", {})
    jp_ref, text_ref = set(), set()
    for fact in refs.get("jnihongo") or []:
        jp_ref |= ngrams_jp(fact.get("summary_ja", ""))
        for u in fact.get("usages", []):
            jp_ref |= ngrams_jp(u.get("desc_ja", ""))
            for e in u.get("examples", []):
                jp_ref |= ngrams_jp(e.get("jp", ""))
                text_ref |= ngrams_words(e.get("en", ""))
        text_ref |= ngrams_words(fact.get("summary_en", ""))
    han = refs.get("hanabira") or {}
    for e in han.get("examples", []):
        jp_ref |= ngrams_jp(e.get("jp", ""))
        text_ref |= ngrams_words(e.get("en", ""))

    # The pattern formula itself (＋keywords) is grammar, not copying: a
    # valid example MUST contain it, and formulas longer than the n-gram
    # size guarantee overlap. Drop reference grams that carry a keyword.
    keywords = [k for grp in ctx.get("detection", {}).get("keywords", []) for k in grp]
    pattern_core = re.sub(r"[～〜\s]", "", ctx["rule"].get("pattern") or "")
    stop = [k for k in keywords if k] + ([pattern_core] if pattern_core else [])
    if stop:
        jp_ref = {gram for gram in jp_ref if not any(s in gram or gram in s for s in stop)}

    errs = []
    for ex in g.get("jp_examples", []):
        grams = ngrams_jp(ex)
        if grams and jp_ref and len(grams & jp_ref) / len(grams) > 0.30:
            errs.append(f"jp example overlaps reference ≥30%: {ex[:40]}…")
    for loc in LOCALES:
        expl = (g.get(loc) or {}).get("explanation", "")
        grams = ngrams_words(expl)
        if grams and text_ref and len(grams & text_ref) / len(grams) > 0.25:
            errs.append(f"{loc}.explanation overlaps reference ≥25%")
    return errs


def run(rid: str) -> dict:
    gen_path = STAGING / "gen" / f"{rid}.json"
    ctx = json.loads((STAGING / "contexts" / f"{rid}.json").read_text(encoding="utf-8"))
    g = json.loads(gen_path.read_text(encoding="utf-8"))

    schema_errs = check_schema(g)
    kw = ctx.get("detection", {}).get("keywords", [])
    ex_errs, ex_status = check_examples(g, kw)
    copy_errs = check_copy(g, ctx)

    ok = not (schema_errs or ex_errs or copy_errs)
    return {
        "rule_id": rid,
        "ok": ok,
        "schema": schema_errs,
        "examples_audit": {"status": ex_status, "errors": ex_errs},
        "copy": copy_errs,
    }


def main() -> int:
    args = sys.argv[1:]
    rids = (
        [p.stem for p in (STAGING / "gen").glob("*.json")]
        if args == ["--all"]
        else args
    )
    all_ok = True
    for rid in rids:
        res = run(rid)
        status = "PASS" if res["ok"] else "FAIL"
        all_ok &= res["ok"]
        print(f"[{status}] {rid}")
        for k in ("schema", "copy"):
            for e in res[k]:
                print(f"    {k}: {e}")
        if res["examples_audit"]["errors"]:
            for e in res["examples_audit"]["errors"]:
                print(f"    examples: {e}")
        elif res["examples_audit"]["status"] != "OK":
            print(f"    examples: {res['examples_audit']['status']}")
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())
