"""Unit tests for the platform cfg guard (task 20261104060604, GH #854).

The guard is warning-only by design: a missed warning never turns CI red,
so a silent regression in its detection logic would ship unnoticed (the
#701 class this guard exists for). These tests pin the detection contract:

- added `cfg(` lines with a platform token are counted, in every wrapper
  shape (`any`/`all`/`not`, nesting, short `windows`/`unix` forms);
- `cfg!()` macros, non-platform cfgs (`feature = ...`) and lookalikes
  (`some_cfg(`) are not;
- the test-coverage exemption (`#[cfg(test)]`, `mod tests`, ...) silences
  the warning;
- git failure fails loudly (exit 1) instead of silently passing.

Subprocess cases run the script the way the Lint job invokes it
(`python3 .github/scripts/platform_cfg_guard.py`) against a synthetic git
repo; the final pin runs it against the real repo checkout.
"""

from __future__ import annotations

import importlib.util
import subprocess
import sys
from pathlib import Path

import pytest

_REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = _REPO_ROOT / ".github" / "scripts" / "platform_cfg_guard.py"

spec = importlib.util.spec_from_file_location("platform_cfg_guard", SCRIPT)
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)


def diff_of(*lines: str) -> list[str]:
    """A synthetic `git diff --unified=0` payload in the shape the guard parses."""
    payload = [
        "diff --git a/tauri/src/demo.rs b/tauri/src/demo.rs",
        "+++ b/tauri/src/demo.rs",
    ]
    payload.extend(lines)
    return payload


# --- unit: line classification ------------------------------------------------


@pytest.mark.parametrize(
    ("line", "counted"),
    [
        ('#[cfg(target_os = "linux")]', True),
        ('#[cfg(any(target_os = "macos", target_os = "ios"))]', True),
        ("#[cfg(all(unix, not(mobile)))]", True),
        ('#[cfg(not(target_os = "android"))]', True),
        ("#[cfg(windows)]", True),
        ("#[cfg(desktop)]", True),
        ('#[cfg(any(windows, target_os = "linux"))]', True),
        ('#[cfg(feature = "custom-protocol")]', False),
        ('if cfg!(target_os = "macos")', False),
        ("let x = some_cfg(y);", False),
        ("let y = mycfg(windows);", False),
        ('let plain = "cfg(target_os = \\"linux\\") in a string";', True),
    ],
)
def test_added_line_classification(line: str, counted: bool) -> None:
    counts = guard.added_cfg_paths(diff_of(f"+{line}"))
    assert (counts.get("tauri/src/demo.rs", 0) > 0) is counted


def test_removed_and_context_lines_are_ignored() -> None:
    counts = guard.added_cfg_paths(
        diff_of(
            '-#[cfg(target_os = "linux")]',
            ' #[cfg(target_os = "macos")]',
        )
    )
    assert counts == {}


def test_files_are_counted_separately() -> None:
    lines = [
        "+++ b/tauri/src/a.rs",
        "+#[cfg(windows)]",
        "+++ b/tauri/src/b.rs",
        "+#[cfg(unix)]",
    ]
    counts = guard.added_cfg_paths(lines)
    assert counts == {"tauri/src/a.rs": 1, "tauri/src/b.rs": 1}


# --- unit: coverage exemption -------------------------------------------------


@pytest.mark.parametrize(
    "marker", ["#[cfg(test)]", "#[rstest]", "#[test]", "mod tests", "wasm_bindgen_test"]
)
def test_coverage_markers_silence_the_warning(tmp_path: Path, marker: str) -> None:
    path = tmp_path / "demo.rs"
    path.write_text(f"{marker}\nmod something {{}}\n", encoding="utf-8")
    assert guard.has_test_coverage(str(path)) is True


def test_file_without_tests_has_no_coverage(tmp_path: Path) -> None:
    path = tmp_path / "demo.rs"
    path.write_text("pub const X: u8 = 1;\n", encoding="utf-8")
    assert guard.has_test_coverage(str(path)) is False


def test_missing_file_is_not_a_crash(tmp_path: Path) -> None:
    assert guard.has_test_coverage(str(tmp_path / "gone.rs")) is False


# --- subprocess: the way CI invokes it ----------------------------------------


def _git(repo: Path, *args: str) -> None:
    subprocess.run(["git", *args], cwd=repo, check=True, capture_output=True)


@pytest.fixture()
def demo_repo(tmp_path: Path) -> Path:
    """A synthetic git repo whose HEAD adds one bare platform cfg to tauri/src."""
    repo = tmp_path / "repo"
    repo.mkdir()
    _git(repo, "init", "-q", "-b", "main")
    _git(repo, "config", "user.email", "test@example.com")
    _git(repo, "config", "user.name", "test")
    (repo / "tauri" / "src").mkdir(parents=True)
    (repo / "tauri" / "src" / "base.rs").write_text("// base\n", encoding="utf-8")
    _git(repo, "add", "-A")
    _git(repo, "commit", "-qm", "base")
    base_sha = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repo,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    (repo / "tauri" / "src" / "demo.rs").write_text(
        '#[cfg(any(target_os = "macos", target_os = "windows"))]\npub const DEMO: u8 = 1;\n',
        encoding="utf-8",
    )
    _git(repo, "add", "-A")
    _git(repo, "commit", "-qm", "demo")
    (repo / ".base").write_text(base_sha, encoding="utf-8")
    return repo


def _run_guard(repo: Path) -> subprocess.CompletedProcess[str]:
    env_base = (repo / ".base").read_text(encoding="utf-8")
    return subprocess.run(
        [sys.executable, str(SCRIPT)],
        cwd=repo,
        capture_output=True,
        text=True,
        env={"PLATFORM_GUARD_BASE": env_base, "PATH": "/usr/bin:/bin:/usr/local/bin"},
        check=False,
    )


def test_bare_cfg_produces_warning_and_never_blocks(demo_repo: Path) -> None:
    result = _run_guard(demo_repo)
    assert result.returncode == 0, result.stderr
    assert "::warning file=tauri/src/demo.rs::" in result.stdout


def test_cfg_with_test_module_is_silent(demo_repo: Path) -> None:
    (demo_repo / "tauri" / "src" / "demo.rs").write_text(
        '#[cfg(any(target_os = "macos", target_os = "windows"))]\n'
        "pub const DEMO: u8 = 1;\n"
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn pin() {\n        assert_eq!(super::DEMO, 1);\n    }\n}\n",
        encoding="utf-8",
    )
    _git(demo_repo, "add", "-A")
    _git(demo_repo, "commit", "-qm", "demo + tests")
    result = _run_guard(demo_repo)
    assert result.returncode == 0, result.stderr
    assert "::warning" not in result.stdout
    assert "test coverage present" in result.stdout


def test_git_failure_fails_loudly(demo_repo: Path) -> None:
    result = subprocess.run(
        [sys.executable, str(SCRIPT)],
        cwd=demo_repo,
        capture_output=True,
        text=True,
        env={
            "PLATFORM_GUARD_BASE": "definitely-not-a-ref",
            "PATH": "/usr/bin:/bin:/usr/local/bin",
        },
        check=False,
    )
    assert result.returncode == 1
    assert "::error::" in result.stdout


def test_real_repo_checkout_never_blocks() -> None:
    """The guard exits 0 over the real checkout: every cfg-bearing file in
    the current diff carries test coverage (or the diff is empty)."""
    result = subprocess.run(
        [sys.executable, str(SCRIPT)],
        cwd=_REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
