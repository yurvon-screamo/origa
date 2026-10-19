#!/usr/bin/env bash
# Compute the Google Play versionCode for release-tag builds.
#
# Used by .github/workflows/_build-tauri.yml (build-android, tag runs only)
# to override tauri-cli's default versionCode. tauri-cli ignores the
# pre-release segment (major*1_000_000 + minor*1_000 + patch), which gives
# 0.8.0-rc1, 0.8.0-rc2 and 0.8.0 the SAME code — Google Play forbids reusing
# a versionCode, so the second RC of a train and the stable itself would be
# impossible to deliver (issue #747).
#
# Scheme: BASE = major*1_000_000 + minor*1_000 + patch
#         stable X.Y.Z     -> BASE*100 + 99
#         prerelease -rcN  -> BASE*100 + N   (N in 1..98, no leading zeros)
# Invariants: rc1 < rc2 < ... < rc98 < stable of the same train; every new
# code exceeds all historical tauri-formula codes (verified 2026-10-13: max
# code in the Play tracks was 4004). PATCH > 99 would overflow into the next
# minor's version space — rejected. Codes above 2_100_000_000 are rejected
# at `>` (tauri-cli's own bound); a +offset code can never equal the bound.
#
# Usage: compute-android-versioncode.sh <version> <version_base> <version_type>
#   version       full version, e.g. 0.8.0 or 0.8.0-rc1
#   version_base  numeric X.Y.Z (the version action's version_base output)
#   version_type  stable | prerelease (the repo's `alpha` = untagged builds
#                 is rejected — this contract is tag-only)
# Prints the code on stdout; diagnostics on stderr. Exit 2 usage, 1 invalid.
#
# Local test matrix (run from the repo root; expectations inline):
#   .github/scripts/compute-android-versioncode.sh 0.8.0 0.8.0 stable            # 800099
#   .github/scripts/compute-android-versioncode.sh 0.8.0-rc1 0.8.0 prerelease   # 800001
#   .github/scripts/compute-android-versioncode.sh 0.8.0-rc98 0.8.0 prerelease  # 800098
#   .github/scripts/compute-android-versioncode.sh 0.7.99 0.7.99 stable         # 709999
#   .github/scripts/compute-android-versioncode.sh 20.0.99 20.0.99 stable       # 2000009999
#   All of these must FAIL (exit 1):
#   ... 0.8.0-rc99 0.8.0 prerelease   # rc range reserves 99 for stable
#   ... 0.8.0-rc0 0.8.0 prerelease    # rc0 is not a positive release number
#   ... 0.8.0-rc01 0.8.0 prerelease   # leading zero: rc01 and rc1 would collide
#   ... 0.8.0-rc.1 0.8.0 prerelease   # dotted rc forms are not deliverable
#   ... 0.8.0-rc1.2 0.8.0 prerelease  # dotted rc forms are not deliverable
#   ... 0.7.100 0.7.100 stable        # patch > 99 overflows into next minor
#   ... 21.0.99 21.0.99 stable        # exceeds the Play versionCode maximum
#   ... 0.8.1-alpha.5 0.8.1 alpha     # untagged builds have no Play delivery
set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo "usage: $0 <version> <version_base> <version_type>" >&2
    exit 2
fi

VERSION="$1"
VERSION_BASE="$2"
VERSION_TYPE="$3"

if [[ "$VERSION_TYPE" != "stable" && "$VERSION_TYPE" != "prerelease" ]]; then
    echo "error: version_type must be 'stable' or 'prerelease', got '$VERSION_TYPE'" >&2
    exit 1
fi

if [[ ! "$VERSION_BASE" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "error: version_base must be numeric X.Y.Z, got '$VERSION_BASE'" >&2
    exit 1
fi

IFS='.' read -r MAJOR MINOR PATCH <<< "$VERSION_BASE"
MAJOR=$((10#$MAJOR))
MINOR=$((10#$MINOR))
PATCH=$((10#$PATCH))

if (( PATCH > 99 )); then
    echo "error: patch segment $PATCH exceeds 99 — BASE*100 would collide with the next minor's version space (bump the minor instead)" >&2
    exit 1
fi

BASE_CODE=$((MAJOR * 1000000 + MINOR * 1000 + PATCH))

RC_OFFSET=99
if [[ "$VERSION_TYPE" == "prerelease" ]]; then
    RC_SUFFIX="${VERSION#*-}"
    if [[ ! "$RC_SUFFIX" =~ ^rc(0|[1-9][0-9]*)$ ]]; then
        echo "error: prerelease suffix '$RC_SUFFIX' is not rcN with no leading zeros — rc01/rc.1/rc1.2 forms would produce colliding or ambiguous Play versionCodes" >&2
        exit 1
    fi
    RC_OFFSET=$((10#${RC_SUFFIX#rc}))
    if (( RC_OFFSET < 1 || RC_OFFSET > 98 )); then
        echo "error: rc number $RC_OFFSET is out of the 1..98 range (99 is reserved for the stable of the same train)" >&2
        exit 1
    fi
fi

VERSION_CODE=$((BASE_CODE * 100 + RC_OFFSET))

if (( VERSION_CODE > 2100000000 )); then
    echo "error: versionCode $VERSION_CODE exceeds the Play maximum 2100000000 — retire the major version line before tagging" >&2
    exit 1
fi

echo "$VERSION_CODE"
