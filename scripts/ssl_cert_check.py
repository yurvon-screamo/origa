#!/usr/bin/env python3
"""Web-facing certificate clock for Origa: read-only checks of the public
TLS endpoints, Railway custom-domain certificate state, and the uwuwu.net
domain expiry.

Companion to ``marketing/tools/asc_cert_check.py`` (ASC/Apple side) — this
one guards the web delivery surface. Incident 2026-11-06 (#891-class):
Railway skipped the scheduled renewal of two custom-domain certificates
(app.origa.uwuwu.net expired Nov 4, origa.uwuwu.net was hours from expiry)
while ``domain status`` showed ``CERTIFICATE_STATUS_TYPE_ISSUE_FAILED``
with ``retryable: true`` — recoverable with ``railway domain certificate
retry``, but invisible until a browser shows a TLS warning. This script
surfaces that state weeks earlier.

Checks (all read-only):
  1. TLS notAfter for the public endpoints (world branch, direct SNI).
  2. Railway custom-domain certificate status via ``railway domain status``
     (needs CLI auth; skipped gracefully with ``--no-railway``).
  3. uwuwu.net registrar expiry via RDAP (rdap.org).

Exit codes:
  0 — everything healthy, nothing expires within WARN_DAYS
  1 — TLS/RDAP expiry inside the warning window (list in stdout)
  2 — Railway certificate issuance failed with retryable=true
      (run: railway domain certificate retry <domain> ...)
  3 — infrastructure error (network, CLI missing/crashed, bad response)
"""
from __future__ import annotations

import argparse
import json
import os
import ssl
import subprocess
import sys
import time
import urllib.error
import urllib.request
from datetime import UTC, datetime

WARN_DAYS = 30

# (hostname, note) probed directly at :443 — the world branch (Railway edge
# for origa/app/umami, CloudFront for s3). The RF branch terminates TLS on
# the RF VM with its own acme.sh certificate and is covered by the VM-side
# probe; this script answers "what does a normal user outside RF see".
TLS_ENDPOINTS = [
    ("origa.uwuwu.net", "landing (Railway custom domain)"),
    ("app.origa.uwuwu.net", "app/TrailBase (Railway custom domain)"),
    ("s3.origa.uwuwu.net", "CDN (CloudFront)"),
    ("umami.origa.uwuwu.net", "analytics (Railway custom domain)"),
]

# Railway side: domain -> owning service (project origa-appuru).
RAILWAY_PROJECT = "e7a27898-cbd4-4597-bf26-8870cbc2bcbd"
RAILWAY_ENVIRONMENT = "production"
RAILWAY_DOMAINS = [
    ("origa.uwuwu.net", "landing"),
    ("app.origa.uwuwu.net", "origa-trailbase"),
    ("umami.origa.uwuwu.net", "umami"),
]

RDAP_DOMAIN = "uwuwu.net"
RDAP_URL = f"https://rdap.org/domain/{RDAP_DOMAIN}"


def tls_expiry(host: str, port: int = 443, timeout: float = 15.0) -> datetime | None:
    """Connect with SNI and return the leaf certificate's notAfter (UTC)."""
    pem = ssl.get_server_certificate((host, port))
    result = subprocess.run(
        ["openssl", "x509", "-noout", "-enddate"],
        input=pem.encode(),
        capture_output=True,
        check=True,
    )
    return parse_openssl_enddate(result.stdout.decode())


def parse_openssl_enddate(text: str) -> datetime | None:
    """Parse ``notAfter=Nov  4 17:49:24 2026 GMT`` into an aware UTC datetime."""
    for line in text.splitlines():
        if line.startswith("notAfter="):
            raw = line.split("=", 1)[1].strip()
            try:
                return datetime.strptime(raw, "%b %d %H:%M:%S %Y %Z").replace(
                    tzinfo=UTC
                )
            except ValueError:
                return None
    return None


def days_left(expiry: datetime | None, now: float | None = None) -> float | None:
    """Days until expiry; negative when already expired."""
    if expiry is None:
        return None
    now_dt = datetime.fromtimestamp(
        time.time() if now is None else now, tz=UTC
    )
    return (expiry - now_dt).total_seconds() / 86400


class RailwayStatus:
    """Parsed ``railway domain status`` fields we act on."""

    def __init__(
        self,
        cert_status: str = "",
        error_type: str = "",
        retryable: bool = False,
        verified: bool | None = None,
    ) -> None:
        self.cert_status = cert_status
        self.error_type = error_type
        self.retryable = retryable
        self.verified = verified

    @property
    def issue_failed_retryable(self) -> bool:
        return self.cert_status == "CERTIFICATE_STATUS_TYPE_ISSUE_FAILED" and self.retryable

    @property
    def issuing(self) -> bool:
        return self.cert_status == "CERTIFICATE_STATUS_TYPE_ISSUING"


def parse_railway_status(text: str) -> RailwayStatus:
    """Parse the CLI's stable ``Key: VALUE`` report. Unknown lines are ignored."""
    out = RailwayStatus()
    for line in text.splitlines():
        if ":" not in line:
            continue
        key, _, value = line.partition(":")
        key = key.strip()
        value = value.strip()
        if key == "Certificate status":
            out.cert_status = value
        elif key == "Certificate error type":
            out.error_type = value
        elif key == "Certificate retryable":
            out.retryable = value.lower() == "true"
        elif key == "Verified":
            out.verified = value.lower() == "yes"
    return out


def railway_status(domain: str, service: str, project: str, environment: str) -> RailwayStatus:
    """Query ``railway domain status``; raises RuntimeError on CLI failure."""
    env = dict(os.environ)
    env.setdefault("RAILWAY_CALLER", "origa:ssl_cert_check")
    env.setdefault("RAILWAY_AGENT_SESSION", f"ssl-cert-check-{int(time.time())}")
    result = subprocess.run(
        [
            "railway", "domain", "status", domain,
            "-s", service,
            "-e", environment,
            "--project", project,
        ],
        capture_output=True,
        text=True,
        timeout=60,
        env=env,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"railway domain status {domain} failed (exit {result.returncode}): "
            f"{(result.stderr or result.stdout).strip()[:300]}"
        )
    return parse_railway_status(result.stdout)


def rdap_expiry(url: str = RDAP_URL, timeout: float = 20.0) -> datetime | None:
    """Registrar expiry date from the RDAP events list (UTC)."""
    req = urllib.request.Request(url, headers={"Accept": "application/rdap+json"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        data = json.loads(resp.read())
    return parse_rdap_events(data.get("events", []))


def parse_rdap_events(events: list) -> datetime | None:
    """Extract the ``expiration`` event date from an RDAP payload."""
    for event in events:
        if event.get("eventAction") != "expiration":
            continue
        raw = str(event.get("eventDate", ""))
        if not raw:
            return None
        # RDAP dates are ISO-8601, usually with fractional seconds and Z.
        raw = raw.replace("Z", "+00:00")
        try:
            dt = datetime.fromisoformat(raw)
        except ValueError:
            return None
        if dt.tzinfo is None:
            dt = dt.replace(tzinfo=UTC)
        return dt
    return None


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Read-only certificate clock for Origa's web surface."
    )
    parser.add_argument(
        "--no-railway",
        action="store_true",
        help="skip Railway domain status checks (no CLI auth needed)",
    )
    parser.add_argument(
        "--no-rdap",
        action="store_true",
        help="skip the uwuwu.net registrar expiry check",
    )
    args = parser.parse_args()

    warnings: list[str] = []
    retries_needed: list[str] = []
    hard_errors: list[str] = []

    now = time.time()

    print("== Public TLS endpoints (world branch) ==")
    for host, note in TLS_ENDPOINTS:
        try:
            expiry = tls_expiry(host)
        except Exception as exc:  # noqa: BLE001 — report any probe failure
            hard_errors.append(f"TLS probe failed for {host}: {exc}")
            print(f"- {host:26s} {note:32s} PROBE FAILED: {exc}")
            continue
        left = days_left(expiry, now)
        if left is None:
            hard_errors.append(f"no notAfter parsed for {host}")
            continue
        state = "OK" if left >= WARN_DAYS else ("EXPIRED" if left < 0 else "EXPIRING")
        print(f"- {host:26s} {note:32s} expires={expiry:%Y-%m-%d} ({left:.0f} d) [{state}]")
        if left < WARN_DAYS:
            warnings.append(
                f"TLS: {host} ({note}) expires {expiry:%Y-%m-%d %H:%M} UTC "
                f"({'already expired' if left < 0 else f'in {left:.0f} days'})"
            )

    if not args.no_railway:
        print("\n== Railway custom-domain certificates ==")
        for domain, service in RAILWAY_DOMAINS:
            try:
                status = railway_status(
                    domain, service, RAILWAY_PROJECT, RAILWAY_ENVIRONMENT
                )
            except (RuntimeError, subprocess.TimeoutExpired, FileNotFoundError) as exc:
                hard_errors.append(f"Railway status failed for {domain}: {exc}")
                print(f"- {domain:26s} service={service:16s} CLI FAILED: {exc}")
                continue
            suffix = ""
            if status.issue_failed_retryable:
                suffix = " -> run: railway domain certificate retry " + domain
                retries_needed.append(
                    f"Railway: {domain} cert ISSUE_FAILED (retryable) — retry needed"
                )
            elif status.issuing:
                suffix = " (issuance in progress)"
            print(f"- {domain:26s} service={service:16s} "
                  f"cert={status.cert_status or '?'}{suffix}")

    if not args.no_rdap:
        print("\n== Registrar expiry (RDAP) ==")
        try:
            expiry = rdap_expiry()
        except Exception as exc:  # noqa: BLE001
            hard_errors.append(f"RDAP probe failed for {RDAP_DOMAIN}: {exc}")
            print(f"- {RDAP_DOMAIN:26s} PROBE FAILED: {exc}")
        else:
            left = days_left(expiry, now)
            if left is not None:
                state = "OK" if left >= WARN_DAYS else (
                    "EXPIRED" if left < 0 else "EXPIRING"
                )
                print(f"- {RDAP_DOMAIN:26s} domain expires={expiry:%Y-%m-%d} "
                      f"({left:.0f} d) [{state}]")
                if left < WARN_DAYS:
                    warnings.append(
                        f"Registrar: {RDAP_DOMAIN} expires {expiry:%Y-%m-%d} "
                        f"({'already expired' if left < 0 else f'in {left:.0f} days'}) "
                        "— renew with the registrar (OnlineNIC)"
                    )
            else:
                hard_errors.append(f"RDAP: no expiration event for {RDAP_DOMAIN}")

    if warnings:
        print(f"\n!! EXPIRING WITHIN {WARN_DAYS} DAYS:")
        for w in warnings:
            print(f"!! {w}")
    if retries_needed:
        print("\n!! RAILWAY CERTIFICATE RETRY NEEDED:")
        for r in retries_needed:
            print(f"!! {r}")
    if hard_errors:
        print("\n!! PROBE ERRORS (infrastructure, not expiry):")
        for e in hard_errors:
            print(f"!! {e}")

    if hard_errors:
        return 3
    if retries_needed:
        return 2
    if warnings:
        return 1
    print(f"\nOK: nothing expires within {WARN_DAYS} days, no failed Railway issuances")
    return 0


if __name__ == "__main__":
    sys.exit(main())
