"""format_map ↔ pattern consistency check (schema v3 only).

Guards the quiz/tokenizer anchor against content drift: the kana a rule's
``format_map`` chain produces must be visible inside the rule's own
``pattern``. This is the tripwire for the v2→v3 migration bug class where
a rule's content was rewritten but the v2-era anchor survived — the
～ます／～です rule shipped with the honorific お～ください anchor
(VerbToOKudasai), so its quiz presented お開きください as the correct
"polite ます/です" form.

The check is heuristic (substring overlap, not conjugation-aware): it is
a deploy gate against wrong-action-family anchors, not a semantics
verifier. Known-good chains the heuristic cannot prove live in
``KNOWN_GOOD`` with one-line justifications.
"""

from __future__ import annotations

import re
from typing import TYPE_CHECKING, Final

if TYPE_CHECKING:
    # Runtime import would be circular (validate_grammar_v2 imports this
    # module); the Report type is annotation-only.
    from validate_grammar_v2 import Report

# Kana a FormatAction contributes to the produced form. Mirrors the Rust
# enum ``FormatAction`` (origa/src/dictionary/grammar.rs) — keep in sync:
# a variant added in Rust without an entry here emits a WARN on the first
# corpus rule that uses it. Stem operations (no fixed tail) and removals
# map to "" — a legal empty contribution, never a WARN.
FORMAT_ACTION_SIGNATURES: Final[dict[str, str]] = {
    # I-adjective / な-adjective
    "AdjectiveRemovePostfix": "",
    "AdjectiveToKunai": "くない",
    "AdjectiveToKatta": "かった",
    "AdjectiveToKunakatta": "くなかった",
    "AdjectiveToKute": "くて",
    "AdjectiveToKu": "く",
    "AdjectiveToKereba": "ければ",
    "AdjectiveToSou": "そう",
    "AdjectiveToSugiru": "すぎる",
    "AdjectiveToNa": "な",
    "AdjectiveToDe": "で",
    "AdjectiveToNara": "なら",
    "AdjectiveToSouNa": "そうな",
    "AdjectiveToNasasou": "なさそう",
    "AdjectiveToGaru": "がる",
    # Verb conjugations
    "VerbToTeForm": "て",
    "VerbToMainView": "",
    "VerbToMasu": "ます",
    "VerbToMasen": "ません",
    "VerbToMashita": "ました",
    "VerbToMasenDeshita": "ませんでした",
    "VerbToMashou": "ましょう",
    "VerbToStem": "",
    "VerbToMizenkei": "",
    "VerbToTa": "た",
    "VerbToNai": "ない",
    "VerbToTara": "たら",
    "VerbToBa": "ば",
    "VerbToPotential": "れる",
    "VerbToPassive": "れる",
    "VerbToCausative": "せる",
    "VerbToCausativePassive": "せられる",
    "VerbToImperative": "ろ",
    "VerbToVolitional": "う",
    "VerbToSou": "そう",
    "VerbToZu": "ず",
    "VerbToTai": "たい",
    "VerbToYasui": "やすい",
    "VerbToNikui": "にくい",
    "VerbToSugiru": "すぎる",
    "VerbToChau": "ちゃう",
    "VerbToToku": "とく",
    "VerbToTeru": "てる",
    "VerbToONinarimasu": "になります",
    "VerbToOKudasai": "ください",
    "VerbToOShimasu": "します",
    "VerbToNasai": "なさい",
    "VerbToKudasai": "ください",
    "VerbToIrasshai": "いらっしゃい",
    # Universal
    "RemovePostfix": "",
}

# Chains the substring heuristic cannot prove but that are verified
# correct. Keyed (rule_id, action_name, signature); drop an entry when the
# corpus stops using it. Extra entries are harmless (they simply never fire).
KNOWN_GOOD: Final[frozenset[tuple[str, str, str]]] = frozenset(
    {
        # ～ておく → とく is the contracted おく; shares single kana only.
        ("01G000000000000000KR000000", "VerbToToku", "とく"),
        # ～てみる → てみ + みます (polite ます of みる); shared み is 1 kana.
        ("01G000000000000000TM000000", "AddPostfix", "みます"),
        # ～て来る → 来ます (polite ます of 来る); shared 来 is 1 kanji.
        ("01G000000000000000WC000000", "AddPostfix", "来ます"),
        # お～する → します (polite ます of する); shared す is 1 kana.
        ("01G000000000000000ZR000000", "VerbToOShimasu", "します"),
        # Causative せる attaches as （さ）せて…; patterns show せて, not せる.
        ("01G000000000000000YW000000", "VerbToCausative", "せる"),
        ("01KV2BRAW30ESEMGXK3N2PTAEN", "VerbToCausative", "せる"),
        ("01KV2BRAW30ESEMGXK3N2PTAEK", "VerbToCausative", "せる"),
        ("01KV2BRAW30ESEMGXK3N2PTAEM", "VerbToCausative", "せる"),
        ("01KV2BRAW30ESEMGXK3N2PTAEJ", "VerbToCausative", "せる"),
    }
)

# Hiragana + katakana (incl. ー) + CJK ideographs: the runs that carry the
# pattern's visible kana. Katakana is future-proofing (0 occurrences in the
# current corpus; dropping it would only ever skip, never false-ERROR).
_RUN_RE = re.compile(r"[\u3041-\u309F\u30A0-\u30FF\u3400-\u4DBF\u4E00-\u9FFF]+")
_OPTIONALITY_RE = re.compile(r"（[^）]*）|\[[^\]]*\]")


def pattern_substrings(pattern: str) -> set[str]:
    """All ≥2-char substrings of the pattern's kana/kanji runs.

    Optionality markers （…）/[…] and the ～/＿ placeholders are stripped
    first (～（よ）う legitimately yields no ≥2-char substrings — the
    chain-skip rule handles that).
    """
    text = _OPTIONALITY_RE.sub("", pattern).replace("～", "").replace("＿", "")
    out: set[str] = set()
    # The ／ and / splits are defensive (neither occurs inside a kana/kanji
    # run); the live separator is ・ (U+30FB, inside the katakana range).
    for run in _RUN_RE.findall(text):
        for alt in re.split(r"[／/・]", run):
            for i in range(len(alt)):
                for j in range(i + 2, len(alt) + 1):
                    out.add(alt[i:j])
    return out


def _signature(name: str, payload: object) -> tuple[str | None, str | None]:
    """(fixed kana, error) for one action. The kana is None when the payload
    is malformed (error set) or the action is unknown to the table (error
    None — the caller reports the table rot as a WARN)."""
    if name in ("AddPostfix", "ReplacePostfix"):
        key = "postfix" if name == "AddPostfix" else "new_postfix"
        if not isinstance(payload, dict) or not isinstance(payload.get(key), str):
            return None, f"format_map {name} missing string {key!r}: {payload!r}"
        return payload[key], None
    if name in FORMAT_ACTION_SIGNATURES:
        return FORMAT_ACTION_SIGNATURES[name], None
    return None, None


def _chain_signatures(
    actions: list[object], loc: str, report: Report
) -> list[tuple[str, str]]:
    """(action_name, signature) pairs of one POS chain. A conjugation whose
    tail is rewritten by the *immediately following* ReplacePostfix (old
    postfix == its signature) contributes nothing — that is the exact
    overwrite semantics; later-but-not-adjacent replacements keep their
    contribution. Unknown action names emit a WARN."""
    signatures: list[tuple[str, str]] = []
    for idx, action in enumerate(actions):
        if not isinstance(action, dict) or len(action) != 1:
            report.error(
                loc, f"format_map action must be a single-key object: {action!r}"
            )
            continue
        ((name, payload),) = action.items()
        if not isinstance(name, str):
            report.error(loc, f"format_map action name must be a string: {name!r}")
            continue
        signature, error = _signature(name, payload)
        if error is not None:
            report.error(loc, error)
            continue
        if signature is None:
            report.warn(
                loc,
                f"format_map action {name!r} has no signature entry — "
                f"keep _format_map_check.py in sync with FormatAction",
            )
            continue
        following = actions[idx + 1] if idx + 1 < len(actions) else None
        if (
            isinstance(following, dict)
            and len(following) == 1
            and "ReplacePostfix" in following
            and isinstance(following["ReplacePostfix"], dict)
            and following["ReplacePostfix"].get("old_postfix") == signature
        ):
            continue
        signatures.append((name, signature))
    return signatures


def check_rule_format_map(rule: dict, report: Report) -> None:
    """ERROR when a POS chain's signatures share no ≥2-char kana run with
    the rule's pattern (union across locales). One failing chain fails the
    rule; chains whose effective signature is <2 kana, and patterns that
    yield no ≥2-char kana at all, are skipped."""
    format_map = rule.get("format_map")
    if not isinstance(format_map, dict) or not format_map:
        return
    rule_id = rule.get("rule_id", "?")

    patterns = [
        content["pattern"]
        for content in (rule.get("content") or {}).values()
        if isinstance(content, dict) and isinstance(content.get("pattern"), str)
    ]
    substrings: set[str] = set()
    for pattern in patterns:
        substrings |= pattern_substrings(pattern)

    for pos, actions in format_map.items():
        loc = f"{rule_id}/format_map/{pos}"
        if not isinstance(actions, list):
            report.error(loc, f"format_map.{pos} must be a list of actions")
            continue
        signatures = _chain_signatures(actions, loc, report)
        if not any(len(sig) >= 2 for _, sig in signatures):
            continue  # effective signature <2 kana — nothing to match
        if not substrings:
            continue  # pattern yields no ≥2-char kana — nothing to match
        ok = any(
            (rule_id, name, sig) in KNOWN_GOOD  # verified-correct chain
            or any(sub in sig for sub in substrings)
            or len(sig) < 2  # single-kana tail: too weak to judge
            for name, sig in signatures
        )
        if not ok:
            tails = [sig for _, sig in signatures]
            report.error(
                loc,
                f"format_map signatures {tails!r} share no kana run with "
                f"pattern(s) {patterns!r} — anchor/content mismatch?",
            )
