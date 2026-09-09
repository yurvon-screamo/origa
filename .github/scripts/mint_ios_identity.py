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

  mint    — delete stale CI-minted DISTRIBUTION certificates (dead weight:
            their private keys are lost), generate a key + CSR, mint a new
            Apple-issued "Apple Distribution: Origa CI" certificate and an
            IOS_APP_STORE profile for the bundle that contains it, then
            write key.pem / cert.pem / profile.mobileprovision to --out-dir.
  cleanup — delete the minted certificate and profile (call in an always()
            step; frees the limited certificate slots).

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

CERT_CN = "Apple Distribution: Origa CI"
# CI junk that can never be used again (private keys are gone with the
# runners): our own previous runs plus the ephemeral tauri-cli mints.
DEAD_CERT_CNS = {CERT_CN, "Apple Distribution: Tauri (unset)"}


def asc_request(path: str, method: str = "GET", body: dict | None = None) -> tuple[int, dict | bytes]:
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
            return response.status, (json.loads(raw) if raw.strip() else {})
    except urllib.error.HTTPError as error:
        detail = error.read().decode(errors="replace")
        raise SystemExit(f"::error::ASC API {method} {path} -> {error.code}: {detail[:2000]}")


def run_openssl(args: list[str]) -> bytes:
    result = subprocess.run(["openssl", *args], check=True, capture_output=True)
    return result.stdout


def find_bundle_id_resource(identifier: str) -> dict:
    _, response = asc_request(f"/bundleIds?filter[identifier]={identifier}")
    data = response.get("data", [])
    if not data:
        raise SystemExit(f"::error::No bundleId resource found for {identifier}")
    return data[0]


def delete_dead_certs() -> None:
    _, response = asc_request("/certificates?filter[certificateType]=DISTRIBUTION&limit=200")
    for cert in response.get("data", []):
        name = cert.get("attributes", {}).get("name", "")
        if name in DEAD_CERT_CNS:
            cert_id = cert["id"]
            print(f"deleting dead CI certificate {cert_id} ({name})")
            asc_request(f"/certificates/{cert_id}", method="DELETE")
    # Same-name profiles would 409 on retries of the same commit.
    _, response = asc_request("/profiles?limit=200")
    for profile in response.get("data", []):
        name = profile.get("attributes", {}).get("name", "")
        if name.startswith("origa-ci-"):
            profile_id = profile["id"]
            print(f"deleting stale CI profile {profile_id} ({name})")
            asc_request(f"/profiles/{profile_id}", method="DELETE")


def mint(bundle_id: str, out_dir: str, suffix: str) -> None:
    delete_dead_certs()

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
            f"/CN={CERT_CN}/O=Origa/C=US",
        ]
    )
    csr = open(csr_path).read()

    _, response = asc_request(
        "/certificates",
        method="POST",
        body={
            "data": {
                "type": "certificates",
                "attributes": {"certificateType": "DISTRIBUTION", "csrContent": csr},
            }
        },
    )
    cert_id = response["data"]["id"]
    cert_b64 = response["data"]["attributes"]["certificateContent"]
    cert_pem = base64.b64decode(cert_b64)
    open(f"{out_dir}/cert.pem", "wb").write(cert_pem)
    print(f"minted certificate {cert_id}: {CERT_CN}")

    bundle = find_bundle_id_resource(bundle_id)
    _, response = asc_request(
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
    profile_id = response["data"]["id"]
    profile_b64 = response["data"]["attributes"]["profileContent"]
    open(f"{out_dir}/profile.mobileprovision", "wb").write(base64.b64decode(profile_b64))
    print(f"created profile {profile_id}: origa-ci-{suffix}")

    # Pass the ids to the cleanup step via $GITHUB_OUTPUT-compatible files.
    with open(os.environ.get("GITHUB_OUTPUT", "/dev/null"), "a") as output:
        output.write(f"cert-id={cert_id}\n")
        output.write(f"profile-id={profile_id}\n")
    open(f"{out_dir}/ids.txt", "w").write(f"{cert_id} {profile_id}\n")


def cleanup(cert_id: str, profile_id: str) -> None:
    if profile_id:
        asc_request(f"/profiles/{profile_id}", method="DELETE")
        print(f"deleted profile {profile_id}")
    if cert_id:
        asc_request(f"/certificates/{cert_id}", method="DELETE")
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
