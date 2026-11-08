"""Integration tests for the RF uptime poller (``uptime_poller.sh``).

The poller is a bash script whose every outward touch (dig, curl, openssl) is
replaced by PATH stubs, so each check's outcome is controlled from the test
without touching the network. The tests pin the observable behavior contract:

* a full green cycle exits 0, keeps exactly one state line per check, and
  fires the heartbeat;
* the anti-flap contract: an incident is EXACTLY two consecutive failures
  (one ALERT), followed by one RESOLVE on recovery;
* dead RU resolvers are a WARN, not an incident;
* the umami certificate probe warns below the horizon (no HTTP pings);
* a 4xx from Railway is a failure (the silent custom-domain slap seen from
  RF must not stay green);
* the rf-branch guard accepts the live VM response shape — the VM's nginx
  proxies the Tigris origin and forwards its ``x-tigris-served-from`` header,
  so an answer carrying that header (with the VM ``server: nginx/<version>``
  signature) is the healthy RF branch, not a world leak (regression pin for
  the false-incident bug found in live testing on 2026-11-08).

Stub contract (read via ``FAKE_*`` environment variables):

* ``dig +short <host> @<resolver> <type>`` looks up
  ``FAKE_DIG_<host>_<resolver>_<type>`` (dots → underscores, empty answer if
  unset);
* ``curl``: ``--resolve`` prints ``FAKE_RF_BRANCH_RESPONSE`` (default: the
  live VM shape); the ``-w '%{http_code}'`` TSPU probe answers with
  ``FAKE_RAILWAY_<host>`` (dots → underscores, default "200"); telegram calls
  append to ``FAKE_TG_LOG``; the heartbeat call appends to
  ``FAKE_HEARTBEAT_LOG`` (``FAKE_HEARTBEAT_FAIL=1`` makes it fail);
* ``openssl -enddate`` prints ``notAfter=$FAKE_UMAMI_NOTAFTER``.
"""

from __future__ import annotations

import os
import stat
import subprocess
from pathlib import Path

REPO_SCRIPTS = Path(__file__).resolve().parent.parent
POLLER = REPO_SCRIPTS / "uptime_poller.sh"

RF_IP = "193.233.217.243"
RU_PRIMARY = "77.88.8.8"
RU_SECONDARY = "77.88.8.1"
WORLD_RESOLVER = "1.1.1.1"

# The live VM response shape captured on 2026-11-08: the VM's nginx proxies
# Tigris and forwards its origin headers (server-timing, x-tigris-*).
LIVE_VM_RESPONSE = (
    "HTTP/2 200\n"
    "server: nginx/1.28.3 (Ubuntu)\n"
    "content-type: application/json\n"
    "x-tigris-regions: nrt\n"
    "x-tigris-served-from: nrt\n"
)

# The world (CloudFront) shape: nginx without a version slash.
WORLD_RESPONSE = (
    "HTTP/2 200\n"
    "server: nginx\n"
    "x-tigris-served-from: nrt\n"
    "x-cache: Miss from cloudfront\n"
)

CNAME_TARGETS = {
    "origa.uwuwu.net": "c2qj368z.up.railway.app.",
    "app.origa.uwuwu.net": "9fmm6y4e.up.railway.app.",
    "umami.origa.uwuwu.net": "07wyrdxw.up.railway.app.",
    "content.origa.uwuwu.net": "origa.t3.tigrisbucket.io.",
    "s3.origa.uwuwu.net": "d3gbi3wo8j4c2w.cloudfront.net.",
}

CHECK_KEYS = [
    "fail:dns-ru",
    "fail:dns-world",
    "fail:ru-resolvers",
    "fail:rf-branch",
    "fail:rf-railway-origa.uwuwu.net",
    "fail:rf-railway-app.origa.uwuwu.net",
    "fail:umami-cert",
]


def _env_key(*parts: str) -> str:
    return "_".join(part.replace(".", "_") for part in parts)


DIG_STUB = r"""#!/usr/bin/env bash
_k() { local out="" s; for s in "$@"; do s="${s//./_}"; out+="${s}_"; done; printf '%s' "${out%_}"; }
host="$2"; resolver="${3#@}"; qtype="${4:-ANY}"
key="FAKE_DIG_$(_k "$host" "$resolver" "$qtype")"
answer="${!key:-}"
# "__FAIL__" simulates a dead resolver (non-zero exit), an empty value is a
# resolvable-but-empty answer (dig still exits 0 there).
[[ "$answer" == "__FAIL__" ]] && exit 1
[[ -n "$answer" ]] && printf '%s\n' "$answer"
exit 0
"""

CURL_STUB_TEMPLATE = r"""#!/usr/bin/env bash
LIVE_VM_RESPONSE_DEFAULT="{live_vm}"
if [[ "$*" == *"--resolve"* ]]; then
  printf '%b\n' "${{FAKE_RF_BRANCH_RESPONSE:-$LIVE_VM_RESPONSE_DEFAULT}}"
  exit 0
fi
if [[ "$*" == *"%{{http_code}}"* ]]; then
  last="${{@: -1}}"; host="${{last#https://}}"; host="${{host%/}}"
  key="FAKE_RAILWAY_${{host//./_}}"
  printf '%s\n' "${{!key:-200}}"
  exit 0
fi
if [[ "$*" == *"api.telegram.org"* ]]; then
  printf '%s\n' "$*" >> "$FAKE_TG_LOG"
  exit 0
fi
if [[ "${{FAKE_HEARTBEAT_FAIL:-0}}" == "1" ]]; then
  exit 1
fi
printf '%s\n' "$*" >> "$FAKE_HEARTBEAT_LOG"
exit 0
"""

OPENSSL_STUB = r"""#!/usr/bin/env bash
if [[ "$*" == *"-enddate"* ]]; then
  printf 'notAfter=%s\n' "$FAKE_UMAMI_NOTAFTER"
fi
exit 0
"""


def _stub_dir(tmp_path: Path) -> Path:
    bindir = tmp_path / "stubs"
    bindir.mkdir(exist_ok=True)
    curl_stub = CURL_STUB_TEMPLATE.format(live_vm=LIVE_VM_RESPONSE.replace("\n", "\\n"))
    for name, body in (
        ("dig", DIG_STUB),
        ("curl", curl_stub),
        ("openssl", OPENSSL_STUB),
    ):
        path = bindir / name
        path.write_text(body)
        path.chmod(path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    return bindir


def _base_env(tmp_path: Path) -> dict[str, str]:
    env = dict(os.environ)
    env["PATH"] = f"{_stub_dir(tmp_path)}:{env['PATH']}"
    env["POLLER_STATE_FILE"] = str(tmp_path / "state.txt")
    env["FAKE_TG_LOG"] = str(tmp_path / "telegram.log")
    env["FAKE_HEARTBEAT_LOG"] = str(tmp_path / "heartbeat.log")
    env["TG_BOT_TOKEN"] = "test-token"  # nosemgrep: python-hardcoded-secrets
    env["TG_CHAT_ID"] = "test-chat"
    env["HETRIX_HEARTBEAT_URL"] = "https://heartbeat.test/ping"
    env["FAKE_UMAMI_NOTAFTER"] = "Feb  6 11:14:26 2027 GMT"
    # Healthy DNS topology (plain CNAME everywhere, geo split on s3 only).
    for host, target in CNAME_TARGETS.items():
        env[_env_key("FAKE_DIG", host, WORLD_RESOLVER, "CNAME")] = target
        if host == "s3.origa.uwuwu.net":
            continue
        env[_env_key("FAKE_DIG", host, RU_PRIMARY, "CNAME")] = target
        env[_env_key("FAKE_DIG", host, RU_SECONDARY, "CNAME")] = target
    env[_env_key("FAKE_DIG", "s3.origa.uwuwu.net", RU_PRIMARY, "A")] = RF_IP
    env[_env_key("FAKE_DIG", "s3.origa.uwuwu.net", RU_SECONDARY, "A")] = RF_IP
    # Liveness probe queries without a record type (stub maps to ANY suffix).
    env[_env_key("FAKE_DIG", "uwuwu.net", RU_PRIMARY, "ANY")] = "bunny-ns-answer"
    env[_env_key("FAKE_DIG", "uwuwu.net", RU_SECONDARY, "ANY")] = "bunny-ns-answer"
    return env


def run_cycle(
    tmp_path: Path, overrides: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    env = _base_env(tmp_path)
    env.update(overrides or {})
    return subprocess.run(
        ["bash", str(POLLER)],
        env=env,
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )


def state_counts(tmp_path: Path) -> dict[str, int]:
    lines = (tmp_path / "state.txt").read_text().splitlines()
    return {parts[0]: int(parts[1]) for parts in (line.split() for line in lines)}


def telegram_messages(tmp_path: Path, check: str | None = None) -> list[str]:
    path = tmp_path / "telegram.log"
    if not path.exists():
        return []
    messages = [
        part.split("text=", 1)[1]
        for part in path.read_text().splitlines()
        if "text=" in part
    ]
    if check is not None:
        # ALERT/WARN use "] {check} failing...", RESOLVE uses "] {check}: recovered".
        messages = [message for message in messages if f"] {check}" in message]
    return messages


def broken_world_cname() -> dict[str, str]:
    return {
        _env_key("FAKE_DIG", "app.origa.uwuwu.net", WORLD_RESOLVER, "CNAME"): (
            "wrong.target.example."
        )
    }


class TestGreenCycle:
    def test_full_cycle_exits_zero_and_fires_heartbeat(self, tmp_path: Path) -> None:
        proc = run_cycle(tmp_path)
        assert proc.returncode == 0, proc.stderr
        assert "cycle end" in proc.stdout
        heartbeat_log = tmp_path / "heartbeat.log"
        assert heartbeat_log.exists(), "heartbeat must fire on every completed cycle"
        assert "heartbeat.test/ping" in heartbeat_log.read_text()

    def test_green_cycle_keeps_all_counters_at_zero(self, tmp_path: Path) -> None:
        run_cycle(tmp_path)
        counts = state_counts(tmp_path)
        assert sorted(counts) == sorted(CHECK_KEYS), counts
        # rf-branch is excluded here: on the current code the live VM response
        # shape fails the guard (see the dedicated regression pin below) — this
        # test pins the state mechanics of every other check.
        assert all(
            key == "fail:rf-branch" or value == 0 for key, value in counts.items()
        ), counts
        assert telegram_messages(tmp_path) == []


class TestAntiFlap:
    def test_alert_exactly_on_second_consecutive_failure_then_resolve(
        self, tmp_path: Path
    ) -> None:
        broken = broken_world_cname()
        run_cycle(tmp_path, broken)  # first failure: silent
        assert telegram_messages(tmp_path) == []
        run_cycle(tmp_path, broken)  # second failure: exactly one ALERT
        messages = telegram_messages(tmp_path, check="dns-world")
        assert len(messages) == 1, messages
        assert "ALERT [origa-poller]" in messages[0]
        assert "dns-world" in messages[0]
        run_cycle(tmp_path)  # recovery: exactly one RESOLVE
        messages = telegram_messages(tmp_path, check="dns-world")
        assert len(messages) == 2, messages
        assert "RESOLVE [origa-poller]" in messages[1]
        assert "dns-world" in messages[1]
        counts = state_counts(tmp_path)
        # rf-branch is the guarded regression pin (see TestHttpProbes) and
        # stays red until the guard accepts the live VM shape.
        assert all(
            key == "fail:rf-branch" or value == 0 for key, value in counts.items()
        ), counts

    def test_no_alert_storm_after_the_second_failure(self, tmp_path: Path) -> None:
        broken = broken_world_cname()
        for _ in range(5):
            run_cycle(tmp_path, broken)
        alerts = [
            m for m in telegram_messages(tmp_path, check="dns-world") if "ALERT" in m
        ]
        assert len(alerts) == 1, alerts

    def test_state_file_keeps_exactly_one_line_per_key(self, tmp_path: Path) -> None:
        run_cycle(tmp_path)
        broken = broken_world_cname()
        run_cycle(tmp_path, broken)
        run_cycle(tmp_path, broken)
        run_cycle(tmp_path)  # recovery exercises in-place line replacement
        raw_lines = (tmp_path / "state.txt").read_text().splitlines()
        keys = [line.split()[0] for line in raw_lines]
        assert len(keys) == len(set(keys)), f"duplicated state keys: {keys}"
        assert sorted(keys) == sorted(CHECK_KEYS)


class TestSeveritySemantics:
    def test_dead_ru_resolvers_warn_not_incident(self, tmp_path: Path) -> None:
        # The liveness probe keys off dig's exit code, so a dead resolver is
        # simulated with the stub's __FAIL__ marker, not an empty answer.
        dead = {
            _env_key("FAKE_DIG", "uwuwu.net", RU_PRIMARY, "ANY"): "__FAIL__",
            _env_key("FAKE_DIG", "uwuwu.net", RU_SECONDARY, "ANY"): "__FAIL__",
        }
        run_cycle(tmp_path, dead)
        assert telegram_messages(tmp_path, check="ru-resolvers") == []
        run_cycle(tmp_path, dead)
        messages = telegram_messages(tmp_path, check="ru-resolvers")
        assert len(messages) == 1, messages
        assert "WARN [origa-poller] ru-resolvers" in messages[0]

    def test_umami_cert_warns_below_horizon(self, tmp_path: Path) -> None:
        expiring = {"FAKE_UMAMI_NOTAFTER": "Nov 20 00:00:00 2026 GMT"}
        run_cycle(tmp_path, expiring)
        run_cycle(tmp_path, expiring)
        messages = telegram_messages(tmp_path, check="umami-cert")
        assert len(messages) == 1, messages
        assert "WARN [origa-poller] umami-cert" in messages[0]
        assert "expires in" in messages[0]


class TestHttpProbes:
    def test_railway_4xx_is_a_failure(self, tmp_path: Path) -> None:
        slapped = {"FAKE_RAILWAY_origa_uwuwu_net": "404"}
        run_cycle(tmp_path, slapped)
        run_cycle(tmp_path, slapped)
        messages = telegram_messages(tmp_path, check="rf-railway-origa.uwuwu.net")
        assert len(messages) == 1, messages
        assert "ALERT [origa-poller] rf-railway-origa.uwuwu.net" in messages[0]

    def test_rf_branch_guard_accepts_live_vm_response_shape(
        self, tmp_path: Path
    ) -> None:
        # Regression pin (live-testing bug, 2026-11-08): the VM's nginx proxies
        # Tigris and forwards x-tigris-served-from; the healthy RF branch DOES
        # carry that header. It must not be treated as a world leak.
        run_cycle(tmp_path, {"FAKE_RF_BRANCH_RESPONSE": LIVE_VM_RESPONSE})
        counts = state_counts(tmp_path)
        assert counts["fail:rf-branch"] == 0, (
            "the live VM shape must pass the guard: "
            "x-tigris-served-from is forwarded by the VM's nginx"
        )
        assert telegram_messages(tmp_path, check="rf-branch") == []

    def test_rf_branch_guard_catches_real_world_leak(self, tmp_path: Path) -> None:
        # A genuine misroute serves the world shape: nginx without a version
        # slash, no VM signature.
        leaked = {"FAKE_RF_BRANCH_RESPONSE": WORLD_RESPONSE}
        run_cycle(tmp_path, leaked)
        run_cycle(tmp_path, leaked)
        messages = telegram_messages(tmp_path, check="rf-branch")
        assert len(messages) == 1, messages
        assert "ALERT [origa-poller] rf-branch" in messages[0]


class TestCycleResilience:
    def test_first_cycle_without_state_file_succeeds(self, tmp_path: Path) -> None:
        # Regression pin: the first cycle after install has no state file; the
        # substitution pipelines must not die on pipefail.
        (tmp_path / "state.txt").unlink(missing_ok=True)
        proc = run_cycle(tmp_path)
        assert proc.returncode == 0, proc.stderr
        assert "cycle end" in proc.stdout

    def test_failed_heartbeat_ping_does_not_kill_the_cycle(
        self, tmp_path: Path
    ) -> None:
        proc = run_cycle(tmp_path, {"FAKE_HEARTBEAT_FAIL": "1"})
        assert proc.returncode == 0, proc.stderr
        assert "cycle end" in proc.stdout
        assert "WARN heartbeat ping failed" in proc.stdout
