#!/usr/bin/env python3
"""Mint a throwaway Apple-issued distribution identity for iOS re-signing.

tauri-cli signs the iOS export with an Apple-issued "Apple Distribution"
certificate it mints at build time via the App Store Connect API (the
certificate lives in an ephemeral keychain deleted right after the build),
and the embedded provisioning profile contains that same certificate —
which is why uploads validate. There is no way to reuse that identity
afterwards, and App Store Connect rejects anything else with
-19241 "must be signed with the certificate that is contained in the
provisioning profile".

This script mirrors the trick deterministically:

  mint    — purge leftovers from previous runs (profiles by the
            "origa-ci-" name prefix, their certificates by id — Apple
            rewrites certificate CNs to the account holder name, so a
            name-based certificate match would never fire), generate a
            key + CSR, mint a new Apple-issued distribution certificate
            and an IOS_APP_STORE profile for the bundle that contains it,
            then write key.pem / cert.pem / profile.mobileprovision to
            --out-dir. The cert/profile ids are appended to $GITHUB_OUTPUT
            immediately after each resource is created so a crash cannot
            orphan a certificate without its id.
  cleanup — delete the minted certificate and profile (call in an always()
            step; frees the limited certificate slots). 404 is tolerated:
            a parallel run may have purged the same leftovers.

Usage:
    APPLE_API_KEY_PATH=... APPLE_API_KEY=... APPLE_API_ISSUER=... \
    mint_ios_identity.py mint --bundle-id net.uwuwu.origa --out-dir /tmp/id
    mint_ios_identity.py cleanup --cert-id ID --profile-id ID

Exit codes: 0 ok, 1 API/validation error.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request

from download_macos_profile import create_es256_jwt

ASC_API_BASE = "https://api.appstoreconnect.apple.com/v1"


def asc_request(
    path: str, method: str = "GET", body: dict | None = None, *, ignore_missing: bool = False
) -> dict:
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
    try:
        with urllib.request.urlopen(request) as response:
            raw = response.read().decode()
            # DELETE returns 204 No Content with an empty body.
            return json.loads(raw) if raw.strip() else {}
    except urllib.error.HTTPError as error:
        detail = error.read().decode(errors="replace")
        if ignore_missing and error.code == 404:
            print(f"{method} {path} -> 404 (already gone, tolerated)")
            return {}
        raise SystemExit(f"::error::ASC API {method} {path} -> {error.code}: {detail[:2000]}")


def expect_data(response: dict, what: str) -> dict:
    data = response.get("data")
    if not isinstance(data, dict) or "id" not in data:
        raise SystemExit(
            f"::error::unexpected ASC API response for {what}: {json.dumps(response)[:1000]}"
        )
    return data


def run_openssl(args: list[str]) -> bytes:
    result = subprocess.run(["openssl", *args], capture_output=True)
    if result.returncode != 0:
        raise SystemExit(
            f"::error::openssl {' '.join(args)} failed: "
            f"{result.stderr.decode(errors='replace').strip()[:1000]}"
        )
    return result.stdout


def write_output(key: str, value: str) -> None:
    # Append immediately after each resource is created: a later crash must
    # not orphan a minted certificate without leaving its id for cleanup.
    output_path = os.environ.get("GITHUB_OUTPUT")
    if output_path:
        with open(output_path, "a", encoding="utf-8") as output:
            output.write(f"{key}={value}\n")


def find_bundle_id_resource(identifier: str) -> dict:
    response = asc_request(f"/bundleIds?filter[identifier]={identifier}")
    data = response.get("data", [])
    if not data:
        raise SystemExit(f"::error::No bundleId resource found for {identifier}")
    return data[0]


def purge_previous_ci_identities() -> None:
    # Profile names are fully under our control (ASC preserves them), so
    # stale runs are found by the "origa-ci-" prefix and their certificates
    # are unlinked by id through the profile relationship. Certificates are
    # deliberately NOT matched by name: Apple rewrites certificate CNs to
    # the account holder name, so a CN match would never fire (and would be
    # dangerous if it did). A same-name profile would also 409 on retries
    # of the same commit. Growth is bounded to at most a few profiles, well
    # under the 200-entry listing below.
    response = asc_request("/profiles?limit=200")
    for profile in response.get("data", []):
        name = profile.get("attributes", {}).get("name", "")
        if not name.startswith("origa-ci-"):
            continue
        profile_id = profile["id"]
        certs = asc_request(f"/profiles/{profile_id}/relationships/certificates")
        cert_ids = [cert["id"] for cert in certs.get("data", [])]
        print(f"deleting stale CI profile {profile_id} ({name})")
        asc_request(f"/profiles/{profile_id}", method="DELETE")
        for cert_id in cert_ids:
            print(f"deleting stale CI certificate {cert_id}")
            asc_request(f"/certificates/{cert_id}", method="DELETE")


def mint(bundle_id: str, out_dir: str, suffix: str) -> None:
    purge_previous_ci_identities()

    os.makedirs(out_dir, exist_ok=True)
    key_path = f"{out_dir}/key.pem"
    csr_path = f"{out_dir}/csr.pem"

    run_openssl(["genrsa", "-out", key_path, "2048"])
    run_openssl(
        [
            "req",
            "-new",
            "-key",
            key_path,
            "-out",
            csr_path,
            "-subj",
            "/CN=Apple Distribution: Origa CI/O=Origa/C=US",
        ]
    )
    with open(csr_path, encoding="utf-8") as csr_file:
        csr = csr_file.read()

    response = asc_request(
        "/certificates",
        method="POST",
        body={
            "data": {
                "type": "certificates",
                "attributes": {"certificateType": "DISTRIBUTION", "csrContent": csr},
            }
        },
    )
    cert = expect_data(response, "certificate creation")
    cert_id = cert["id"]
    # Apple rewrites the CN to the account holder name — the printed CN is
    # only what we requested in the CSR.
    print(f"minted certificate {cert_id}")
    write_output("cert-id", cert_id)
    with open(f"{out_dir}/cert.pem", "wb") as cert_file:
        cert_file.write(base64.b64decode(cert["attributes"]["certificateContent"]))

    bundle = find_bundle_id_resource(bundle_id)
    response = asc_request(
        "/profiles",
        method="POST",
        body={
            "data": {
                "type": "profiles",
                "attributes": {
                    "name": f"origa-ci-{suffix}",
                    "profileType": "IOS_APP_STORE",
                },
                "relationships": {
                    "bundleId": {"data": {"type": "bundleIds", "id": bundle["id"]}},
                    "certificates": {"data": [{"type": "certificates", "id": cert_id}]},
                },
            }
        },
    )
    profile = expect_data(response, "profile creation")
    profile_id = profile["id"]
    print(f"created profile {profile_id}: origa-ci-{suffix}")
    write_output("profile-id", profile_id)
    with open(f"{out_dir}/profile.mobileprovision", "wb") as profile_file:
        profile_file.write(base64.b64decode(profile["attributes"]["profileContent"]))


def cleanup(cert_id: str, profile_id: str) -> None:
    if profile_id:
        asc_request(f"/profiles/{profile_id}", method="DELETE", ignore_missing=True)
        print(f"deleted profile {profile_id}")
    if cert_id:
        asc_request(f"/certificates/{cert_id}", method="DELETE", ignore_missing=True)
        print(f"deleted certificate {cert_id}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="command", required=True)

    mint_parser = sub.add_parser("mint")
    mint_parser.add_argument("--bundle-id", required=True)
    mint_parser.add_argument("--out-dir", required=True)
    mint_parser.add_argument("--suffix", default="tmp")

    cleanup_parser = sub.add_parser("cleanup")
    cleanup_parser.add_argument("--cert-id", default="")
    cleanup_parser.add_argument("--profile-id", default="")

    args = parser.parse_args()
    if args.command == "mint":
        mint(args.bundle_id, args.out_dir, args.suffix)
    else:
        cleanup(args.cert_id, args.profile_id)
    return 0


if __name__ == "__main__":
    sys.exit(main())
