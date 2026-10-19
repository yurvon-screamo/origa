#!/usr/bin/env python3
"""Set the App Store Connect release type to AUTOMATIC for a version (issue #730).

With releaseType == MANUAL (the ASC default) Apple shows the manual "Release"
button after review approval. It was forgotten for the 0.7.37 drafts. This
script runs in CI right after the altool upload and guarantees the version's
releaseType is AUTOMATIC: Apple publishes the app itself right after approval
and the button never appears.

Flow:
  1. ES256 JWT — reuses the battle-tested helper from download_macos_profile
     (stdlib + openssl; aud: appstoreconnect-v1 is the undocumented claim
     Apple requires, without it the API answers 401 NOT_AUTHORIZED).
  2. GET /v1/apps?filter[bundleId]=<bundle_id> → app id (0 or >1 → fail loud).
  3. GET /v1/apps/{app}/appStoreVersions?filter[versionString]=V&filter[platform]=P
     — up to 3 attempts with backoff, because a version can become visible
     asynchronously after the upload.
  4. decide_action on appStoreState:
       - PATCH state   → PATCH /v1/appStoreVersions/{id} {releaseType: AUTOMATIC}
       - NOOP state    → log + exit 0 (idempotent re-runs)
       - no version    → POST /v1/appStoreVersions (create with AUTOMATIC)
       - unknown state → fail loud: a new Apple state is a conscious decision,
                         never a silent PATCH.
  5. POST returns 409 (version appeared between GET and POST, or an older
     unsabmitted version occupies the slot) → re-GET: found → PATCH;
     still empty → fail loud with the 409 body.

Zero external dependencies — Python stdlib + openssl (via the JWT helper).

Usage:
    APPLE_API_KEY_PATH=path/to/AuthKey.p8 \\
    APPLE_API_KEY=KEYID123ABC \\
    APPLE_API_ISSUER=12345678-abcd-... \\
    python3 asc_set_automatic_release.py \\
        --platform IOS --version-string 0.7.38 [--bundle-id net.uwuwu.origa]

Exit codes:
    0 — release type is AUTOMATIC (patched, created, or already released)
    1 — API error, ambiguous app lookup, or unknown appStoreState (fail loud)
"""

from __future__ import annotations

import argparse
import enum
import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

# Sibling CI script in the same directory: sys.path[0] is this script's
# directory, so the plain import mirrors how the workflow invokes it.
# An explicit import (vs a copy) fails loudly at startup if the helper
# moves, instead of silently drifting.
from download_macos_profile import create_es256_jwt

ASC_API_BASE = "https://api.appstoreconnect.apple.com/v1"

# How many times to re-fetch the version list before concluding the version
# does not exist (and creating it). ASC processes uploads asynchronously, so
# an auto-created version may lag a bit behind the altool exit.
VERSION_FETCH_ATTEMPTS = 3
VERSION_FETCH_BACKOFF_SECONDS = 20


class Action(enum.Enum):
    """What to do with the (possibly absent) appStoreVersion."""

    PATCH = "patch"
    NOOP = "noop"
    CREATE = "create"
    UNKNOWN = "unknown"


# States where the version is editable (or waiting for the manual release
# button) — setting releaseType=AUTOMATIC is meaningful. PENDING_DEVELOPER_
# RELEASE is the "approved, waiting for the manual button" state: the PATCH
# is best-effort there (an immediate auto-release from PDR via the REST API
# is NOT verified; if the API rejects or ignores it, the job reddens and the
# release still has the manual fallback — the exact status quo this issue
# removes).
PATCH_STATES = frozenset(
    {
        "PREPARE_FOR_SUBMISSION",
        "WAITING_FOR_REVIEW",
        "IN_REVIEW",
        "PENDING_DEVELOPER_RELEASE",
        "PENDING_APPLE_RELEASE",
        "REJECTED",
        "METADATA_REJECTED",
        "DEVELOPER_REJECTED",
        "INVALID_BINARY",
    }
)

# Terminal states: the version is already live / replaced / withdrawn —
# patching a release type there is meaningless. A re-run of the same tag
# exits 0 (idempotent).
NOOP_STATES = frozenset(
    {
        "READY_FOR_SALE",
        "PROCESSING_FOR_APP_STORE",
        "PREORDER_READY_FOR_SALE",
        "READY_FOR_DISTRIBUTION",
        "REPLACED_WITH_NEW_VERSION",
        "REMOVED_FROM_SALE",
        "DEVELOPER_REMOVED_FROM_SALE",
    }
)

# Known-but-unclassified states, deliberately fail-closed: they depend on
# account-level conditions (contract, export compliance) where an automatic
# PATCH decision would be guesswork.
#   WAITING_FOR_EXPORT_COMPLIANCE, PENDING_CONTRACT


def decide_action(state: str | None) -> Action:
    """Classify an appStoreState (None = version not found)."""
    if state is None:
        return Action.CREATE
    if state in PATCH_STATES:
        return Action.PATCH
    if state in NOOP_STATES:
        return Action.NOOP
    return Action.UNKNOWN


def create_outcome(http_code: int) -> str:
    """Classify the POST /appStoreVersions response code.

    "created"  — draft created with AUTOMATIC, done.
    "conflict" — a version for this (versionString, platform) already exists
                 (it appeared between our GET and POST, or an older
                 unsabmitted version occupies the slot) → re-fetch and patch.
    "error"    — anything else → fail loud.
    """
    if http_code < 300:
        return "created"
    if http_code == 409:
        return "conflict"
    return "error"


def build_create_payload(
    version_string: str, platform: str, app_id: str
) -> dict:
    return {
        "data": {
            "type": "appStoreVersions",
            "attributes": {
                "versionString": version_string,
                "platform": platform,
                "releaseType": "AUTOMATIC",
            },
            "relationships": {
                "app": {"data": {"type": "apps", "id": app_id}}
            },
        }
    }


def build_patch_payload(version_id: str) -> dict:
    return {
        "data": {
            "type": "appStoreVersions",
            "id": version_id,
            "attributes": {"releaseType": "AUTOMATIC"},
        }
    }


# ---------------------------------------------------------------------------
# App Store Connect API
# ---------------------------------------------------------------------------


def asc_request(
    url: str,
    jwt_token: str,
    method: str = "GET",
    payload: dict | None = None,
) -> tuple[int, dict | None]:
    """Perform one ASC API call, return (http_code, parsed_body_or_None).

    HTTP >= 400 responses are RETURNED (not raised) so callers can log the
    full body before failing — the same diagnostics convention as the
    upload-rustore workflow and download_macos_profile.py.
    """
    data = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Authorization", f"Bearer {jwt_token}")
    req.add_header("Accept", "application/json")
    if data is not None:
        req.add_header("Content-Type", "application/json")

    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return resp.status, json.loads(resp.read())
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        try:
            return exc.code, json.loads(body)
        except json.JSONDecodeError:
            return exc.code, {"raw": body}


def resolve_app_id(jwt_token: str, bundle_id: str) -> str:
    """Resolve the ASC app id from the bundle identifier. Fail loud otherwise."""
    params = urllib.parse.urlencode({"filter[bundleId]": bundle_id})
    code, body = asc_request(f"{ASC_API_BASE}/apps?{params}", jwt_token)
    if code != 200 or body is None:
        print(
            f"ERROR: GET /apps returned HTTP {code}: "
            f"{json.dumps(body, ensure_ascii=False)}",
            file=sys.stderr,
        )
        raise SystemExit(1)

    apps = body.get("data", [])
    if len(apps) != 1:
        print(
            "ERROR: expected exactly one app for bundleId "
            f"{bundle_id!r}, got {len(apps)}. "
            f"Response: {json.dumps(body, ensure_ascii=False)}",
            file=sys.stderr,
        )
        raise SystemExit(1)

    return apps[0]["id"]


def fetch_versions(
    jwt_token: str, app_id: str, version_string: str, platform: str
) -> list[dict]:
    """Fetch appStoreVersions for (versionString, platform), with retries.

    Retries only cover the "no versions yet" case — ASC may make an
    auto-created version visible asynchronously after the upload. API errors
    fail immediately.
    """
    params = urllib.parse.urlencode(
        {
            "filter[versionString]": version_string,
            "filter[platform]": platform,
        }
    )
    url = f"{ASC_API_BASE}/apps/{app_id}/appStoreVersions?{params}"

    versions: list[dict] = []
    for attempt in range(1, VERSION_FETCH_ATTEMPTS + 1):
        code, body = asc_request(url, jwt_token)
        if code != 200 or body is None:
            print(
                f"ERROR: GET appStoreVersions returned HTTP {code}: "
                f"{json.dumps(body, ensure_ascii=False)}",
                file=sys.stderr,
            )
            raise SystemExit(1)
        versions = body.get("data", [])
        if versions:
            return versions
        if attempt < VERSION_FETCH_ATTEMPTS:
            print(
                f"No appStoreVersion for {version_string}/{platform} yet "
                f"(attempt {attempt}/{VERSION_FETCH_ATTEMPTS}), "
                f"retrying in {VERSION_FETCH_BACKOFF_SECONDS}s…"
            )
            time.sleep(VERSION_FETCH_BACKOFF_SECONDS)
    return versions


def patch_release_type(
    jwt_token: str, version_id: str, version_string: str, platform: str
) -> None:
    """PATCH releaseType=AUTOMATIC on an existing version. Fail loud on error."""
    code, body = asc_request(
        f"{ASC_API_BASE}/appStoreVersions/{version_id}",
        jwt_token,
        method="PATCH",
        payload=build_patch_payload(version_id),
    )
    if code < 300:
        print(
            f"✅ {platform}: version {version_string} "
            f"(id {version_id}) set to releaseType=AUTOMATIC"
        )
        return

    print(
        f"ERROR: PATCH appStoreVersions/{version_id} returned HTTP {code}: "
        f"{json.dumps(body, ensure_ascii=False)}",
        file=sys.stderr,
    )
    if code in (401, 403):
        print(
            "HINT: the ASC API key needs the App Manager (or Admin) role "
            "to patch appStoreVersions. Check the key's role in "
            "App Store Connect → Users and Access → Integration.",
            file=sys.stderr,
        )
    raise SystemExit(1)


def create_version(
    jwt_token: str,
    app_id: str,
    version_string: str,
    platform: str,
) -> None:
    """Create the version with AUTOMATIC; on 409 re-fetch and patch instead.

    409 semantics: the version appeared between our GET and POST (async
    auto-creation), or an older unsabmitted version occupies the editable
    slot. Re-fetch resolves both; if the re-fetch still finds nothing, the
    409 body is the only diagnostic we have — fail loud with it.
    """
    code, body = asc_request(
        f"{ASC_API_BASE}/appStoreVersions",
        jwt_token,
        method="POST",
        payload=build_create_payload(version_string, platform, app_id),
    )
    outcome = create_outcome(code)
    if outcome == "created":
        print(
            f"✅ {platform}: appStoreVersion {version_string} created by CI "
            "with releaseType=AUTOMATIC"
        )
        return
    if outcome != "conflict":
        print(
            f"ERROR: POST appStoreVersions returned HTTP {code}: "
            f"{json.dumps(body, ensure_ascii=False)}",
            file=sys.stderr,
        )
        if code in (401, 403):
            print(
                "HINT: the ASC API key needs the App Manager (or Admin) "
                "role to create appStoreVersions.",
                file=sys.stderr,
            )
        raise SystemExit(1)

    # Conflict path: re-fetch; found → patch, still empty → fail loud.
    print(
        f"POST returned 409 (version already exists or slot occupied), "
        f"re-fetching {version_string}/{platform}…"
    )
    versions = fetch_versions(jwt_token, app_id, version_string, platform)
    if not versions:
        print(
            "ERROR: POST appStoreVersions returned 409 but the re-fetch "
            f"still found no version for {version_string}/{platform}. "
            f"409 body: {json.dumps(body, ensure_ascii=False)}. "
            "An older unsabmitted version likely occupies the editable "
            "slot — resolve it in App Store Connect.",
            file=sys.stderr,
        )
        raise SystemExit(1)
    patch_release_type(
        jwt_token, versions[0]["id"], version_string, platform
    )


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Set the App Store Connect release type to AUTOMATIC "
            "(issue #730)"
        )
    )
    parser.add_argument(
        "--platform",
        required=True,
        choices=["IOS", "MAC_OS"],
        help="ASC platform of the appStoreVersion",
    )
    parser.add_argument(
        "--version-string",
        required=True,
        help="CFBundleShortVersionString of the release (e.g. 0.7.38)",
    )
    parser.add_argument(
        "--bundle-id",
        default="net.uwuwu.origa",
        help="Bundle identifier (default: net.uwuwu.origa)",
    )
    args = parser.parse_args()

    key_path = os.environ.get("APPLE_API_KEY_PATH", "")
    key_id = os.environ.get("APPLE_API_KEY", "")
    issuer_id = os.environ.get("APPLE_API_ISSUER", "")
    if not all([key_path, key_id, issuer_id]):
        print(
            "ERROR: APPLE_API_KEY_PATH, APPLE_API_KEY, APPLE_API_ISSUER "
            "must all be set",
            file=sys.stderr,
        )
        return 1
    if not os.path.isfile(key_path):
        print(f"ERROR: key file not found: {key_path}", file=sys.stderr)
        return 1

    print("Generating ES256 JWT…")
    jwt_token = create_es256_jwt(key_path, key_id, issuer_id)

    app_id = resolve_app_id(jwt_token, args.bundle_id)
    print(f"App id: {app_id}")

    versions = fetch_versions(
        jwt_token, app_id, args.version_string, args.platform
    )
    state = (
        versions[0].get("attributes", {}).get("appStoreState")
        if versions
        else None
    )
    action = decide_action(state)
    print(
        f"{args.platform} {args.version_string}: "
        f"state={state or 'not found'} → action={action.value}"
    )

    if action == Action.NOOP:
        print(
            "✅ Nothing to do — version is already released/replaced "
            f"(state={state})"
        )
        return 0
    if action == Action.UNKNOWN:
        print(
            f"ERROR: unknown appStoreState {state!r} — classify it in "
            "asc_set_automatic_release.py (PATCH_STATES / NOOP_STATES) "
            "before trusting this step with a new state.",
            file=sys.stderr,
        )
        return 1
    if action == Action.PATCH:
        patch_release_type(
            jwt_token, versions[0]["id"], args.version_string, args.platform
        )
        return 0

    create_version(jwt_token, app_id, args.version_string, args.platform)
    return 0


if __name__ == "__main__":
    sys.exit(main())
