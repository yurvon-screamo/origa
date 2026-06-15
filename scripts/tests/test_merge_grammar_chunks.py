"""Unit tests for ``merge_grammar_chunks.py``.

Covers the patch-or-append merge logic, the level → lesson → title sort, and
the tooling-field stripping that keeps the canonical ``grammar.json`` schema
homogeneous with the legacy N5/N4 rules (which never carried ``chunk_id`` /
``lesson``). All fixtures are inline dicts — no dependency on the gitignored
``cdn/`` store.
"""

from __future__ import annotations

from typing import Any

from merge_grammar_chunks import (
    LEVEL_ORDER,
    TOOLING_ONLY_FIELDS,
    MergeReportContext,
    ValidationReport,
    _sort_key,
    _strip_tooling_fields,
    _verify_chunk_id_consistency,
    apply_patches_in_place,
    build_report,
    patch_or_append_rules,
    sort_merged,
    warn_on_duplicate_titles,
)


def _rule(
    rule_id: str,
    level: str = "N3",
    title: str = "T",
    lesson: int | None = None,
    chunk_id: str | None = None,
) -> dict[str, Any]:
    rule: dict[str, Any] = {
        "rule_id": rule_id,
        "level": level,
        "content": {"English": {"title": title}},
    }
    if lesson is not None:
        rule["lesson"] = lesson
    if chunk_id is not None:
        rule["chunk_id"] = chunk_id
    return rule


# ---------------------------------------------------------------------------
# patch_or_append_rules
# ---------------------------------------------------------------------------


class TestPatchOrAppend:
    def test_existing_rule_id_patches_in_place(self):
        existing = [_rule("BASE1", level="N5"), _rule("BASE2", level="N5")]
        replacement = _rule("BASE1", level="N5", title="Patched")
        report = ValidationReport()

        appended, patch_map = patch_or_append_rules(
            existing, [replacement], report
        )

        assert report.ok
        assert appended == []  # nothing new
        assert patch_map == {0: replacement}
        # apply_patches_in_place performs the actual replacement.
        apply_patches_in_place(existing, patch_map)
        assert existing[0]["content"]["English"]["title"] == "Patched"
        assert existing[1]["content"]["English"]["title"] == "T"

    def test_new_rule_id_is_appended(self):
        existing = [_rule("BASE1")]
        new_rule = _rule("NEW1")
        report = ValidationReport()

        appended, patch_map = patch_or_append_rules(
            existing, [new_rule], report
        )

        assert report.ok
        assert appended == [new_rule]
        assert patch_map == {}

    def test_cross_file_duplicate_rule_id_is_hard_error(self):
        existing = [_rule("BASE1")]
        # Same rule_id in two new rule files — the author must disambiguate.
        dup_a = _rule("DUP")
        dup_b = _rule("DUP")
        report = ValidationReport()

        appended, _ = patch_or_append_rules(
            existing, [dup_a, dup_b], report
        )

        assert not report.ok
        assert len(report.errors) == 1
        assert "Duplicate rule_id 'DUP'" in report.errors[0]
        # The first occurrence still lands in appended; the second is dropped.
        assert appended == [dup_a]

    def test_patch_then_append_in_same_call(self):
        existing = [_rule("BASE1")]
        rules = [_rule("BASE1", title="P"), _rule("NEW1")]
        report = ValidationReport()

        appended, patch_map = patch_or_append_rules(existing, rules, report)

        assert report.ok
        assert len(appended) == 1
        assert appended[0]["rule_id"] == "NEW1"
        assert 0 in patch_map


# ---------------------------------------------------------------------------
# sort key + sort_merged
# ---------------------------------------------------------------------------


class TestSortKey:
    def test_n5_sorts_before_n3(self):
        assert _sort_key(_rule("a", level="N5")) < _sort_key(_rule("b", level="N3"))

    def test_n3_sorts_before_n2(self):
        assert _sort_key(_rule("a", level="N3")) < _sort_key(_rule("b", level="N2"))

    def test_within_level_lesson_orders_rules(self):
        early = _sort_key(_rule("a", level="N3", lesson=1))
        late = _sort_key(_rule("b", level="N3", lesson=5))
        assert early < late

    def test_absent_lesson_sorts_last_within_level(self):
        with_lesson = _sort_key(_rule("a", level="N3", lesson=2))
        without = _sort_key(_rule("b", level="N3"))  # no lesson
        assert with_lesson < without

    def test_title_breaks_ties(self):
        a = _sort_key(_rule("a", level="N3", lesson=1, title="Aaa"))
        z = _sort_key(_rule("z", level="N3", lesson=1, title="Zzz"))
        assert a < z

    def test_unknown_level_sorts_last(self):
        assert LEVEL_ORDER["N1"] == 4
        assert _sort_key(_rule("x", level="N1")) < _sort_key(
            _rule("y", level="N0")  # N0 unknown → rank 99
        )


class TestSortMerged:
    def test_existing_order_preserved_appended_sorted(self):
        existing = [_rule("BASE2"), _rule("BASE1")]  # order preserved verbatim
        appended = [
            _rule("N2", level="N2"),
            _rule("N3", level="N3"),
        ]
        merged = sort_merged(existing, appended)
        # Existing block first, in original order.
        assert [r["rule_id"] for r in merged[:2]] == ["BASE2", "BASE1"]
        # Appended block sorted N3 before N2.
        assert [r["rule_id"] for r in merged[2:]] == ["N3", "N2"]


# ---------------------------------------------------------------------------
# _strip_tooling_fields
# ---------------------------------------------------------------------------


class TestStripToolingFields:
    def test_chunk_id_and_lesson_removed(self):
        rule = _rule("R", chunk_id="n3_01", lesson=3)
        cleaned = _strip_tooling_fields(rule)
        assert "chunk_id" not in cleaned
        assert "lesson" not in cleaned
        # Canonical fields preserved.
        assert cleaned["rule_id"] == "R"
        assert cleaned["level"] == "N3"

    def test_rule_without_tooling_fields_unchanged(self):
        rule = _rule("R")
        cleaned = _strip_tooling_fields(rule)
        assert cleaned == rule

    def test_tooling_fields_constant_is_stable(self):
        # A future contributor adding a field here must understand WHY it is
        # tooling-only; pinning the set guards against accidental schema drift.
        assert set(TOOLING_ONLY_FIELDS) == {"chunk_id", "lesson"}


# ---------------------------------------------------------------------------
# _verify_chunk_id_consistency
# ---------------------------------------------------------------------------


class TestVerifyChunkIdConsistency:
    def test_consistent_chunk_id_returned(self):
        report = ValidationReport()
        rules = [
            _rule("R1", chunk_id="n3_01"),
            _rule("R2", chunk_id="n3_01"),
        ]
        cid = _verify_chunk_id_consistency("n3_01", rules, report)
        assert cid == "n3_01"
        assert report.ok
        assert len(report.warnings) == 0

    def test_inconsistent_chunk_ids_emit_warning(self):
        report = ValidationReport()
        rules = [
            _rule("R1", chunk_id="n3_01"),
            _rule("R2", chunk_id="n3_02"),
        ]
        cid = _verify_chunk_id_consistency("mixed", rules, report)
        # First observed wins (documented behavior).
        assert cid == "n3_01"
        assert report.ok  # warning, not error
        assert len(report.warnings) == 1
        assert "different chunk_id values" in report.warnings[0]

    def test_no_chunk_id_returns_none(self):
        report = ValidationReport()
        rules = [_rule("R1")]  # no chunk_id at all
        assert _verify_chunk_id_consistency("x", rules, report) is None
        assert report.ok


# ---------------------------------------------------------------------------
# warn_on_duplicate_titles
# ---------------------------------------------------------------------------


class TestWarnOnDuplicateTitles:
    def test_cross_file_duplicate_title_warns(self):
        report = ValidationReport()
        new_rules = [
            _rule("R1", level="N3", title="Same"),
            _rule("R2", level="N3", title="Same"),
        ]
        warn_on_duplicate_titles(new_rules, report)
        assert report.ok
        assert len(report.warnings) == 1
        assert "Duplicate" in report.warnings[0]

    def test_different_titles_no_warning(self):
        report = ValidationReport()
        new_rules = [
            _rule("R1", title="A"),
            _rule("R2", title="B"),
        ]
        warn_on_duplicate_titles(new_rules, report)
        assert len(report.warnings) == 0


# ---------------------------------------------------------------------------
# build_report (smoke — guards the report-formatting contract)
# ---------------------------------------------------------------------------


class TestBuildReport:
    def test_report_contains_core_counts(self, tmp_path, capsys):
        ctx = MergeReportContext(
            existing_count=10,
            total_count=42,
            rules_per_chunk={"n3_01": [tmp_path / "rule_01.json"]},
            chunk_id_per_subdir={"n3_01": "n3_01"},
            legacy_chunks=[],
            new_count=30,
            patched_count=2,
        )
        report = ValidationReport()
        text = build_report(ctx, report)
        assert "Base rules (existing):       10" in text
        assert "Appended new rules:          30" in text
        assert "Patched existing rules:      2" in text
        assert "Total after merge:           42" in text
