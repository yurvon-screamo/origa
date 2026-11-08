"""Acceptance E2E for the monitor setup against the live HetrixTools account.

The only safe live path is the default dry-run: a v3 GET listing plus the
plan print — zero writes. This is the top-level acceptance check that the
vendor contract (two-protocol API, account credentials, desired set) holds
against the real service, which the pure/monkeypatched tests cannot prove.

Gated behind ``HETRIX_LIVE_E2E=1`` + ``HETRIX_V3_API_KEY`` (an Account API
Key with read access): CI has no credentials, and the account belongs to the
owner. ``--apply`` is intentionally never exercised here — writes to the live
account are an owner-run step (runbook slice 3).
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

import pytest

REPO_SCRIPTS = Path(__file__).resolve().parent.parent
SCRIPT = REPO_SCRIPTS / "setup_uptime_monitors.py"

REQUIRED_ENV = ("HETRIX_LIVE_E2E", "HETRIX_V3_API_KEY")

pytestmark = pytest.mark.skipif(
    os.environ.get("HETRIX_LIVE_E2E") != "1" or not os.environ.get("HETRIX_V3_API_KEY"),
    reason="live HetrixTools account: needs HETRIX_LIVE_E2E=1 and HETRIX_V3_API_KEY",
)


def test_dry_run_plans_the_desired_set_against_the_live_account() -> None:
    proc = subprocess.run(
        [sys.executable, str(SCRIPT)],
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    assert "dry-run plan" in proc.stdout
    for monitor in (
        "origa-landing",
        "app-healthcheck",
        "s3-manifest",
        "content-tigris",
    ):
        assert monitor in proc.stdout, monitor
    # Dry-run never needs (or uses) the write token.
    assert "HETRIX_V2_API_TOKEN" not in proc.stdout + proc.stderr
