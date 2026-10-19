"""Generate the Irodori → JFT-Basic Can-do mapping JSON (issue #760).

Source: the official Japan Foundation correspondence list "JF Can-do for
Life in Japan → Irodori target Can-dos" (2023-10 revision), committed at
``scripts/data/irodori_cando.xls``. It maps 341 of the 381 framework
Can-dos to Irodori books/lessons; every mapped Can-do resolves to exactly
one lesson, which makes lesson import status a valid coverage signal.

Band semantics (approved in #760): the three JFT-Basic scoring bands
(145-174 = A1, 175-199 = A2.1, 200-250 = A2.2/A2) follow the Irodori
course progression — 入門 → A1, 初級1 → A2.1, 初級2 → A2. The official
per-Can-do level tag (A1/A2) is preserved in ``level`` for future use.

Outputs (byte-identical, single run):
  1. ``cdn/well_known_set/irodori_cando.json``   — deployed CDN artifact.
  2. ``origa/tests/fixtures/irodori_cando.json`` — tracked fixture consumed
     by ``origa/src/domain/jft_cando.rs`` tests via ``include_str!``; the
     tests assert band totals, so data drift fails them loudly.

Usage: ``uv run generate_irodori_cando.py`` (from ``scripts/``).
"""

from __future__ import annotations

import json
import re
from collections.abc import Iterable, Sequence
from pathlib import Path

import xlrd

PROJECT_ROOT = Path(__file__).resolve().parent.parent
EXCEL_PATH = PROJECT_ROOT / "scripts" / "data" / "irodori_cando.xls"
CDN_OUTPUT = PROJECT_ROOT / "cdn" / "well_known_set" / "irodori_cando.json"
FIXTURE_OUTPUT = PROJECT_ROOT / "origa" / "tests" / "fixtures" / "irodori_cando.json"

SOURCE_LABEL = "JF Can-do for Life in Japan -> Irodori, 2023-10 revision"
FORMAT_VERSION = 1

# Rows 0-2 are title/notes; row 3 is the column header; data starts at row 4.
HEADER_ROWS = 4

# Book column value -> (set-id infix, UI band). The infix matches the CDN
# well-known set ids: irodori_{infix}_{lesson:02d}.
BOOK_MAP: dict[str, tuple[str, str]] = {
    "入門": ("nyuumon", "A1"),
    "初級1": ("shokyuu1", "A2.1"),
    "初級2": ("shokyuu2", "A2"),
}

# Matches "入門／第13課／活動5". The activity number is dropped: coverage is
# tracked at lesson granularity (one imported lesson covers all of its
# Can-dos regardless of which activity introduced them).
LOCATION_RE = re.compile(r"^(入門|初級1|初級2)／第(\d+)課／活動\d+$")

# Two Can-dos (#312, #314) are listed under 入門／教室の日本語 ("classroom
# Japanese", the front-matter section of the Nyuumon book) with no specific
# lesson. They are attributed to lesson 01 so all 341 mapped Can-dos stay
# trackable through imports; the official list ties them to the book, not
# to a lesson, so this is the coarsest faithful assignment.
CLASSROOM_LOCATION = "入門／教室の日本語"
CLASSROOM_FALLBACK_LESSON = 1

EXPECTED_TOTAL = 341
EXPECTED_BAND_TOTALS = {"A1": 107, "A2.1": 109, "A2": 125}


def cell_str(value: object) -> str:
    """Normalize an xlrd cell (floats for numeric columns) to a strip'd str."""
    if value is None:
        return ""
    if isinstance(value, float) and value.is_integer():
        return str(int(value))
    return str(value).strip()


def is_blank_cell(value: object) -> bool:
    """Blank in the source sense: no cell, empty text, or numeric zero.

    xlrd returns empty numeric cells as ``0.0``; Can-do ids start at 1, so a
    zero id column means the row carries no framework Can-do.
    """
    return value is None or value == 0 or (isinstance(value, str) and not value.strip())


def parse_entries(rows: Iterable[Sequence[object]]) -> list[dict]:
    """Project source rows into sorted mapping entries.

    Each entry: {"id": int, "level": "A1"|"A2", "band": band, "lesson": set-id}.
    Rows without a Can-do number or without an Irodori location (40 framework
    Can-dos are outside Irodori) are skipped.
    """
    entries: list[dict] = []
    for row in rows:
        if len(row) < 9:
            continue
        if is_blank_cell(row[0]) or is_blank_cell(row[8]):
            continue
        can_do_id = cell_str(row[0])
        level = cell_str(row[3])
        location = cell_str(row[8])

        match = LOCATION_RE.match(location)
        if match:
            book, lesson = match.group(1), int(match.group(2))
        elif location == CLASSROOM_LOCATION:
            book, lesson = "入門", CLASSROOM_FALLBACK_LESSON
        else:
            raise ValueError(f"Unrecognized location {location!r} (Can-do {can_do_id})")

        if level not in ("A1", "A2"):
            raise ValueError(f"Unexpected level {level!r} (Can-do {can_do_id})")
        infix, band = BOOK_MAP[book]
        entries.append(
            {
                "id": int(can_do_id),
                "level": level,
                "band": band,
                "lesson": f"irodori_{infix}_{lesson:02d}",
            }
        )

    entries.sort(key=lambda e: e["id"])
    return entries


def assert_consistent(entries: list[dict]) -> None:
    ids = [e["id"] for e in entries]
    if len(ids) != len(set(ids)):
        raise AssertionError("duplicate Can-do ids in source")
    if len(entries) != EXPECTED_TOTAL:
        raise AssertionError(f"expected {EXPECTED_TOTAL} entries, got {len(entries)}")
    totals: dict[str, int] = {}
    for entry in entries:
        totals[entry["band"]] = totals.get(entry["band"], 0) + 1
    if totals != EXPECTED_BAND_TOTALS:
        raise AssertionError(f"band totals {totals} != {EXPECTED_BAND_TOTALS}")


def build_json(entries: list[dict]) -> str:
    document = {
        "version": FORMAT_VERSION,
        "source": SOURCE_LABEL,
        "can_dos": entries,
    }
    return json.dumps(document, ensure_ascii=False, indent=1) + "\n"


def main() -> None:
    workbook = xlrd.open_workbook(str(EXCEL_PATH))
    sheet = workbook.sheet_by_index(0)
    # get_rows() yields Cell objects; the parser works on raw values.
    rows = [[cell.value for cell in row] for row in sheet.get_rows()]
    entries = parse_entries(rows[HEADER_ROWS:])
    assert_consistent(entries)

    payload = build_json(entries)
    for path in (CDN_OUTPUT, FIXTURE_OUTPUT):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(payload, encoding="utf-8")
        print(f"Wrote {path} ({len(entries)} entries)")


if __name__ == "__main__":
    main()
