#!/usr/bin/env python3
"""Delete stale iOS App Store provisioning profiles for a bundle ID.

When a capability (e.g. Sign in with Apple) is enabled on an App ID, App
Store Connect keeps handing out the EXISTING profile — it does not gain the
new entitlement until the profile is regenerated. xcodebuild
(-allowProvisioningUpdates) reuses that cached profile, so the exported
.ipa ends up without the entitlement and the runtime sheet fails with
ASAuthorizationError.unknown (1000).

This script removes the matching IOS_APP_STORE profiles; the next
xcodebuild run creates a fresh one that includes all current capabilities.

Usage:
    APPLE_API_KEY_PATH=path/to/AuthKey.p8 \\
    APPLE_API_KEY=KEYID \\
    APPLE_API_ISSUER=ISSUER \\
    python3 manage_ios_profiles.py --bundle-id net.uwuwu.origa [--dry-run]

Exit codes:
    0 — profiles deleted (or nothing to delete with --dry-run)
    1 — API error
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.request

from download_macos_profile import create_es256_jwt

ASC_API_BASE = "https://api.appstoreconnect.apple.com/v1"


def asc_request(path: str, method: str = "GET", body: dict | None = None) -> dict:
    key_path = os.environ.get("APPLE_API_KEY_PATH", "")
    key_id = os.environ.get("APPLE_API_KEY", "")
    issuer_id = os.environ.get("APPLE_API_ISSUER", "")
    if not (key_path and key_id and issuer_id):
        raise SystemExit(
            "ERROR: APPLE_API_KEY_PATH, APPLE_API_KEY, APPLE_API_ISSUER env vars are required"
        )
    token = create_es256_jwt(key_path, key_id, issuer_id)
    request = urllib.request.Request(
        f"{ASC_API_BASE}{path}",
        method=method,
        data=json.dumps(body).encode() if body is not None else None,
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
        },
    )
    with urllib.request.urlopen(request) as response:
        return json.loads(response.read().decode())


def find_bundle_id_resource(identifier: str) -> dict:
    response = asc_request(f"/bundleIds?filter[identifier]={identifier}")
    data = response.get("data", [])
    if not data:
        raise SystemExit(f"::error::No bundleId resource found for {identifier}")
    return data[0]


def list_profiles(bundle_id_resource_id: str) -> list[dict]:
    response = asc_request(f"/bundleIds/{bundle_id_resource_id}/profiles")
    return [
        profile
        for profile in response.get("data", [])
        # IOS_APP_STORE profiles only: leave development/ad-hoc alone.
        if profile.get("attributes", {}).get("profileType") == "IOS_APP_STORE"
    ]


def delete_profile(profile_id: str) -> None:
    asc_request(f"/profiles/{profile_id}", method="DELETE")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Delete stale IOS_APP_STORE profiles for a bundle ID"
    )
    parser.add_argument("--bundle-id", required=True, help="e.g. net.uwuwu.origa")
    parser.add_argument(
        "--dry-run", action="store_true", help="List matching profiles without deleting"
    )
    args = parser.parse_args()

    bundle = find_bundle_id_resource(args.bundle_id)
    profiles = list_profiles(bundle["id"])
    if not profiles:
        print(f"No IOS_APP_STORE profiles for {args.bundle_id} — nothing to delete")
        return 0

    for profile in profiles:
        name = profile.get("attributes", {}).get("name", "?")
        uuid = profile.get("attributes", {}).get("uuid", "?")
        print(
            f"{'[dry-run] would delete' if args.dry_run else 'deleting'} "
            f"profile {profile['id']} ({name}, uuid {uuid})"
        )
        if not args.dry_run:
            delete_profile(profile["id"])

    print(
        "Done. The next xcodebuild -allowProvisioningUpdates run will create a "
        "fresh profile that includes all current App ID capabilities."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
