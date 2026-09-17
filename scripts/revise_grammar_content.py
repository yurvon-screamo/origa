#!/usr/bin/env python3
"""Content revision worklist for the grammar corpus (#503 content pass).

The corpus content was LLM-generated without reference grounding, which
left hallucinations ("обман": examples that do not carry the taught form,
mistranslations) and level exhaustion (N1 explanation medians are half of
N2's). This tool drives a supervised revision against reference sources
(Hanabira CC-BY-SA datasets used for FACT-CHECKING ONLY — the license
strategy is "сверка, не копирование": no text is copied from them).

Modes:
    prepare  — match our N-level rules against reference points and emit a
               worklist JSON: per rule, our current RU/EN content plus the
               matched reference for side-by-side revision.
    apply    — validate and merge a revision batch (my hand-written
               content) back into the corpus.

Immutable per rule: rule_id, level, title, pattern, related_patterns,
keywords, format_map — the tokenizer/validators anchor on them. Only the
content fields (short_description, explanation, how_to_form, examples,
nuances, pro_tip) may change, in both locales or one.

Usage:
    python scripts/revise_grammar_content.py prepare --levels N1,N2 \
        --hanabira-dir /tmp/opencode/hanabira --out /tmp/opencode/worklist_n1n2.json
    python scripts/revise_grammar_content.py apply --batch /tmp/opencode/batch_001.json
"""

import argparse
import json
import re
import sys
from pathlib import Path

CORPUS = Path("cdn/grammar/grammar_v2.json")
CONTENT_FIELDS = (
    "short_description",
    "explanation",
    "how_to_form",
    "examples",
    "nuances",
    "pro_tip",
)
TEXT_FIELDS = tuple(f for f in CONTENT_FIELDS if f != "nuances")
LOCALES = ("English", "Russian")
MIN_EXAMPLES = 3


# ───────────────────────── matching ─────────────────────────

_ROMAJI_PAREN = re.compile(r"\([^)]*\)")
_ANY_PAREN = re.compile(r"[（(][^）)]*[）)]")
_NON_JP = re.compile(r"[^ぁ-ヿ㐀-䶿一-鿿]+")


def _jp_core(title: str) -> str:
    """Japanese-only core of a title: romaji parens, glosses, waves, spaces out."""
    s = _ROMAJI_PAREN.sub("", title)
    s = _ANY_PAREN.sub("", s)
    s = _NON_JP.sub("", s)
    return s


def _bigrams(s: str) -> set[str]:
    return {s[i : i + 2] for i in range(len(s) - 1)} if len(s) > 1 else {s}


def match_score(our_pattern: str, ref_title: str) -> float:
    """0..1 similarity between our pattern and a reference title."""
    a, b = _jp_core(our_pattern), _jp_core(ref_title)
    if not a or not b:
        return 0.0
    if a == b:
        return 1.0
    short, long_ = (a, b) if len(a) <= len(b) else (b, a)
    # Substring containment only counts for non-trivial cores: a single
    # kana (と, に) nests into thousands of unrelated titles.
    if len(short) >= 4 and short in long_:
        return 0.9
    ja, jb = _bigrams(a), _bigrams(b)
    return len(ja & jb) / len(ja | jb) if ja | jb else 0.0


def best_reference(pattern: str, refs: list[dict]) -> tuple[dict | None, float]:
    best, best_score = None, 0.0
    for ref in refs:
        score = match_score(pattern, ref["title"])
        if score > best_score:
            best, best_score = ref, score
    return best, best_score


# ───────────────────────── prepare ─────────────────────────


def load_references(hanabira_dir: Path | None) -> list[dict]:
    refs: list[dict] = []
    if not hanabira_dir:
        return refs
    for path in sorted(hanabira_dir.glob("grammar_N*.json")):
        level = re.search(r"(N[1-5])", path.stem).group(1)
        for item in json.loads(path.read_text(encoding="utf-8")):
            refs.append(
                {
                    "title": item.get("title", ""),
                    "level": level,
                    "short_explanation": item.get("short_explanation", ""),
                    "long_explanation": item.get("long_explanation", ""),
                    "formation": item.get("formation", ""),
                    "examples": item.get("examples", []),
                }
            )
    return refs


def _example_jp_count(examples: str) -> int:
    return len([l for l in examples.split("\n") if re.search(r"[ぁ-ヿ]", l)])


def _quality_flag(our: dict) -> str:
    """Cheap priority heuristics: which rules need attention first."""
    flags = []
    for loc in LOCALES:
        c = our[loc]
        if len(c["explanation"].strip()) < 150:
            flags.append(f"thin-explanation/{loc[:2].lower()}")
        if _example_jp_count(c["examples"]) < MIN_EXAMPLES:
            flags.append(f"few-examples/{loc[:2].lower()}")
        jp = [l for l in c["examples"].split("\n") if re.search(r"[ぁ-ヿ]", l)]
        tr = [l for l in c["examples"].split("\n") if re.search(r"[а-яА-Яa-z]", l)]
        if jp and len(jp) > len(tr):
            flags.append(f"untranslated-examples/{loc[:2].lower()}")
    return ",".join(sorted(set(flags)))


def cmd_prepare(args: argparse.Namespace) -> int:
    data = json.loads(CORPUS.read_text(encoding="utf-8"))
    refs = load_references(Path(args.hanabira_dir) if args.hanabira_dir else None)
    levels = {x.strip() for x in args.levels.split(",")}

    worklist, matched = [], 0
    for rule in data["grammar"]:
        if rule["level"] not in levels:
            continue
        pattern = rule["content"]["English"].get("pattern") or rule["content"]["English"]["title"]
        ref, score = (None, 0.0)
        if refs:
            ref, score = best_reference(pattern, refs)
            if score < 0.5:
                ref = None
        if ref:
            matched += 1
        worklist.append(
            {
                "rule_id": rule["rule_id"],
                "level": rule["level"],
                "title": rule["content"]["English"]["title"],
                "pattern": pattern,
                "done": False,
                "priority_flags": _quality_flag(rule["content"]),
                "our": {
                    loc: {f: rule["content"][loc][f] for f in CONTENT_FIELDS}
                    for loc in LOCALES
                },
                "reference": {
                    "title": ref["title"] if ref else None,
                    "level": ref["level"] if ref else None,
                    "match_score": round(score, 2),
                    "short_explanation": ref["short_explanation"] if ref else None,
                    "long_explanation": ref["long_explanation"] if ref else None,
                    "formation": ref["formation"] if ref else None,
                    "examples": ref["examples"] if ref else None,
                },
                "revision": None,
            }
        )

    out = Path(args.out)
    out.write_text(
        json.dumps(worklist, ensure_ascii=False, indent=1), encoding="utf-8"
    )
    by_level = {}
    for w in worklist:
        key = w["level"] + ("/ref" if w["reference"]["title"] else "/no-ref")
        by_level[key] = by_level.get(key, 0) + 1
    print(f"worklist: {len(worklist)} rules -> {out}")
    for key in sorted(by_level):
        print(f"  {key}: {by_level[key]}")
    return 0


# ───────────────────────── apply ─────────────────────────


def validate_revision(rev: dict) -> list[str]:
    errors = []
    for loc in rev:
        if loc not in LOCALES:
            errors.append(f"unknown locale {loc!r}")
            continue
        for field, value in rev[loc].items():
            if field not in CONTENT_FIELDS:
                errors.append(f"{loc}.{field}: immutable/unknown field")
                continue
            if field == "nuances":
                if not isinstance(value, dict):
                    errors.append(f"{loc}.nuances: must be an object")
                else:
                    for key in ("common_mistakes", "notes"):
                        seq = value.get(key, [])
                        if not isinstance(seq, list) or any(
                            not isinstance(x, str) or not x.strip() for x in seq
                        ):
                            errors.append(f"{loc}.nuances.{key}: must be string list")
                continue
            if not isinstance(value, str) or not value.strip():
                errors.append(f"{loc}.{field}: empty")
            elif field in ("short_description", "explanation", "how_to_form", "examples"):
                if len(value.strip()) < 60:
                    errors.append(f"{loc}.{field}: too thin ({len(value.strip())} chars)")
        if "English" in rev and "examples" in rev.get("English", {}):
            jp_lines = [
                l
                for l in rev["English"]["examples"].split("\n")
                if re.search(r"[ぁ-ヿ]", l)
            ]
            if len(jp_lines) < MIN_EXAMPLES:
                errors.append(
                    f"English.examples: {len(jp_lines)} JP lines, need >= {MIN_EXAMPLES}"
                )
    return errors


def cmd_apply(args: argparse.Namespace) -> int:
    batch = json.loads(Path(args.batch).read_text(encoding="utf-8"))
    data = json.loads(CORPUS.read_text(encoding="utf-8"))
    by_id = {r["rule_id"]: r for r in data["grammar"]}

    errors = []
    applied = 0
    for entry in batch:
        rid = entry.get("rule_id")
        rev = entry.get("revision")
        if rid not in by_id:
            errors.append(f"{rid}: unknown rule_id")
            continue
        if not isinstance(rev, dict) or not rev:
            errors.append(f"{rid}: empty revision")
            continue
        errors += [f"{rid}: {e}" for e in validate_revision(rev)]

    if errors:
        for e in errors:
            print(f"ERROR {e}", file=sys.stderr)
        print(f"REFUSING: {len(errors)} errors, corpus untouched", file=sys.stderr)
        return 1

    for entry in batch:
        rule = by_id[entry["rule_id"]]
        for loc, fields in entry["revision"].items():
            rule["content"][loc].update(fields)
        applied += 1

    CORPUS.write_text(
        json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"applied {applied} rule revisions -> {CORPUS}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="mode", required=True)

    p_prepare = sub.add_parser("prepare", help="build a revision worklist")
    p_prepare.add_argument("--levels", required=True, help="comma-separated, e.g. N1,N2")
    p_prepare.add_argument("--hanabira-dir", default=None, help="dir with grammar_N*.json refs")
    p_prepare.add_argument("--out", required=True)

    p_apply = sub.add_parser("apply", help="merge a revision batch into the corpus")
    p_apply.add_argument("--batch", required=True)

    args = parser.parse_args()
    return cmd_prepare(args) if args.mode == "prepare" else cmd_apply(args)


if __name__ == "__main__":
    sys.exit(main())
