"""Set up the HetrixTools uptime monitors for Origa (idempotent).

The monitoring plan lives in ``docs/runbooks/uptime-monitoring.md``. HetrixTools
carries the world-facing part: website monitors with keyword assertions (they
catch the silent Railway custom-domain fallback 404s), SSL/domain-expiry flags,
and heartbeat monitors that act as the dead-man switch for the RF poller.

The HetrixTools API is split across two protocol generations:

- **v2** (``https://api.hetrixtools.com/v2/<uptime_api_token>/uptime/...``) —
  the only write path. The token rides in the URL path, so it must never be
  logged or echoed. Field shapes follow the v2 documentation: ``Target`` for
  the URL, ``Frequency`` as a string of minutes (1/3/5/10), ``Locations`` as a
  mapping of location code to boolean, expiry reminders as day-count strings,
  and ``MID`` to address an existing monitor on edit.
- **v3** (``GET /v3/uptime-monitors``, ``Authorization: Bearer``) — read-only,
  used as the source of truth for idempotency: we diff what the account
  already has (flat ``target``/``keyword`` entries) against the desired state
  below.

Heartbeat monitors are intentionally NOT created here: the API cannot create
them (v2 knows website/ping/smtp/agent types only). They are created by hand
in the HetrixTools UI — see the runbook for the periods.

Usage:

- default: dry-run, prints the plan without touching the account;
- ``--apply``: performs the missing adds/edits.

Exit codes: 0 ok, 1 configuration missing, 2 API error.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from dataclasses import dataclass
from typing import Final

V3_BASE: Final[str] = "https://api.hetrixtools.com/v3"
V2_BASE: Final[str] = "https://api.hetrixtools.com/v2"

# EU locations closest to the product audience; the free tier allows 4 of 12.
LOCATIONS: Final[list[str]] = ["fra", "waw", "lon", "ams"]

# Fastest interval the v2 API accepts (minutes, string per the docs).
CHECK_FREQUENCY_MINUTES: Final[str] = "1"

# Expiry reminders are day-count strings in v2: warn 30 days ahead so a
# broken renewal has weeks of runway before browsers complain.
EXPIRY_HORIZON_DAYS: Final[str] = "30"

# v2 monitor type 1 = website.
WEBSITE_TYPE: Final[int] = 1


@dataclass(frozen=True)
class DesiredMonitor:
    name: str
    target: str
    keyword: str
    ssl_expiry_reminder: bool = False
    domain_expiry_reminder: bool = False


DESIRED_MONITORS: Final[list[DesiredMonitor]] = [
    DesiredMonitor(
        name="origa-landing",
        target="https://origa.uwuwu.net/",
        keyword="Learn Japanese",
        ssl_expiry_reminder=True,
        domain_expiry_reminder=True,
    ),
    DesiredMonitor(
        name="app-healthcheck",
        target="https://app.origa.uwuwu.net/api/healthcheck",
        keyword="Ok",
        ssl_expiry_reminder=True,
    ),
    DesiredMonitor(
        name="s3-manifest",
        target="https://s3.origa.uwuwu.net/manifest.json",
        keyword="generated_at",
        ssl_expiry_reminder=True,
    ),
    DesiredMonitor(
        name="content-tigris",
        target="https://content.origa.uwuwu.net/",
        keyword="",
        ssl_expiry_reminder=True,
    ),
]


@dataclass
class PlanAction:
    action: str  # "add" | "edit" | "skip"
    monitor: DesiredMonitor
    monitor_id: str = ""
    reason: str = ""


def require_env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        print(f"missing required environment variable: {name}", file=sys.stderr)
        raise SystemExit(1)
    return value


def fetch_existing_monitors(api_key: str) -> list[dict]:
    request = urllib.request.Request(
        f"{V3_BASE}/uptime-monitors",
        headers={"Authorization": f"Bearer {api_key}"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            body = json.load(response)
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as error:
        print(f"v3 read failed: {error}", file=sys.stderr)
        raise SystemExit(2) from error
    return body.get("monitors", [])


def build_v2_payload(monitor: DesiredMonitor, monitor_id: str | None = None) -> dict:
    payload: dict = {
        "Name": monitor.name,
        "Target": monitor.target,
        "Type": WEBSITE_TYPE,
        "Frequency": CHECK_FREQUENCY_MINUTES,
        "Locations": {location: True for location in LOCATIONS},
        "FailsBeforeAlert": 2,
    }
    if monitor.keyword:
        payload["Keyword"] = monitor.keyword
    if monitor.ssl_expiry_reminder:
        payload["SSLExpiryReminder"] = EXPIRY_HORIZON_DAYS
    if monitor.domain_expiry_reminder:
        payload["DomainExpiryReminder"] = EXPIRY_HORIZON_DAYS
    if monitor_id:
        payload["MID"] = monitor_id
    return payload


def describe_mismatch(current: dict, monitor: DesiredMonitor) -> str:
    problems: list[str] = []
    if current.get("target") != monitor.target:
        problems.append("target")
    if monitor.keyword and current.get("keyword") != monitor.keyword:
        problems.append("keyword")
    # The v3 listing exposes only a subset of the v2 fields; the SSL flag is
    # one of them. When it is present we enforce it too — so flipping the
    # flag by hand in the UI does not leave the monitor "in sync". Absent
    # keys stay outside the idempotency contract (the runbook documents it).
    if (
        "ssl_expiration_warn" in current
        and bool(current["ssl_expiration_warn"]) != monitor.ssl_expiry_reminder
    ):
        problems.append("ssl-flag")
    return ", ".join(problems)


def diff_plan(existing: list[dict], desired: list[DesiredMonitor]) -> list[PlanAction]:
    by_name: dict[str, dict] = {item.get("name", ""): item for item in existing}
    actions: list[PlanAction] = []
    for monitor in desired:
        current = by_name.get(monitor.name)
        if current is None:
            actions.append(PlanAction("add", monitor))
            continue
        mismatch = describe_mismatch(current, monitor)
        if mismatch:
            actions.append(
                PlanAction("edit", monitor, str(current.get("id", "")), mismatch)
            )
        else:
            actions.append(
                PlanAction("skip", monitor, str(current.get("id", "")), "in sync")
            )
    return actions


def v2_call(api_token: str, endpoint: str, payload: dict) -> dict:
    # The v2 token rides in the URL path — never log or echo the full URL.
    url = f"{V2_BASE}/{api_token}/uptime/{endpoint}/"
    request = urllib.request.Request(
        url,
        data=json.dumps(payload).encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as error:
        print(f"v2 write failed ({endpoint}): {error}", file=sys.stderr)
        raise SystemExit(2) from error


def execute_plan(actions: list[PlanAction], api_token: str) -> int:
    failures = 0
    for action in actions:
        if action.action == "skip":
            print(f"skip  {action.monitor.name}: {action.reason}")
            continue
        payload = build_v2_payload(action.monitor, action.monitor_id or None)
        result = v2_call(api_token, action.action, payload)
        if result.get("status") == "success" or result.get("ok"):
            print(f"{action.action:4}  {action.monitor.name}: ok")
        else:
            failures += 1
            print(
                f"{action.action:4}  {action.monitor.name}: API refused: {result}",
                file=sys.stderr,
            )
    return failures


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--apply",
        action="store_true",
        help="write changes to the account (default: dry-run)",
    )
    arguments = parser.parse_args()

    api_key = require_env("HETRIX_V3_API_KEY")
    api_token = require_env("HETRIX_V2_API_TOKEN") if arguments.apply else ""

    existing = fetch_existing_monitors(api_key)
    actions = diff_plan(existing, DESIRED_MONITORS)

    if not arguments.apply:
        print("dry-run plan (re-run with --apply to write):")
        for action in actions:
            suffix = f" [{action.reason}]" if action.reason else ""
            print(f"{action.action:4}  {action.monitor.name}{suffix}")
        return

    failures = execute_plan(actions, api_token)
    raise SystemExit(1 if failures else 0)


if __name__ == "__main__":
    main()
