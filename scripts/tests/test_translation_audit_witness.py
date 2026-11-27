"""Guards for the translation-audit witness (uwuwu#1737, #935).

The witness JSON is the only reviewable artifact of the audit: the fixed data
itself lives in the CDN corpus outside the repository (ADR-083). These tests
keep that record honest — a dropped change, a hollowed-out count, or a
malformed before/after pair must fail the build instead of silently turning
the audit into an unverifiable claim.

Pure tests: they read the repository only. The corpus is deliberately not
loaded here — a pin on mutable production state would rot on the next deploy
(the К2/К3 bad-test class); live verification is a manual run, not a test.
The round-2 pins guard invariants of this record, not its literal wording;
a generator overwrite of the CDN corpus (#1743) stays invisible to them.
"""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
WITNESS_PATH = REPO_ROOT / "docs" / "worklogs" / "2026-11-27-translation-audit.json"


@pytest.fixture(scope="module")
def witness() -> dict:
    return json.loads(WITNESS_PATH.read_text(encoding="utf-8"))


def test_witness_counts_cover_the_declared_population(witness: dict) -> None:
    # Arrange
    changes = witness["changes"]
    audit = witness["audit"]

    # Act
    classes = Counter(change["class"] for change in changes)

    # Assert
    assert witness["change_count"] == len(changes) == 58
    assert sum(audit["outcomes"].values()) == audit["population"] == 9468
    assert dict(classes) == {"A": 6, "B": 17, "пропущенное поле": 35}
    assert audit["missing_fields_total"] == classes["пропущенное поле"]


def test_every_recorded_field_really_changes(witness: dict) -> None:
    for change in witness["changes"]:
        assert change["before"] != change["after"], change
        after = change["after"]
        assert after, change
        if isinstance(after, list):
            assert all(isinstance(sense, str) and sense for sense in after), change
        else:
            assert isinstance(after, str) and after.strip(), change


def test_change_keys_are_unique(witness: dict) -> None:
    keys = [
        (c["path"], c["field"], tuple(c["sid"]), c["word"]) for c in witness["changes"]
    ]
    assert len(keys) == len(set(keys))


def test_every_change_has_a_reason(witness: dict) -> None:
    # The record is read and rolled back by path and reason; glosses need the
    # justification as much as examples do.
    for change in witness["changes"]:
        assert change["reason"].strip(), change


def test_dictionary_changes_only_append_ru_senses(witness: dict) -> None:
    glosses = [c for c in witness["changes"] if c["path"].startswith("dictionary/")]

    assert len(glosses) == 6
    for change in glosses:
        before, after = change["before"], change["after"]
        assert change["field"] == "ru.t" and change["word"], change
        assert change["class"] == "A", change
        assert len(after) > len(before) and after[: len(before)] == before, change
        assert len(set(after)) == len(after), change
        assert change["sid"] and all(sid > 0 for sid in change["sid"]), change


def test_example_changes_touch_only_translations(witness: dict) -> None:
    examples = [c for c in witness["changes"] if c["path"].startswith("examples/data/")]

    assert len(examples) == 52
    for change in examples:
        assert change["field"] in {"ru", "vi", "ko"}, change
        assert change["word"] == "" and change["sid"], change
        assert all(
            change["path"] == f"examples/data/s{sid // 1000:04d}.json"
            for sid in change["sid"]
        ), change
        if change["class"] == "пропущенное поле":
            assert change["before"] is None, change
        else:
            assert change["class"] == "B" and change["before"], change


def test_witness_takes_gloss_record_drops_initiative(witness: dict) -> None:
    # Pin the record, not the product: round 1 wrote a sense absent from the
    # JA/EN/VI/KO sources. A different correct verb wording stays valid.
    takes = next(c for c in witness["changes"] if c["word"] == "取れる")

    assert "инициатив" not in " ".join(takes["after"])


def test_witness_example_19194_record_keeps_antecedent(witness: dict) -> None:
    # Pin the record, not the product: round 1 lost the quoted reply, so the
    # recorded RU translation must keep it. Wording around it may change.
    change = next(c for c in witness["changes"] if c["sid"] == [19194])

    assert "немного беспокоит" in change["after"]


def test_witness_records_chunk_09_layout_correction(witness: dict) -> None:
    # Pin the record, not the product: round 1 rewrote the compact chunk with
    # indent=1 and the witness did not describe the byte-level delta.
    layout = witness["round2_adjustments"]["dictionary_chunk_09_layout"]

    assert (
        layout["before_bytes"] <= layout["final_bytes"] < layout["intermediate_bytes"]
    )
    assert layout["result"].strip()
    assert witness["round2_adjustments"]["canonical_example_key_order"]["order"] == [
        "i",
        "x",
        "en",
        "f",
        "ru",
        "vi",
        "ko",
    ]
