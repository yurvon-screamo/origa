#!/usr/bin/env python3
"""Web-facing certificate clock for Origa: read-only checks of the public
TLS endpoints, Railway custom-domain certificate state, and the uwuwu.net
domain expiry.

Guards the web delivery surface (companion to the Apple/ASC-side checker
that lives on the marketing branch). Incident 2026-11-06: Railway skipped
the scheduled renewal of two custom-domain certificates (app.origa.uwuwu.net
expired Nov 4, origa.uwuwu.net was hours from expiry) while ``domain
status`` showed ``CERTIFICATE_STATUS_TYPE_ISSUE_FAILED`` with
``retryable: true`` — recoverable with ``railway domain certificate
retry``, but invisible until a browser shows a TLS warning. This script
surfaces that state weeks earlier.

Checks (all read-only):
  1. TLS notAfter for the public endpoints.
  2. Railway custom-domain certificate status via ``railway domain status``
     (needs CLI auth; skipped gracefully with ``--no-railway``).
  3. uwuwu.net registrar expiry via RDAP (rdap.org).

Style note: ``timezone.utc`` with bare ``# noqa: UP017`` instead of the
``datetime.UTC`` alias — the alias needs Python 3.11 while
``scripts/pyproject.toml`` declares ``requires-python >= 3.10``.

TLS vantage point: ADR-061 geo-splits uwuwu.net per record, so each probe
host is resolved individually and rows whose connection lands on the RF VM
anchor (193.233.217.243 — its own acme.sh certificate, independent of the
Railway edge guarded here) are annotated inline; those hosts' Railway-side
dates are simply not measured from this machine.

Exit codes (highest wins):
  3 — infrastructure error (network, CLI missing/crashed, unparseable
      output). Masks everything below on purpose: a blind checker must
      page louder than an actionable finding.
  2 — Railway certificate issuance failed with retryable=true
      (run: railway domain certificate retry <domain> ...).
  1 — TLS/RDAP expiry inside the warning window (list in stdout).
  0 — everything healthy, nothing expires within WARN_DAYS.
"""

from __future__ import annotations

import argparse
import json
import os
import socket
import ssl
import subprocess
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from datetime import datetime, timezone

WARN_DAYS = 30

# ADR-061 RF-branch anchor: RF-resolved clients terminate TLS on the RF VM
# (its own acme.sh certificate), not on the Railway edge guarded here.
RF_VM_IP = "193.233.217.243"

# (hostname, note) probed directly at :443. s3 rides CloudFront; the three
# custom domains ride the Railway edge whose certificates this script guards.
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


def tls_expiry(
    host: str,
    resolved_ip: str,
    port: int = 443,
    timeout: float = 15.0,
) -> datetime | None:
    """Connect to ``resolved_ip`` with proper SNI for ``host`` and return the
    leaf certificate's notAfter (UTC).

    Pinning the IP the caller resolved keeps the vantage annotation and the
    certificate on the same server: ADR-061 geo-split answers per query, so a
    separate resolve inside the connect could land on a different branch.
    """
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    ctx.check_hostname = False
    ctx.verify_mode = ssl.CERT_NONE  # dates only, chain trust is out of scope
    with (
        socket.create_connection((resolved_ip, port), timeout=timeout) as sock,
        ctx.wrap_socket(sock, server_hostname=host) as tls,
    ):
        der = tls.getpeercert(binary_form=True)
    pem = ssl.DER_cert_to_PEM_cert(der)
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
                    tzinfo=timezone.utc  # noqa: UP017
                )
            except ValueError:
                return None
    return None


def days_left(expiry: datetime | None, now: float | None = None) -> float | None:
    """Days until expiry; negative when already expired."""
    if expiry is None:
        return None
    now_dt = datetime.fromtimestamp(
        time.time() if now is None else now,
        tz=timezone.utc,  # noqa: UP017
    )
    return (expiry - now_dt).total_seconds() / 86400


def detect_vantage_from_ips(resolved_ips: set[str]) -> str:
    """Classify one host's resolution: ``rf`` when any resolved IP is the RF
    VM anchor (ADR-061), else ``world``."""
    if RF_VM_IP in resolved_ips:
        return "rf"
    return "world"


@dataclass
class RailwayStatus:
    """Parsed ``railway domain status`` fields we act on."""

    cert_status: str = ""
    error_type: str = ""
    retryable: bool = False
    verified: bool | None = None

    @property
    def issue_failed_retryable(self) -> bool:
        return (
            self.cert_status == "CERTIFICATE_STATUS_TYPE_ISSUE_FAILED"
            and self.retryable
        )

    @property
    def issuing(self) -> bool:
        return self.cert_status == "CERTIFICATE_STATUS_TYPE_ISSUING"

    @property
    def recognized(self) -> bool:
        """False when the CLI output drifted and nothing was parsed."""
        return bool(self.cert_status)


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


def railway_status(
    domain: str, service: str, project: str, environment: str
) -> RailwayStatus:
    """Query ``railway domain status``; raises RuntimeError on CLI failure."""
    env = dict(os.environ)
    env.setdefault("RAILWAY_CALLER", "origa:ssl_cert_check")
    env.setdefault("RAILWAY_AGENT_SESSION", f"ssl-cert-check-{int(time.time())}")
    result = subprocess.run(
        [
            "railway",
            "domain",
            "status",
            domain,
            "-s",
            service,
            "-e",
            environment,
            "--project",
            project,
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


def parse_rdap_events(events: list[dict]) -> datetime | None:
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
            dt = dt.replace(tzinfo=timezone.utc)  # noqa: UP017
        return dt
    return None


def run_tls_checks(now: float) -> tuple[list[str], list[str]]:
    """TLS section: prints the report, returns (warnings, hard_errors).

    Per-host vantage (ADR-061 geo-split is per record): every probe host is
    resolved on its own; rows landing on the RF VM anchor are annotated
    inline and excluded from Railway-edge claims — the script never states
    more than it observed.
    """
    warnings: list[str] = []
    hard_errors: list[str] = []

    print("== Public TLS endpoints ==")
    print(
        "! vantage: resolved per host (ADR-061 geo-split); rows annotated "
        "[RF-VM anchor] show the RF VM's acme.sh certificate — the Railway "
        "edge is not measured for them"
    )
    rf_anchored_rows: list[str] = []

    for host, note in TLS_ENDPOINTS:
        try:
            infos = socket.getaddrinfo(host, 443, proto=socket.IPPROTO_TCP)
        except OSError as exc:
            hard_errors.append(f"DNS resolution failed for {host}: {exc}")
            print(f"- {host:26s} {note:32s} DNS FAILED: {exc}")
            continue
        resolved_ip = infos[0][4][0]
        rf_anchored = detect_vantage_from_ips({info[4][0] for info in infos}) == "rf"
        try:
            expiry = tls_expiry(host, resolved_ip)
        except Exception as exc:  # noqa: BLE001 — report any probe failure
            hard_errors.append(f"TLS probe failed for {host}: {exc}")
            print(f"- {host:26s} {note:32s} PROBE FAILED: {exc}")
            continue
        if expiry is None:
            hard_errors.append(f"no notAfter parsed for {host}")
            print(f"- {host:26s} {note:32s} PARSE FAILED (no notAfter)")
            continue
        left = days_left(expiry, now)
        state = "OK" if left >= WARN_DAYS else ("EXPIRED" if left < 0 else "EXPIRING")
        row = f"- {host:26s} {note:32s} via={resolved_ip} expires={expiry:%Y-%m-%d} ({left:.0f} d) [{state}]"
        if rf_anchored:
            row += " [RF-VM anchor]"
            rf_anchored_rows.append(host)
        print(row)
        if left < WARN_DAYS:
            warnings.append(
                f"TLS: {host} ({note}) expires {expiry:%Y-%m-%d %H:%M} UTC "
                f"({'already expired' if left < 0 else f'in {left:.0f} days'})"
            )

    if rf_anchored_rows:
        joined = ", ".join(rf_anchored_rows)
        print(
            f"! RF-anchored ({joined}): dates above are the RF VM's acme.sh "
            f"certificate; the Railway edge was NOT measured for these hosts "
            f"from this machine"
        )
    return warnings, hard_errors


def run_railway_checks() -> tuple[list[str], list[str]]:
    """Railway section: prints the report, returns (retries, hard_errors)."""
    retries: list[str] = []
    hard_errors: list[str] = []

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
        if not status.recognized:
            hard_errors.append(
                f"Railway: unparseable status output for {domain} "
                "(CLI format drift? re-pin parse_railway_status)"
            )
            print(
                f"- {domain:26s} service={service:16s} "
                "UNRECOGNIZED OUTPUT (format drift)"
            )
            continue
        suffix = ""
        if status.issue_failed_retryable:
            suffix = " -> run: railway domain certificate retry " + domain
            retries.append(
                f"Railway: {domain} cert ISSUE_FAILED (retryable) — retry needed"
            )
        elif status.issuing:
            suffix = " (issuance in progress)"
        print(f"- {domain:26s} service={service:16s} cert={status.cert_status}{suffix}")
    return retries, hard_errors


def run_rdap_check(now: float) -> tuple[list[str], list[str]]:
    """RDAP section: prints the report, returns (warnings, hard_errors)."""
    warnings: list[str] = []
    hard_errors: list[str] = []

    print("\n== Registrar expiry (RDAP) ==")
    try:
        expiry = rdap_expiry()
    except Exception as exc:  # noqa: BLE001
        hard_errors.append(f"RDAP probe failed for {RDAP_DOMAIN}: {exc}")
        print(f"- {RDAP_DOMAIN:26s} PROBE FAILED: {exc}")
        return warnings, hard_errors
    left = days_left(expiry, now)
    if left is None:
        hard_errors.append(f"RDAP: no expiration event for {RDAP_DOMAIN}")
        print(f"- {RDAP_DOMAIN:26s} NO EXPIRATION EVENT")
        return warnings, hard_errors
    state = "OK" if left >= WARN_DAYS else ("EXPIRED" if left < 0 else "EXPIRING")
    print(
        f"- {RDAP_DOMAIN:26s} domain expires={expiry:%Y-%m-%d} ({left:.0f} d) [{state}]"
    )
    if left < WARN_DAYS:
        warnings.append(
            f"Registrar: {RDAP_DOMAIN} expires {expiry:%Y-%m-%d} "
            f"({'already expired' if left < 0 else f'in {left:.0f} days'}) "
            "— renew with the registrar (OnlineNIC)"
        )
    return warnings, hard_errors


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

    now = time.time()
    warnings: list[str] = []
    retries: list[str] = []
    hard_errors: list[str] = []

    tls_warn, tls_err = run_tls_checks(now)
    warnings += tls_warn
    hard_errors += tls_err

    if not args.no_railway:
        rr, re_ = run_railway_checks()
        retries += rr
        hard_errors += re_

    if not args.no_rdap:
        rdap_warn, rdap_err = run_rdap_check(now)
        warnings += rdap_warn
        hard_errors += rdap_err

    if warnings:
        print(f"\n!! EXPIRING WITHIN {WARN_DAYS} DAYS:")
        for w in warnings:
            print(f"!! {w}")
    if retries:
        print("\n!! RAILWAY CERTIFICATE RETRY NEEDED:")
        for r in retries:
            print(f"!! {r}")
    if hard_errors:
        print("\n!! PROBE ERRORS (infrastructure, not expiry):")
        for e in hard_errors:
            print(f"!! {e}")

    # Priority: 3 > 2 > 1 > 0. A broken probe means the checker is blind and
    # must page louder than any actionable finding it did manage to collect.
    if hard_errors:
        return 3
    if retries:
        return 2
    if warnings:
        return 1
    print(f"\nOK: nothing expires within {WARN_DAYS} days, no failed Railway issuances")
    return 0


if __name__ == "__main__":
    sys.exit(main())
