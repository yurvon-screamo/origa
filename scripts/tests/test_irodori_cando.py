"""Unit tests for the Irodori → JFT-Basic Can-do mapping generator.

Covers the pure projection (`parse_entries`) and the consistency guards on
synthetic rows — no .xls fixture needed (xlrd 2.x only reads .xls, it cannot
write them). The end-to-end totals of the real dataset are asserted by the
generator itself and by the Rust domain tests on the tracked fixture
(``origa/tests/fixtures/irodori_cando.json``).
"""

from __future__ import annotations

import json

import pytest

from generate_irodori_cando import (
    CLASSROOM_LOCATION,
    EXPECTED_BAND_TOTALS,
    EXPECTED_TOTAL,
    assert_consistent,
    build_json,
    parse_entries,
)

HEADER = ["番号", "大", "小", "レベル", "Can-do", "活動", "目標", "技能", "冊／課／活動"]


def source_row(can_do_id, level, location):
    return [can_do_id, "大", "小", level, "statement", "活動", "target", "【話】", location]


def test_parse_entries_projects_all_columns_and_sorts_by_id():
    rows = [
        HEADER,
        source_row(2.0, "A1", "初級1／第3課／活動2"),
        source_row(1.0, "A1", "入門／第13課／活動5"),
        source_row(381.0, "A2", "初級2／第18課／活動1"),
    ]

    entries = parse_entries(rows[1:])

    assert [e["id"] for e in entries] == [1, 2, 381]
    assert entries[0] == {"id": 1, "level": "A1", "band": "A1", "lesson": "irodori_nyuumon_13"}
    assert entries[1] == {
        "id": 2,
        "level": "A1",
        "band": "A2.1",
        "lesson": "irodori_shokyuu1_03",
    }
    assert entries[2] == {"id": 381, "level": "A2", "band": "A2", "lesson": "irodori_shokyuu2_18"}


def test_parse_entries_skips_rows_without_number_or_location():
    rows = [
        source_row(None, "A1", "入門／第1課／活動1"),
        source_row(5.0, "A1", ""),
        source_row("", "A1", "入門／第1課／活動1"),
        source_row(6.0, "A2", "初級2／第6課／活動1"),
    ]

    entries = parse_entries(rows)

    assert [e["id"] for e in entries] == [6]


def test_parse_entries_attributes_classroom_can_dos_to_nyuumon_lesson_01():
    entries = parse_entries([source_row(312.0, "A1", CLASSROOM_LOCATION)])

    assert entries == [{"id": 312, "level": "A1", "band": "A1", "lesson": "irodori_nyuumon_01"}]


@pytest.mark.parametrize(
    "location",
    ["中級／第1課／活動1", "入門／第13課", "入門 教室の日本語", "unknown"],
)
def test_parse_entries_rejects_unrecognized_locations(location):
    with pytest.raises(ValueError, match="Unrecognized location"):
        parse_entries([source_row(1.0, "A1", location)])


def test_parse_entries_rejects_unknown_official_level():
    with pytest.raises(ValueError, match="Unexpected level"):
        parse_entries([source_row(1.0, "B1", "入門／第1課／活動1")])


def test_assert_consistent_rejects_duplicate_ids():
    entries = parse_entries(
        [
            source_row(1.0, "A1", "入門／第1課／活動1"),
            source_row(1.0, "A2", "初級2／第6課／活動1"),
        ]
    )

    with pytest.raises(AssertionError, match="duplicate"):
        assert_consistent(entries)


def test_assert_consistent_rejects_wrong_totals():
    entries = parse_entries([source_row(1.0, "A1", "入門／第1課／活動1")])

    with pytest.raises(AssertionError, match="expected 341 entries"):
        assert_consistent(entries)


def test_expected_constants_match_approved_band_semantics():
    assert EXPECTED_TOTAL == 341
    assert EXPECTED_BAND_TOTALS == {"A1": 107, "A2.1": 109, "A2": 125}


def test_parse_then_build_yields_id_ascending_document():
    """End-to-end determinism: unsorted source rows come out id-ascending.

    build_json does not sort itself — the guarantee lives in parse_entries,
    so the check must run through both functions to catch an ordering drift.
    """
    entries = parse_entries(
        [
            source_row(300.0, "A2", "初級2／第10課／活動1"),
            source_row(2.0, "A1", "入門／第1課／活動1"),
            source_row(42.0, "A2", "初級1／第5課／活動2"),
        ]
    )

    document = json.loads(build_json(entries))

    assert [entry["id"] for entry in document["can_dos"]] == [2, 42, 300]
    assert document["version"] == 1
    assert "2023-10" in document["source"]
