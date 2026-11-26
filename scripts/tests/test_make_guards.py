"""Acceptance/integration gates for the root Makefile's fail-closed guards (uwuwu#1472).

The authoritative *decision table* (pure verdicts) lives in
``scripts/dev-guards.test.sh`` and is run in CI by ``test_dev_guards.py``.
This module covers the layer underneath and above it:

* acceptance — the real ``make`` targets refuse before the delegate starts
  (the incident's invariant: no trunk/npm process on a bad environment);
* integration — the OS/filesystem probes (ss + /proc listener attribution,
  creds and node_modules in the harness dir) wired through the real Makefile.

The delegate binaries are stubbed with markers so a regression that lets a
guard pass cannot silently launch a server in CI. No production code is
imported or edited — these tests only invoke the shipped files.
"""

from __future__ import annotations

import os
import re
import shutil
import socket
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"
GUARDS = REPO_ROOT / "scripts" / "dev-guards.sh"


def _clean_env(
    tmp_path: Path,
    *,
    minimal_path: bool = False,
    stubs: tuple[str, ...] = ("trunk", "npm", "just", "cargo", "cargo-tauri"),
    **overrides: str,
) -> tuple[dict[str, str], Path]:
    """Minimal env (no inherited ORIGA_*/TRAILBASE_URL) + stubbed delegates.

    Returns (env, marker_path). If a guard wrongly lets a target through, the
    stub touches marker_path — the test fails even though no real server runs.
    The cargo stub records the delegated TRAILBASE_URL for injected-value
    assertions. ``minimal_path`` hides the real cargo-tauri/trunk installs so
    "tool missing" cases are reproducible in CI.
    """
    stub_bin = tmp_path / "stub-bin"
    stub_bin.mkdir(exist_ok=True)
    marker = tmp_path / "delegate-started"
    env_file = tmp_path / "delegate-trailbase"
    for name in stubs:
        stub = stub_bin / name
        stub.write_text(
            "#!/bin/sh\n"
            f"touch {marker}\n"
            f"printf '%s' \"${{TRAILBASE_URL-}}\" > {env_file}\n"
            "exit 0\n"
        )
        stub.chmod(0o755)
    base_path = (
        "/usr/bin:/bin" if minimal_path else os.environ.get("PATH", "/usr/bin:/bin")
    )
    env = {
        "PATH": f"{stub_bin}{os.pathsep}{base_path}",
        "HOME": str(tmp_path),
    }
    env.update(overrides)
    return env, marker


def _make(
    target: str, env: dict[str, str], cwd: Path
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["make", "-f", str(MAKEFILE), target],
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
        timeout=30,
    )


# --- acceptance: the real make target refuses before the delegate starts ---


def test_serve_ui_without_env_refuses_before_trunk(tmp_path: Path) -> None:
    env, marker = _clean_env(tmp_path)
    result = _make("serve-ui", env, REPO_ROOT)
    assert result.returncode != 0
    assert "GUARD FAIL: TRAILBASE_URL" in result.stderr
    assert "export TRAILBASE_URL=http://127.0.0.1:4000" in result.stderr
    assert not marker.exists(), (
        "guard passed — trunk was launched on a bare environment"
    )


@pytest.mark.parametrize(
    "trailbase_url",
    [
        "https://app.origa.uwuwu.net",  # explicit production — the incident value
        "http://127.0.0.1:1@app.origa.uwuwu.net",  # userinfo hides the real host
        "localhost:4000",  # schemeless typo breaks base+path concatenation
        "",  # ADR-020: empty == unset
    ],
)
def test_serve_ui_refuses_bad_trailbase(tmp_path: Path, trailbase_url: str) -> None:
    env, marker = _clean_env(
        tmp_path,
        TRAILBASE_URL=trailbase_url,
        ORIGA_CDN_BASE_URL="https://s3.origa.uwuwu.net",
    )
    result = _make("serve-ui", env, REPO_ROOT)
    assert result.returncode != 0
    assert "GUARD FAIL: TRAILBASE_URL" in result.stderr
    assert not marker.exists()


def test_env_check_reports_holes_and_fails(tmp_path: Path) -> None:
    env, _ = _clean_env(tmp_path)
    result = _make("env-check", env, REPO_ROOT)
    assert result.returncode != 0
    for hole in ("ORIGA_CDN_BASE_URL", "ORIGA_APP_BASE_URL", "TRAILBASE_URL"):
        assert f"HOLE {hole}" in result.stdout
    assert "дыр окружения" in result.stdout
    assert "Tests in a Fresh Worktree" in result.stdout  # remedy points to the runbook


# --- integration: OS/filesystem probes through the real Makefile ---


def _fixture_repo(tmp_path: Path, serve_port: int) -> Path:
    """Minimal checkout skeleton: real guard library + Makefile, empty harness dir."""
    fixture = tmp_path / "fixture"
    (fixture / "scripts").mkdir(parents=True)
    (fixture / "origa_ui").mkdir()
    (fixture / "end2end").mkdir()
    (fixture / "tauri").mkdir()
    shutil.copy(GUARDS, fixture / "scripts" / "dev-guards.sh")
    (fixture / "origa_ui" / "Trunk.toml").write_text(f"[serve]\nport = {serve_port}\n")
    return fixture


def test_serve_ui_starts_delegate_with_valid_env(tmp_path: Path) -> None:
    """Positive acceptance path: valid env -> guards pass -> trunk launches.

    The port is pinned through the fixture's Trunk.toml and found free by
    binding port 0; the delegate is a marker stub, so CI never starts a real
    server. Without this case all refusal tests stay green even if the recipe
    loses its delegate command (test-review round, common finding).
    """
    probe = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    probe.bind(("127.0.0.1", 0))
    port = probe.getsockname()[1]
    probe.close()

    fixture = _fixture_repo(tmp_path, port)
    env, marker = _clean_env(
        tmp_path,
        TRAILBASE_URL="http://127.0.0.1:4000",
        ORIGA_CDN_BASE_URL="https://s3.origa.uwuwu.net",
    )
    result = _make("serve-ui", env, fixture)
    assert result.returncode == 0, result.stderr
    assert marker.exists(), "guards passed but the delegate did not start"


def test_serve_ui_refuses_any_listener_with_attribution(tmp_path: Path) -> None:
    """serve-ui's port is pinned by Trunk.toml; ANY listener must be refused.

    Covers the ss -> /proc/cwd probe that the pure decision table cannot reach.
    """
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(("127.0.0.1", 0))
    sock.listen(1)
    port = sock.getsockname()[1]
    try:
        fixture = _fixture_repo(tmp_path, port)
        env, marker = _clean_env(
            tmp_path,
            TRAILBASE_URL="http://127.0.0.1:4000",
            ORIGA_CDN_BASE_URL="https://s3.origa.uwuwu.net",
        )
        result = _make("serve-ui", env, fixture)
        assert result.returncode != 0
        assert f"порт serve-ui {port} занят" in result.stderr
        # The listener is this very process. A failed readlink prints
        # "cwd=?" — a bare "cwd=" substring would accept broken attribution.
        match = re.search(r"pid=(\d+), cwd=([^)]+)\)", result.stderr)
        assert match, result.stderr
        assert match.group(2) == str(Path.cwd()), result.stderr
        assert not marker.exists()
    finally:
        sock.close()


def test_e2e_refuses_missing_creds_and_missing_node_modules(tmp_path: Path) -> None:
    """Harness artifacts are read from the filesystem — no .env / no modules."""
    fixture = _fixture_repo(tmp_path, 1420)
    env, marker = _clean_env(tmp_path)

    no_creds = _make("e2e", env, fixture)
    assert no_creds.returncode != 0
    assert "креды фикстуры" in no_creds.stderr
    assert "cp end2end/.env.example" in no_creds.stderr

    # Non-empty dummy value built at runtime: the guard only checks presence,
    # and a literal here trips the repo's hardcoded-secret scanner.
    env["ORIGA_ADMIN_PASSWORD"] = chr(120)
    no_modules = _make("e2e", env, fixture)
    assert no_modules.returncode != 0
    assert "node_modules отсутствует" in no_modules.stderr
    assert "cd end2end && npm ci" in no_modules.stderr
    assert not marker.exists()


# --- uwuwu#1663: dist and desktop recipes ---


def _free_port() -> int:
    probe = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    probe.bind(("127.0.0.1", 0))
    port = probe.getsockname()[1]
    probe.close()
    return port


def test_dist_without_env_refuses_before_trunk(tmp_path: Path) -> None:
    env, marker = _clean_env(tmp_path)
    result = _make("dist", env, REPO_ROOT)
    assert result.returncode != 0
    assert "GUARD FAIL: TRAILBASE_URL" in result.stderr
    # Gherkin pins the whole runbook canon, not just the target name: a
    # truncated hint would leave the developer without the complete pair.
    assert (
        "ORIGA_CDN_BASE_URL=http://localhost:8080 "
        "TRAILBASE_URL=http://127.0.0.1:4000 make dist" in result.stderr
    )
    assert not marker.exists()


def test_dist_with_empty_trailbase_refuses_before_trunk(tmp_path: Path) -> None:
    """ADR-024 at the make boundary: SET-EMPTY is the same refusal as unset."""
    env, marker = _clean_env(
        tmp_path,
        ORIGA_CDN_BASE_URL="http://localhost:8080",
        TRAILBASE_URL="",
    )
    result = _make("dist", env, REPO_ROOT)
    assert result.returncode != 0
    assert "GUARD FAIL: TRAILBASE_URL" in result.stderr
    assert not marker.exists()


def test_dist_with_explicit_prod_refuses_before_trunk(tmp_path: Path) -> None:
    env, marker = _clean_env(
        tmp_path,
        TRAILBASE_URL="https://app.origa.uwuwu.net",
        ORIGA_CDN_BASE_URL="http://localhost:8080",
    )
    result = _make("dist", env, REPO_ROOT)
    assert result.returncode != 0
    assert "GUARD FAIL: TRAILBASE_URL" in result.stderr
    assert not marker.exists()


def test_dist_starts_delegate_with_local_pair(tmp_path: Path) -> None:
    env, marker = _clean_env(
        tmp_path,
        TRAILBASE_URL="http://127.0.0.1:4000",
        ORIGA_CDN_BASE_URL="http://localhost:8080",
    )
    result = _make("dist", env, REPO_ROOT)
    assert result.returncode == 0, result.stderr
    assert marker.exists(), "guards passed but trunk build did not start"


def test_tauri_dev_injects_loopback_into_delegate(tmp_path: Path) -> None:
    fixture = _fixture_repo(tmp_path, _free_port())
    env, marker = _clean_env(tmp_path, GUARD_REPO_ROOT=str(fixture))
    result = _make("tauri-dev", env, fixture)
    assert result.returncode == 0, result.stderr
    assert marker.exists()
    delegated = tmp_path / "delegate-trailbase"
    assert delegated.read_text().strip() == "http://127.0.0.1:4000"


def test_tauri_dev_with_empty_trailbase_injects_loopback(tmp_path: Path) -> None:
    """Gherkin: SET-EMPTY is unset at the make boundary too (ADR-024)."""
    fixture = _fixture_repo(tmp_path, _free_port())
    env, marker = _clean_env(
        tmp_path, GUARD_REPO_ROOT=str(fixture), TRAILBASE_URL=""
    )
    result = _make("tauri-dev", env, fixture)
    assert result.returncode == 0, result.stderr
    assert marker.exists()
    assert (tmp_path / "delegate-trailbase").read_text().strip() == "http://127.0.0.1:4000"


def test_tauri_dev_preserves_explicit_loopback(tmp_path: Path) -> None:
    """An explicit loopback backend must survive the recipe unchanged."""
    fixture = _fixture_repo(tmp_path, _free_port())
    env, marker = _clean_env(
        tmp_path, GUARD_REPO_ROOT=str(fixture), TRAILBASE_URL="http://localhost:4001"
    )
    result = _make("tauri-dev", env, fixture)
    assert result.returncode == 0, result.stderr
    assert marker.exists()
    assert (tmp_path / "delegate-trailbase").read_text().strip() == "http://localhost:4001"


def test_tauri_dev_refuses_explicit_prod_before_delegate(tmp_path: Path) -> None:
    env, marker = _clean_env(tmp_path, TRAILBASE_URL="https://app.origa.uwuwu.net")
    result = _make("tauri-dev", env, REPO_ROOT)
    assert result.returncode != 0
    assert "GUARD FAIL: TRAILBASE_URL" in result.stderr
    # Gherkin: the refusal carries the norm and the ready-to-copy loopback fix.
    assert "не должен уходить в прод" in result.stderr
    assert "export TRAILBASE_URL=http://127.0.0.1:4000" in result.stderr
    assert not marker.exists()


def test_tauri_dev_refuses_occupied_port_with_attribution(tmp_path: Path) -> None:
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.bind(("127.0.0.1", 0))
    sock.listen(1)
    port = sock.getsockname()[1]
    try:
        fixture = _fixture_repo(tmp_path, port)
        env, marker = _clean_env(tmp_path, GUARD_REPO_ROOT=str(fixture))
        result = _make("tauri-dev", env, fixture)
        assert result.returncode != 0
        # The label carries the recipe's parenthetical; the port and the
        # refusal verb are the load-bearing parts.
        assert f"порт serve-ui {port}" in result.stderr
        assert "занят (pid=" in result.stderr
        # Gherkin: pid+cwd attribution and a ready remedy, same as serve-ui.
        match = re.search(r"pid=(\d+), cwd=([^)]+)\)", result.stderr)
        assert match, result.stderr
        assert match.group(1) == str(os.getpid()), result.stderr
        assert match.group(2) == str(Path.cwd()), result.stderr
        assert f"остановите процесс (kill {os.getpid()})" in result.stderr
        assert not marker.exists()
    finally:
        sock.close()


def test_tauri_dev_refuses_missing_cli_with_controlled_path(tmp_path: Path) -> None:
    env, marker = _clean_env(
        tmp_path,
        minimal_path=True,
        stubs=("trunk", "npm", "just", "cargo"),
    )
    result = _make("tauri-dev", env, REPO_ROOT)
    assert result.returncode != 0
    assert "cargo-tauri не найден" in result.stderr
    assert not marker.exists()
