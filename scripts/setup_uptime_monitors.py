"""Set up the HetrixTools uptime monitors for Origa (idempotent).

The monitoring plan lives in ``docs/runbooks/uptime-monitoring.md``. HetrixTools
carries the world-facing part: website monitors with keyword assertions (they
catch the silent Railway custom-domain fallback 404s), SSL/domain-expiry flags,
and heartbeat monitors that act as the dead-man switch for the RF poller.

The HetrixTools API is split across two protocol generations:

- **v2** (``/v2/<uptime_api_token>/uptime/add|edit/``) — the only write path.
  The token rides in the URL path, so it must never be logged or echoed.
- **v3** (``GET /v3/uptime-monitors``, ``Authorization: Bearer``) — read-only,
  used as the source of truth for idempotency: we diff what the account
  already has against the desired state below.

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
V2_BASE: Final[str] = "https://hetrixtools.com/api"

# EU locations closest to the product audience; the free tier allows 4 of 12.
LOCATIONS: Final[list[str]] = ["fra", "waw", "lon", "ams"]


# Keyword assertions pick bodies that the Railway fallback 404
# ("Application not found") can never contain, so a silently slapped
# custom-domain binding flips the monitor to DOWN instead of lying green.
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

# v2 monitor type 1 = website.
WEBSITE_TYPE: Final[int] = 1


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


def build_v2_payload(monitor: DesiredMonitor) -> dict:
    payload: dict = {
        "Name": monitor.name,
        "URL": monitor.target,
        "Type": WEBSITE_TYPE,
        "CheckFrequency": 60,
        "Locations": LOCATIONS,
        "FailsBeforeAlert": 2,
        "SendEmail": True,
        "SendSMS": False,
    }
    if monitor.keyword:
        payload["Keyword"] = monitor.keyword
    if monitor.ssl_expiry_reminder:
        payload["SSLExpiryReminder"] = True
    if monitor.domain_expiry_reminder:
        payload["DomainExpiryReminder"] = True
    return payload


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


def describe_mismatch(current: dict, monitor: DesiredMonitor) -> str:
    problems: list[str] = []
    v2 = current.get("v2", {})
    uptime = current.get("uptime", {})
    merged = {**uptime, **v2}
    if merged.get("url") != monitor.target and merged.get("target") != monitor.target:
        problems.append("target")
    keyword = monitor.keyword
    if keyword and keyword not in (merged.get("keyword"), merged.get("Keyword")):
        problems.append("keyword")
    return ", ".join(problems)


def v2_call(api_token: str, endpoint: str, payload: dict) -> dict:
    # The v2 token rides in the URL path — never log or echo the full URL.
    url = f"{V2_BASE}/v2/{api_token}/uptime/{endpoint}/"
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
        payload = build_v2_payload(action.monitor)
        result = v2_call(api_token, action.action, payload)
        status = result.get("status")
        if status == "success" or result.get("ok"):
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
