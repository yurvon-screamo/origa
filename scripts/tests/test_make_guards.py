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
import shutil
import socket
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = REPO_ROOT / "Makefile"
GUARDS = REPO_ROOT / "scripts" / "dev-guards.sh"


def _clean_env(tmp_path: Path, **overrides: str) -> tuple[dict[str, str], Path]:
    """Minimal env (no inherited ORIGA_*/TRAILBASE_URL) + stubbed delegates.

    Returns (env, marker_path). If a guard wrongly lets a target through, the
    stub touches marker_path — the test fails even though no real server runs.
    """
    stub_bin = tmp_path / "stub-bin"
    stub_bin.mkdir(exist_ok=True)
    marker = tmp_path / "delegate-started"
    for name in ("trunk", "npm", "just"):
        stub = stub_bin / name
        stub.write_text(f"#!/bin/sh\ntouch {marker}\nexit 0\n")
        stub.chmod(0o755)
    env = {
        "PATH": f"{stub_bin}{os.pathsep}{os.environ.get('PATH', '/usr/bin:/bin')}",
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
    shutil.copy(GUARDS, fixture / "scripts" / "dev-guards.sh")
    (fixture / "origa_ui" / "Trunk.toml").write_text(f"[serve]\nport = {serve_port}\n")
    return fixture


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
        assert "pid=" in result.stderr and "cwd=" in result.stderr
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
