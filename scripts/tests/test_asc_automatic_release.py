"""Unit tests for the ASC release-type automation (issue #730).

The script guarantees the appStoreVersion's releaseType is AUTOMATIC so
Apple publishes right after review approval and the manual "Release" button
never appears (it was forgotten for the 0.7.37 drafts).

Tests are pure: they pin the appStoreState → Action classification table
(unknown Apple states must fail closed, never silently PATCH) and the
request payload shapes. No network, no keys.
"""

from __future__ import annotations

import pytest

from asc_set_automatic_release import (
    NOOP_STATES,
    PATCH_STATES,
    Action,
    build_create_payload,
    build_patch_payload,
    create_outcome,
    decide_action,
)

# All documented appStoreState values (ASC OpenAPI enum). Every value must be
# consciously classified — a new Apple state reaching decide_action must land
# in UNKNOWN (fail closed), which this table makes visible in review.
ALL_APP_STORE_STATES: frozenset[str] = frozenset(
    {
        "DEVELOPER_REJECTED",
        "DEVELOPER_REMOVED_FROM_SALE",
        "IN_REVIEW",
        "INVALID_BINARY",
        "METADATA_REJECTED",
        "PENDING_APPLE_RELEASE",
        "PENDING_CONTRACT",
        "PENDING_DEVELOPER_RELEASE",
        "PREORDER_READY_FOR_SALE",
        "PREPARE_FOR_SUBMISSION",
        "PROCESSING_FOR_APP_STORE",
        "READY_FOR_DISTRIBUTION",
        "READY_FOR_SALE",
        "REJECTED",
        "REMOVED_FROM_SALE",
        "REPLACED_WITH_NEW_VERSION",
        "WAITING_FOR_EXPORT_COMPLIANCE",
        "WAITING_FOR_REVIEW",
    }
)

# Deliberately unclassified (account-level conditions — contract, export
# compliance): an automatic PATCH decision there would be guesswork. If a
# release ever hits one, classify it consciously first.
FAIL_CLOSED_STATES: frozenset[str] = frozenset(
    {
        "PENDING_CONTRACT",
        "WAITING_FOR_EXPORT_COMPLIANCE",
    }
)


# ---------------------------------------------------------------------------
# Classification table — no gaps, no silent drift
# ---------------------------------------------------------------------------


def test_known_states_partition_is_exhaustive():
    # Every documented state except the fail-closed ones is classified, and
    # the two classification sets don't overlap. A future enum addition by
    # Apple lands in UNKNOWN (exit 1) until classified here.
    assert PATCH_STATES | NOOP_STATES | FAIL_CLOSED_STATES == (
        ALL_APP_STORE_STATES
    )
    assert not PATCH_STATES & NOOP_STATES


@pytest.mark.parametrize("state", sorted(PATCH_STATES))
def test_patch_states(state: str):
    assert decide_action(state) == Action.PATCH


@pytest.mark.parametrize("state", sorted(NOOP_STATES))
def test_noop_states(state: str):
    assert decide_action(state) == Action.NOOP


@pytest.mark.parametrize("state", sorted(FAIL_CLOSED_STATES))
def test_fail_closed_states(state: str):
    assert decide_action(state) == Action.UNKNOWN


def test_unknown_state_fails_closed():
    # A state Apple may add in the future must never silently PATCH.
    assert decide_action("SOME_FUTURE_APPLE_STATE") == Action.UNKNOWN


def test_missing_version_creates():
    assert decide_action(None) == Action.CREATE


# ---------------------------------------------------------------------------
# POST outcome classification (create / conflict / fail loud)
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("code", [200, 201])
def test_create_outcome_created(code: int):
    assert create_outcome(code) == "created"


def test_create_outcome_conflict():
    # 409 = version appeared between GET and POST (async auto-creation), or
    # an older unsabmitted version occupies the slot → re-fetch and patch.
    assert create_outcome(409) == "conflict"


@pytest.mark.parametrize("code", [400, 401, 403, 404, 500, 503])
def test_create_outcome_error(code: int):
    assert create_outcome(code) == "error"


# ---------------------------------------------------------------------------
# Payload shapes — the AUTOMATIC contract
# ---------------------------------------------------------------------------


def test_create_payload_carries_automatic():
    payload = build_create_payload("0.7.38", "IOS", "123456789")
    data = payload["data"]
    assert data["type"] == "appStoreVersions"
    assert data["attributes"] == {
        "versionString": "0.7.38",
        "platform": "IOS",
        "releaseType": "AUTOMATIC",
    }
    assert data["relationships"]["app"]["data"] == {
        "type": "apps",
        "id": "123456789",
    }


def test_patch_payload_carries_automatic_and_id():
    payload = build_patch_payload("987654321")
    assert payload["data"] == {
        "type": "appStoreVersions",
        "id": "987654321",
        "attributes": {"releaseType": "AUTOMATIC"},
    }
