"""Regression tests for the format_map ↔ pattern anchor guard.

The guard exists because the v2→v3 content migration rewrote the
丁寧語 rule into ～ます／～です but kept the v2-era anchor
VerbToOKudasai (honorific お～ください), so the grammar quiz presented
お開きください as the correct "polite ます/です" construction and the
tokenizer mislabeled honorific forms. These tests pin the guard's
behavior contract: which chains must pass, which must fail, and which
are skipped as unjudgeable.

Unit tests exercise ``check_rule_format_map`` as a black box over minimal
rule dicts (the function only reads rule_id, content.pattern and
format_map). One integration test pins the schema-gate wiring through
``validate_corpus``.
"""

from __future__ import annotations

import json
from pathlib import Path

from _format_map_check import check_rule_format_map, pattern_substrings
from validate_grammar_v2 import REQUIRED_LANGS_V3, Report, validate_corpus

PROJECT_ROOT = Path(__file__).resolve().parents[2]


def _rule(rule_id: str, pattern: str, format_map: dict | None) -> dict:
    """Minimal rule shaped the way check_rule_format_map reads it."""
    rule: dict = {
        "rule_id": rule_id,
        "level": "N4",
        "content": {"English": {"pattern": pattern}},
    }
    if format_map is not None:
        rule["format_map"] = format_map
    return rule


def _check(rule: dict) -> Report:
    report = Report()
    check_rule_format_map(rule, report)
    return report


def _full_rule(rule_id: str, pattern: str, format_map: dict | None) -> dict:
    """Rule that also passes every structural check of validate_corpus
    (all four v3 locales, so the only possible error source is the anchor
    guard — structural locale errors must not mask it)."""
    content = {
        "title": pattern,
        "short_description": f"test rule {rule_id}",
        "explanation": "explanation body",
        "how_to_form": "| a | b |\n|---|---|\n| c | d |",
        "examples": "```\n例文\n```",
        "pro_tip": "",
        "nuances": {"common_mistakes": [], "notes": [{"tag": "other", "text": "note"}]},
        "pattern": pattern,
    }
    rule: dict = {
        "rule_id": rule_id,
        "level": "N4",
        "content": {lang: dict(content) for lang in REQUIRED_LANGS_V3},
    }
    if format_map is not None:
        rule["format_map"] = format_map
    return rule


# ---------------------------------------------------------------------------
# matching chains pass
# ---------------------------------------------------------------------------


def test_addpostfix_matching_pattern_tail_passes():
    report = _check(
        _rule(
            "01AAAAAAAAAAAAAAAAAAAAAAAAAA",
            "～てください",
            {"Verb": [{"VerbToTeForm": {}}, {"AddPostfix": {"postfix": "ください"}}]},
        )
    )
    assert report.errors == []


def test_conjugation_matching_pattern_tail_passes():
    report = _check(
        _rule("01AAAAAAAAAAAAAAAAAAAAAAAAAB", "～ます", {"Verb": [{"VerbToMasu": {}}]})
    )
    assert report.errors == []


def test_optionality_markers_stripped_from_pattern():
    report = _check(
        _rule(
            "01AAAAAAAAAAAAAAAAAAAAAAAAAC",
            "～（さ）せてもらえませんか",
            {
                "Verb": [
                    {"VerbToCausative": {}},
                    {"VerbToTeForm": {}},
                    {"AddPostfix": {"postfix": "もらえませんか"}},
                ]
            },
        )
    )
    assert report.errors == []


def test_bracket_qualifiers_stripped_from_pattern():
    report = _check(
        _rule(
            "01AAAAAAAAAAAAAAAAAAAAAAAAAD",
            "～ため[に]",
            {"Verb": [{"AddPostfix": {"postfix": "ため"}}]},
        )
    )
    assert report.errors == []


def test_kanji_postfix_matches_kanji_pattern_run():
    report = _check(
        _rule(
            "01AAAAAAAAAAAAAAAAAAAAAAAAAE",
            "～た結果",
            {"Verb": [{"VerbToTa": {}}, {"AddPostfix": {"postfix": "結果"}}]},
        )
    )
    assert report.errors == []


def test_known_good_entry_passes_without_pattern_overlap():
    report = _check(
        _rule("01G000000000000000KR000000", "～ておく", {"Verb": [{"VerbToToku": {}}]})
    )
    assert report.errors == []


# ---------------------------------------------------------------------------
# mismatching chains fail
# ---------------------------------------------------------------------------


def test_wrong_action_family_fails():
    """The pinned incident: polite ます／です pattern with an honorific
    お～ください anchor (the migrated 丁寧語 rule)."""
    report = _check(
        _rule(
            "01G000000000000000ZW000000",
            "～ます／～です",
            {"Verb": [{"VerbToOKudasai": {}}]},
        )
    )
    assert len(report.errors) == 1
    assert "ください" in report.errors[0]
    assert "～ます／～です" in report.errors[0]


def test_any_failing_pos_chain_fails_the_rule():
    format_map = {
        "Verb": [{"VerbToMasu": {}}],
        "NaAdjective": [{"AddPostfix": {"postfix": "限定削除"}}],
    }
    report = _check(_rule("01AAAAAAAAAAAAAAAAAAAAAAAAAF", "～ます／～です", format_map))
    assert len(report.errors) == 1
    assert "/format_map/NaAdjective" in report.errors[0]


# ---------------------------------------------------------------------------
# skip rules: chains and patterns the heuristic must not judge
# ---------------------------------------------------------------------------


def test_single_kana_chain_is_skipped_silently():
    report = _check(
        _rule(
            "01AAAAAAAAAAAAAAAAAAAAAAAAAG", "～おかげだ", {"Verb": [{"VerbToTa": {}}]}
        )
    )
    assert report.errors == []
    assert report.warnings == []  # た is a known 1-kana signature, not table rot


def test_pattern_without_two_char_kana_is_skipped():
    report = _check(
        _rule(
            "01AAAAAAAAAAAAAAAAAAAAAAAAAH",
            "～（よ）う",
            {"Verb": [{"VerbToVolitional": {}}]},
        )
    )
    assert report.errors == []


def test_action_consumed_by_later_replacepostfix_is_ignored():
    format_map = {
        "Verb": [
            {"VerbToNai": {}},
            {
                "ReplacePostfix": {
                    "old_postfix": "ない",
                    "new_postfix": "なければなりません",
                }
            },
        ]
    }
    report = _check(
        _rule("01AAAAAAAAAAAAAAAAAAAAAAAAAI", "～なければなりません", format_map)
    )
    assert report.errors == []


# ---------------------------------------------------------------------------
# table rot detection
# ---------------------------------------------------------------------------


def test_action_missing_from_signature_table_warns():
    report = _check(
        _rule(
            "01AAAAAAAAAAAAAAAAAAAAAAAAAJ",
            "～ます",
            {"Verb": [{"VerbToFutureTense": {}}]},
        )
    )
    assert report.errors == []
    assert len(report.warnings) == 1
    assert "VerbToFutureTense" in report.warnings[0]


def test_malformed_postfix_payload_is_an_error_not_table_rot():
    report = _check(
        _rule(
            "01AAAAAAAAAAAAAAAAAAAAAAAAAK",
            "～ます",
            {"Verb": [{"AddPostfix": {}}]},
        )
    )
    assert len(report.errors) == 1
    assert "missing string 'postfix'" in report.errors[0]
    assert report.warnings == []


# ---------------------------------------------------------------------------
# integration: schema gate + real corpus
# ---------------------------------------------------------------------------


def test_anchor_check_runs_only_on_schema_v3():
    from validate_grammar_v2 import REQUIRED_LANGS

    format_map = {"Verb": [{"VerbToOKudasai": {}}]}
    for schema, expect_anchor_error in ((3, True), (2, False)):
        data = {
            "schema": schema,
            "grammar": [
                _full_rule("01G000000000000000ZW000000", "～ます／～です", format_map)
            ],
        }
        required = REQUIRED_LANGS_V3 if schema == 3 else REQUIRED_LANGS
        report = Report()
        validate_corpus(data, report, required, check_anchors=schema == 3)
        anchor_errors = [e for e in report.errors if "share no kana run" in e]
        # The anchor guard — not structural validation — decides the outcome.
        assert bool(anchor_errors) == expect_anchor_error, f"schema={schema}"
        assert len(report.errors) == len(anchor_errors), f"schema={schema}"


def test_shipped_v3_corpus_has_no_anchor_errors():
    """Positive corpus-wide proof: the deployed corpus (post-fix) passes the
    anchor guard. Skipped where the gitignored cdn/ store is absent (CI)."""
    corpus = PROJECT_ROOT / "cdn" / "grammar" / "grammar_v3.json"
    if not corpus.exists():
        import pytest

        pytest.skip("gitignored cdn/grammar/grammar_v3.json not present")
    data = json.loads(corpus.read_text(encoding="utf-8").lstrip("﻿"))
    report = Report()
    validate_corpus(data, report, REQUIRED_LANGS_V3, check_anchors=True)
    assert report.errors == []


def test_pattern_substrings_handles_optionality_and_alternatives():
    assert pattern_substrings("～（よ）う") == set()
    assert "ため" in pattern_substrings("～ため[に]")
    assert "ましょう" in pattern_substrings("～ましょう／～よう")
