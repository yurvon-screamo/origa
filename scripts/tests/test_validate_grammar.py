"""Unit tests for ``validate_grammar.py``.

Covers the grammar-rule schema validator against inline JSON fixtures so the
tests have no dependency on the gitignored ``cdn/`` store. The
``FormatAction`` whitelist parser is exercised against the real Rust source
(``origa/src/dictionary/grammar.rs``) since that is the contract the parser
exists to enforce — a drift between the Rust enum and the Python whitelist is
exactly the regression this guard must catch.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import pytest

from validate_grammar import (
    DEFAULT_WHITELIST_SOURCE,
    MIN_EXPECTED_VARIANTS,
    REQUIRED_CONTENT_FIELDS,
    ULID_PATTERN,
    VALID_LEVELS,
    ValidationReport,
    _check_detection_anchor,
    _check_duplicate_rule_ids,
    _check_duplicate_titles,
    _check_format_map,
    _check_keywords,
    _check_level,
    _check_title_parity,
    _check_ulid,
    load_rules,
    parse_format_action_whitelist,
    validate_rule,
)

WORKSPACE_ROOT = Path(__file__).resolve().parents[2]
REAL_GRAMMAR_RS = WORKSPACE_ROOT / "origa" / "src" / "dictionary" / "grammar.rs"

VALID_ULID = "01KV2C0RKHWT3G177AW3GPKJA6"


# ---------------------------------------------------------------------------
# Fixtures / builders
# ---------------------------------------------------------------------------


def _content(title_en: str = "Test", title_ru: str = "Тест") -> dict[str, Any]:
    """Return a minimal valid bilingual content block."""
    base = {field: f"{field}-value" for field in REQUIRED_CONTENT_FIELDS}
    return {
        "Russian": {**base, "title": title_ru},
        "English": {**base, "title": title_en},
    }


def _rule(
    rule_id: str = VALID_ULID,
    level: str = "N3",
    content: dict[str, Any] | None = None,
    format_map: dict[str, Any] | None = None,
    keywords: list[Any] | None = None,
) -> dict[str, Any]:
    """Build a rule dict; only the requested anchor field is attached."""
    rule: dict[str, Any] = {
        "rule_id": rule_id,
        "level": level,
        "content": content if content is not None else _content(),
    }
    if format_map is not None:
        rule["format_map"] = format_map
    if keywords is not None:
        rule["keywords"] = keywords
    return rule


@pytest.fixture
def real_whitelist() -> dict[str, list[str]]:
    return parse_format_action_whitelist(REAL_GRAMMAR_RS)


# ---------------------------------------------------------------------------
# parse_format_action_whitelist
# ---------------------------------------------------------------------------


class TestParseWhitelist:
    def test_real_grammar_rs_yields_at_least_48_variants(self, real_whitelist):
        assert len(real_whitelist) >= 48, (
            f"expected ≥48 FormatAction variants, parsed {len(real_whitelist)}"
        )

    def test_struct_variants_carry_their_fields(self, real_whitelist):
        # AddPostfix / ReplacePostfix / RemovePostfix are the only struct-like
        # variants in grammar.rs; their field names are the contract every
        # per-rule format_map author depends on.
        assert real_whitelist["AddPostfix"] == ["postfix"]
        assert sorted(real_whitelist["ReplacePostfix"]) == sorted(
            ["old_postfix", "new_postfix"]
        )
        assert real_whitelist["RemovePostfix"] == ["postfix"]

    def test_unit_variants_parse_with_empty_field_list(self, real_whitelist):
        assert real_whitelist["VerbToTeForm"] == []
        assert real_whitelist["VerbToMizenkei"] == []

    def test_missing_enum_raises_value_error(self, tmp_path):
        bogus = tmp_path / "no_enum.rs"
        bogus.write_text("// no FormatAction here", encoding="utf-8")

        with pytest.raises(ValueError, match="FormatAction enum not found"):
            parse_format_action_whitelist(bogus)

    def test_suspiciously_small_parse_raises(self, tmp_path):
        # Two variants only — below MIN_EXPECTED_VARIANTS, must hard-fail so a
        # regex silently breaking on a formatting change cannot slip through.
        small_enum = tmp_path / "small.rs"
        small_enum.write_text(
            "enum FormatAction {\n    Foo {}\n    Bar {}\n}\n", encoding="utf-8"
        )

        with pytest.raises(ValueError, match="regex may need updating"):
            parse_format_action_whitelist(small_enum)


# ---------------------------------------------------------------------------
# ULID validation (regex-based)
# ---------------------------------------------------------------------------


class TestUlidPattern:
    @pytest.mark.parametrize(
        "candidate",
        [
            VALID_ULID,
            "01H00000000000000000000000",
            "7ZZZZZZZZZZZZZZZZZZZZZZZZZ",
        ],
    )
    def test_valid_ulids_match(self, candidate):
        assert ULID_PATTERN.match(candidate) is not None

    @pytest.mark.parametrize(
        "candidate",
        [
            "",            # empty
            "ABC",         # too short
            "01KV2C0RKHWT3G177AW3GPKJA",   # 25 chars
            "01KV2C0RKHWT3G177AW3GPKJA66",  # 27 chars
            "0I1O0U0A0B0C0D0E0F0G0H0J0K",   # Crockford-excluded I/O/U present
            "01KV2C0RKHWT3G177AW3GPKJA!",   # punctuation
        ],
    )
    def test_invalid_ulids_rejected(self, candidate):
        assert ULID_PATTERN.match(candidate) is None

    def test_lowercase_rejected(self):
        # Crockford base32 is uppercase only.
        assert ULID_PATTERN.match(VALID_ULID.lower()) is None


class TestCheckUlid:
    def test_valid_ulid_no_error(self):
        report = ValidationReport()
        _check_ulid(VALID_ULID, report, index=0)
        assert report.ok
        assert len(report.warnings) == 0

    def test_invalid_ulid_emits_error(self):
        report = ValidationReport()
        _check_ulid("not-a-ulid", report, index=3)
        assert not report.ok
        assert len(report.errors) == 1
        assert "rule[3]" in report.errors[0]

    def test_non_string_ulid_emits_error(self):
        report = ValidationReport()
        _check_ulid(12345, report, index=1)  # type: ignore[arg-type]
        assert not report.ok


# ---------------------------------------------------------------------------
# Level validation
# ---------------------------------------------------------------------------


class TestCheckLevel:
    @pytest.mark.parametrize("level", sorted(VALID_LEVELS))
    def test_all_canonical_levels_accepted(self, level):
        report = ValidationReport()
        _check_level(level, report, index=0)
        assert report.ok

    @pytest.mark.parametrize("level", ["N6", "N0", "n5", "", "M3", None, 5])
    def test_invalid_levels_rejected(self, level):
        report = ValidationReport()
        _check_level(level, report, index=2)  # type: ignore[arg-type]
        assert not report.ok
        assert "rule[2]" in report.errors[0]


# ---------------------------------------------------------------------------
# Detection anchor suppression (N5/N4 carve-out)
# ---------------------------------------------------------------------------


class TestDetectionAnchor:
    def test_n3_without_anchor_errors(self):
        report = ValidationReport()
        _check_detection_anchor(
            rule={}, level="N3", has_format_map=False, has_keywords=False,
            index=0, report=report,
        )
        assert not report.ok
        assert "required for level N3" in report.errors[0]

    def test_n2_without_anchor_errors(self):
        report = ValidationReport()
        _check_detection_anchor(
            rule={}, level="N2", has_format_map=False, has_keywords=False,
            index=0, report=report,
        )
        assert not report.ok

    def test_n1_without_anchor_errors(self):
        report = ValidationReport()
        _check_detection_anchor(
            rule={}, level="N1", has_format_map=False, has_keywords=False,
            index=0, report=report,
        )
        assert not report.ok

    def test_n5_without_anchor_suppressed(self):
        # N5/N4 carry reference rules (basic particles, categories) that are
        # pedagogical reference material, not text-detection targets. This is
        # an intentional carve-out — re-enabling it requires a content review.
        report = ValidationReport()
        _check_detection_anchor(
            rule={}, level="N5", has_format_map=False, has_keywords=False,
            index=0, report=report,
        )
        assert report.ok
        assert len(report.warnings) == 0

    def test_n4_without_anchor_suppressed(self):
        report = ValidationReport()
        _check_detection_anchor(
            rule={}, level="N4", has_format_map=False, has_keywords=False,
            index=0, report=report,
        )
        assert report.ok
        assert len(report.warnings) == 0

    def test_n5_with_keywords_satisfies_anchor(self):
        report = ValidationReport()
        _check_detection_anchor(
            rule={}, level="N5", has_format_map=False, has_keywords=True,
            index=0, report=report,
        )
        assert report.ok

    def test_n3_with_format_map_satisfies_anchor(self):
        report = ValidationReport()
        _check_detection_anchor(
            rule={}, level="N3", has_format_map=True, has_keywords=False,
            index=0, report=report,
        )
        assert report.ok


# ---------------------------------------------------------------------------
# load_rules auto-detection
# ---------------------------------------------------------------------------


class TestLoadRules:
    def test_final_store_format(self, tmp_path):
        path = tmp_path / "grammar.json"
        path.write_text(
            '{"grammar": [{"rule_id": "x", "level": "N5"}]}', encoding="utf-8"
        )
        assert load_rules(path) == [{"rule_id": "x", "level": "N5"}]

    def test_legacy_chunk_format(self, tmp_path):
        path = tmp_path / "chunk.json"
        path.write_text(
            '{"metadata": {"id": "c1"}, "rules": [{"rule_id": "y"}]}',
            encoding="utf-8",
        )
        assert load_rules(path) == [{"rule_id": "y"}]

    def test_single_rule_wrapped_in_list(self, tmp_path):
        path = tmp_path / "rule.json"
        path.write_text('{"rule_id": "z", "content": {}}', encoding="utf-8")
        result = load_rules(path)
        assert isinstance(result, list)
        assert len(result) == 1
        assert result[0]["rule_id"] == "z"

    def test_directory_concatenates_all_json(self, tmp_path):
        (tmp_path / "a.json").write_text(
            '{"grammar": [{"rule_id": "a"}]}', encoding="utf-8"
        )
        nested = tmp_path / "sub"
        nested.mkdir()
        (nested / "b.json").write_text(
            '{"grammar": [{"rule_id": "b"}]}', encoding="utf-8"
        )
        rules = load_rules(tmp_path)
        ids = [r["rule_id"] for r in rules]
        assert sorted(ids) == ["a", "b"]

    def test_unrecognized_structure_raises_value_error(self, tmp_path):
        path = tmp_path / "weird.json"
        path.write_text('{"foo": 1}', encoding="utf-8")
        with pytest.raises(ValueError, match="Unrecognized structure"):
            load_rules(path)

    def test_non_list_grammar_raises(self, tmp_path):
        path = tmp_path / "bad.json"
        path.write_text('{"grammar": "not-a-list"}', encoding="utf-8")
        with pytest.raises(ValueError, match="'grammar' must be a list"):
            load_rules(path)


# ---------------------------------------------------------------------------
# Duplicate rule_id detection
# ---------------------------------------------------------------------------


class TestDuplicateRuleIds:
    def test_duplicate_rule_id_emits_error(self):
        report = ValidationReport()
        rules = [
            {"rule_id": VALID_ULID, "level": "N3"},
            {"rule_id": VALID_ULID, "level": "N2"},
        ]
        _check_duplicate_rule_ids(rules, report)
        assert not report.ok
        assert len(report.errors) == 1
        assert "duplicate rule_id" in report.errors[0]
        # Positional: the second occurrence is reported.
        assert "rule[1]" in report.errors[0]
        assert "rule[0]" in report.errors[0]

    def test_unique_rule_ids_pass(self):
        report = ValidationReport()
        rules = [
            {"rule_id": "01KV2C0RKHWT3G177AW3GPKJA6"},
            {"rule_id": "01KV2BV4G2TVKG953YH43902Z7"},
        ]
        _check_duplicate_rule_ids(rules, report)
        assert report.ok

    def test_non_string_rule_id_ignored(self):
        report = ValidationReport()
        rules = [{"rule_id": 123}, {"rule_id": 123}]  # type: ignore[dict-item]
        _check_duplicate_rule_ids(rules, report)
        assert report.ok


# ---------------------------------------------------------------------------
# Title parity (cross-language title diff)
# ---------------------------------------------------------------------------


class TestTitleParity:
    def test_strict_level_title_mismatch_is_error(self):
        report = ValidationReport()
        _check_title_parity(
            ru={"title": "Тест"},
            en={"title": "Test"},
            level="N3",
            index=0,
            report=report,
        )
        assert not report.ok

    def test_legacy_level_title_mismatch_is_warning(self):
        report = ValidationReport()
        _check_title_parity(
            ru={"title": "Тест"},
            en={"title": "Test"},
            level="N5",
            index=0,
            report=report,
        )
        assert report.ok  # warnings do not fail the report
        assert len(report.warnings) == 1
        assert "legacy N5/N4" in report.warnings[0]

    def test_matching_titles_pass_silently(self):
        report = ValidationReport()
        _check_title_parity(
            ru={"title": "同じ"},
            en={"title": "同じ"},
            level="N3",
            index=0,
            report=report,
        )
        assert report.ok
        assert len(report.warnings) == 0

    def test_missing_title_skipped(self):
        report = ValidationReport()
        _check_title_parity(
            ru={}, en={"title": "x"}, level="N3", index=0, report=report
        )
        assert report.ok


# ---------------------------------------------------------------------------
# format_map structure checks
# ---------------------------------------------------------------------------


class TestFormatMapCheck:
    def test_known_action_with_required_field_passes(self, real_whitelist):
        report = ValidationReport()
        rule = {
            "format_map": {
                "Verb": [{"AddPostfix": {"postfix": "ざる"}}],
            }
        }
        assert _check_format_map(rule, real_whitelist, 0, report) is True
        assert report.ok

    def test_unknown_action_rejected(self, real_whitelist):
        report = ValidationReport()
        rule = {
            "format_map": {
                "Verb": [{"VerbToBogus": {}}],
            }
        }
        _check_format_map(rule, real_whitelist, 0, report)
        assert not report.ok
        assert "unknown action 'VerbToBogus'" in report.errors[0]

    def test_missing_required_field_rejected(self, real_whitelist):
        report = ValidationReport()
        rule = {
            "format_map": {
                "Verb": [{"AddPostfix": {}}],  # postfix missing
            }
        }
        _check_format_map(rule, real_whitelist, 0, report)
        assert not report.ok

    def test_empty_format_map_rejected(self, real_whitelist):
        report = ValidationReport()
        rule = {"format_map": {}}
        _check_format_map(rule, real_whitelist, 0, report)
        assert not report.ok
        assert "present but empty" in report.errors[0]

    def test_invalid_pos_key_rejected(self, real_whitelist):
        report = ValidationReport()
        rule = {
            "format_map": {
                "Noun": [{"AddPostfix": {"postfix": "x"}}],
            }
        }
        _check_format_map(rule, real_whitelist, 0, report)
        assert not report.ok

    def test_format_map_absent_returns_false(self, real_whitelist):
        report = ValidationReport()
        assert _check_format_map({}, real_whitelist, 0, report) is False
        assert report.ok


# ---------------------------------------------------------------------------
# keywords structure checks
# ---------------------------------------------------------------------------


class TestKeywordsCheck:
    def test_valid_keywords_pass(self):
        report = ValidationReport()
        rule = {"keywords": [["こと"], ["できる"]]}
        assert _check_keywords(rule, 0, report) is True
        assert report.ok

    def test_empty_keywords_rejected(self):
        report = ValidationReport()
        rule = {"keywords": []}
        _check_keywords(rule, 0, report)
        assert not report.ok

    def test_empty_inner_group_rejected(self):
        report = ValidationReport()
        rule = {"keywords": [[]]}
        _check_keywords(rule, 0, report)
        assert not report.ok

    def test_non_string_item_rejected(self):
        report = ValidationReport()
        rule = {"keywords": [[123]]}  # type: ignore[list-item]
        _check_keywords(rule, 0, report)
        assert not report.ok

    def test_keywords_absent_returns_false(self):
        report = ValidationReport()
        assert _check_keywords({}, 0, report) is False
        assert report.ok


# ---------------------------------------------------------------------------
# Full-rule integration: validate_rule
# ---------------------------------------------------------------------------


class TestValidateRuleIntegration:
    def test_minimal_valid_n3_rule_passes(self, real_whitelist):
        # N3 is a strict level: title parity must hold AND a detection anchor
        # (format_map or keywords) is mandatory. The minimal valid N3 rule
        # therefore needs matching titles + at least one anchor.
        rule = _rule(
            level="N3",
            content=_content(title_en="同じ", title_ru="同じ"),
            format_map={"Verb": [{"AddPostfix": {"postfix": "ざる"}}]},
        )
        report = ValidationReport()
        validate_rule(rule, real_whitelist, index=0, report=report)
        assert report.ok, f"unexpected errors: {report.errors}"

    def test_n5_reference_rule_without_anchor_passes(self, real_whitelist):
        # The N5/N4 carve-out: a reference rule with no detection anchor and
        # mismatched titles is accepted (title mismatch becomes a warning).
        rule = _rule(
            level="N5",
            content=_content(title_en="X", title_ru="Игрек"),
        )
        report = ValidationReport()
        validate_rule(rule, real_whitelist, index=0, report=report)
        assert report.ok  # warnings don't fail the report
        assert len(report.warnings) == 1  # title mismatch downgraded to warn

    def test_rule_missing_content_errors(self, real_whitelist):
        report = ValidationReport()
        rule = {"rule_id": VALID_ULID, "level": "N3"}
        level, title = validate_rule(rule, real_whitelist, index=0, report=report)
        assert level == "N3"
        assert title == ""
        assert not report.ok

    def test_rule_returns_en_title_for_dedup(self, real_whitelist):
        report = ValidationReport()
        rule = _rule(
            content=_content(title_en="MyTitle", title_ru="MyTitle"),
            format_map={"Verb": [{"AddPostfix": {"postfix": "x"}}]},
        )
        level, title = validate_rule(rule, real_whitelist, index=0, report=report)
        assert level == "N3"
        assert title == "MyTitle"


# ---------------------------------------------------------------------------
# Duplicate (level, title) detection
# ---------------------------------------------------------------------------


class TestDuplicateTitles:
    def test_duplicate_level_title_emits_warning(self):
        report = ValidationReport()
        rules = [
            {"level": "N3", "content": {"English": {"title": "A"}}},
            {"level": "N3", "content": {"English": {"title": "A"}}},
        ]
        _check_duplicate_titles(rules, report)
        assert report.ok  # warnings don't fail
        assert len(report.warnings) == 1

    def test_different_levels_no_warning(self):
        report = ValidationReport()
        rules = [
            {"level": "N3", "content": {"English": {"title": "A"}}},
            {"level": "N2", "content": {"English": {"title": "A"}}},
        ]
        _check_duplicate_titles(rules, report)
        assert len(report.warnings) == 0


# ---------------------------------------------------------------------------
# DEFAULT_WHITELIST_SOURCE sanity
# ---------------------------------------------------------------------------


def test_default_whitelist_source_points_to_grammar_rs():
    # Drift here would break the CLI invocation for operators who rely on the
    # default rather than passing --whitelist-source explicitly.
    assert DEFAULT_WHITELIST_SOURCE.name == "grammar.rs"
    assert MIN_EXPECTED_VARIANTS >= 30
