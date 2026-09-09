#!/usr/bin/env python3
"""Fail-closed check that the provisioning profile GRANTS an entitlement.

The re-sign step merges app entitlements with profile entitlements as a
silent union — codesign does not validate app entitlements against the
profile, so a capability forgotten on the App ID (developer.apple.com)
produces a green build that App Store validation later rejects.

This script re-extracts the profile's own entitlements and fails when a
required key is not among them, i.e. when the key would only come from the
app's plist.

Usage:
    python3 verify_profile_entitlements.py \
        --profile embedded.provisionprofile \
        --entitlement com.apple.developer.applesignin

Exit codes:
    0 — every required entitlement is granted by the profile
    1 — at least one is missing (printed as a ::error annotation)
"""

from __future__ import annotations

import argparse
import sys

from download_macos_profile import extract_profile_entitlements


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Assert that required entitlements are granted by the "
        "provisioning profile itself"
    )
    parser.add_argument(
        "--profile", required=True, help="Path to the .provisionprofile file"
    )
    parser.add_argument(
        "--entitlement",
        action="append",
        required=True,
        help="Entitlement key that must be present in the profile "
        "(repeatable)",
    )
    args = parser.parse_args()

    profile_entitlements = extract_profile_entitlements(args.profile)

    missing = [key for key in args.entitlement if key not in profile_entitlements]
    if missing:
        print(
            f"::error::Entitlement(s) {missing} are NOT granted by the "
            "provisioning profile. Enable the corresponding capability on "
            "the App ID at developer.apple.com and let the profile refresh — "
            "the merge step would silently union these keys into the "
            "signature and App Store validation would reject the build."
        )
        return 1

    print(f"✅ Profile grants: {args.entitlement}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
