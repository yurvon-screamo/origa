"""Guards for the remainder translation-audit witness (uwuwu#1742).

The witness is the only reviewable artifact of this audit: the corrected data
is held as a patch-set outside the canonical corpus until the owner authorizes
a CDN deploy (ADR-083, NFT-2 in #1463). These pure tests read the repository
only; a pin on mutable production state would rot on the next deploy.
"""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
WITNESS_PATH = (
    REPO_ROOT / "docs" / "worklogs" / "2026-11-28-translation-audit-remainder.json"
)
LOCALES = {"ja", "en", "vi", "ko", "ru"}


@pytest.fixture(scope="module")
def witness() -> dict:
    return json.loads(WITNESS_PATH.read_text(encoding="utf-8"))


def test_witness_counts_cover_the_declared_population(witness: dict) -> None:
    changes = witness["changes"]
    audit = witness["audit"]

    assert witness["change_count"] == len(changes) == 596
    assert sum(audit["outcomes"].values()) == audit["population"] == 9468
    assert audit["covered_words"] == 7701
    assert (
        audit["unverified_pool"]
        == audit["covered_words"] + audit["unverified_remaining"]
    )
    assert dict(Counter(c["class"] for c in changes)) == {"A": 308, "B": 288}


def test_every_recorded_change_really_changes_a_value(witness: dict) -> None:
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
    for change in witness["changes"]:
        assert change["reason"].strip(), change


def test_gloss_changes_only_append_senses(witness: dict) -> None:
    glosses = [c for c in witness["changes"] if c["class"] == "A"]
    assert glosses
    for change in glosses:
        assert change["field"] == "ru.t", change
        assert change["path"].startswith("dictionary/"), change
        assert change["after"][: len(change["before"])] == change["before"], change
        assert len(change["after"]) > len(change["before"]), change


def test_example_changes_address_the_right_shard_and_field(witness: dict) -> None:
    examples = [c for c in witness["changes"] if c["class"] == "B"]
    assert examples
    for change in examples:
        assert change["field"] == "ru", change
        assert change["word"] != "" and change["sid"], change
        assert all(
            change["path"] == f"examples/data/s{sid // 1000:04d}.json"
            for sid in change["sid"]
        ), change


def test_every_change_carries_structured_evidence(witness: dict) -> None:
    for change in witness["changes"]:
        evidence = change["evidence"]
        assert evidence["locale"] in LOCALES, change
        assert evidence["quote"].strip(), change


def test_patch_set_is_held_outside_the_corpus_until_authorized(witness: dict) -> None:
    assert "deploy requires owner command" in witness["deploy_status"]
    assert witness["corpus_snapshot"]["file_count"] == len(
        witness["corpus_snapshot"]["files"]
    )
