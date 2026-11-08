"""Unit tests for the HetrixTools monitor setup (``setup_uptime_monitors``).

The setup script keeps the HetrixTools account in sync with the desired
monitor table of the runbook. The API is split: v2 writes (token in the URL
path), v3 reads (Bearer). The tests pin the parts that can silently go wrong:
the desired keyword contract (every keyword must be absent from the Railway
fallback 404 body, otherwise a silently slapped custom-domain binding would
look green), the diff/idempotency logic over the v3 listing, the v2 payload
shape (anti-flap count, locations, expiry flags), and the guarantee that a
monitor whose v3 entry is already in sync is never rewritten.

The tests are pure: no network, no environment, no filesystem.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import setup_uptime_monitors as setup

RAILWAY_FALLBACK_BODY = (
    '{"status":"error","code":404,"message":"Application not found",'
    '"request_id":"ic_xyz"}'
)


def test_keywords_absent_from_railway_fallback() -> None:
    """A silent custom-domain slap must flip monitors DOWN, not stay green."""
    for monitor in setup.DESIRED_MONITORS:
        if not monitor.keyword:
            continue
        assert monitor.keyword not in RAILWAY_FALLBACK_BODY, monitor.name


def test_desired_set_matches_the_runbook() -> None:
    names = [monitor.name for monitor in setup.DESIRED_MONITORS]
    assert names == [
        "origa-landing",
        "app-healthcheck",
        "s3-manifest",
        "content-tigris",
    ]
    landing = setup.DESIRED_MONITORS[0]
    assert landing.domain_expiry_reminder is True, "uwuwu.net expiry must be watched"


def test_v2_payload_carries_antiflap_and_locations() -> None:
    payload = setup.build_v2_payload(setup.DESIRED_MONITORS[0])
    assert payload["FailsBeforeAlert"] == 2
    assert payload["Locations"] == setup.LOCATIONS
    assert payload["Keyword"] == "Learn Japanese"
    assert payload["SSLExpiryReminder"] is True
    assert payload["DomainExpiryReminder"] is True
    assert payload["Type"] == setup.WEBSITE_TYPE


def test_v2_payload_omits_empty_keyword() -> None:
    content = next(m for m in setup.DESIRED_MONITORS if m.name == "content-tigris")
    payload = setup.build_v2_payload(content)
    assert "Keyword" not in payload


def _existing_entry(name: str, monitor_id: str, target: str, keyword: str) -> dict:
    return {
        "name": name,
        "id": monitor_id,
        "v2": {"target": target, "keyword": keyword},
    }


def test_diff_plans_add_for_unknown_monitor() -> None:
    actions = setup.diff_plan([], setup.DESIRED_MONITORS[:1])
    assert [(a.action, a.reason) for a in actions] == [("add", "")]


def test_diff_skips_in_sync_monitor() -> None:
    monitor = setup.DESIRED_MONITORS[1]
    existing = [_existing_entry(monitor.name, "42", monitor.target, monitor.keyword)]
    actions = setup.diff_plan(existing, [monitor])
    assert [(a.action, a.monitor_id) for a in actions] == [("skip", "42")]


def test_diff_flags_target_drift_for_edit() -> None:
    monitor = setup.DESIRED_MONITORS[0]
    existing = [_existing_entry(monitor.name, "7", "https://old.example/", "")]
    actions = setup.diff_plan(existing, [monitor])
    assert [a.action for a in actions] == ["edit"]
    assert actions[0].monitor_id == "7"
    assert "target" in actions[0].reason


def test_diff_flags_keyword_drift_for_edit() -> None:
    monitor = setup.DESIRED_MONITORS[2]
    existing = [_existing_entry(monitor.name, "9", monitor.target, "stale")]
    actions = setup.diff_plan(existing, [monitor])
    assert [a.action for a in actions] == ["edit"]
    assert "keyword" in actions[0].reason


@pytest.mark.parametrize("variable", ["HETRIX_V3_API_KEY", "HETRIX_V2_API_TOKEN"])
def test_missing_env_stops_before_any_api_call(
    monkeypatch: pytest.MonkeyPatch, variable: str
) -> None:
    monkeypatch.delenv(variable, raising=False)
    with pytest.raises(SystemExit) as raised:
        setup.require_env(variable)
    assert raised.value.code == 1


def test_monitor_count_fits_the_free_tier() -> None:
    # Free tier: 15 uptime monitors. 4 website (API) + 2 heartbeat (UI) = 6.
    api_monitors = len(setup.DESIRED_MONITORS)
    assert api_monitors + 2 <= 15
