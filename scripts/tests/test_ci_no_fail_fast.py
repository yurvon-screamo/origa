"""Regression pins for the --no-fail-fast contract of the ci.yml test jobs
(task 20261110182948, PR #933).

Both cargo test commands in CI — the workspace suite (job ``test``) and the
landing SSR suite (job ``landing-test``) — must run with ``--no-fail-fast``:
a red suite must not hide the rest of the workspace behind it, so one CI
round shows every regression instead of one per round. Before the change a
red workspace suite aborted the run at the first failing binary; the landing
suite lives in a separate job since #800, and the workspace command must
keep excluding ``origa_landing`` — a dropped exclude would run the landing
tests twice.

The flag is behavior of the CI runner, so the workflow text is the only
executable statement the repo makes about it. These pins read ci.yml and
assert:

- the two suite commands keep ``--no-fail-fast`` and their component flags
  (``--exclude origa_landing`` / ``-p origa_landing --features ssr``);
- no other ``cargo test`` invocation in ci.yml quietly reintroduces the
  fail-fast class: every cargo test step either carries the flag or is on
  the explicit allowlist (``macos-check`` — a single-package smoke with a
  single test binary, where fail-fast cannot hide a second suite).

The workflow rides the ``ci`` path component, so editing ci.yml flips
``run_python_tests`` and this file runs on the same PR that touched the
workflow — the guard cannot drift silently. (Compile errors abort a
``--no-fail-fast`` run regardless — cargo semantics, not workflow text —
so there is nothing repo-side to pin for that half of the contract.)

Pure file reads, no network.
"""

from __future__ import annotations

from pathlib import Path

import pytest
import yaml

_REPO_ROOT = Path(__file__).resolve().parents[2]
CI_FILE = _REPO_ROOT / ".github" / "workflows" / "ci.yml"

# Single-package, single-binary invocations where fail-fast cannot hide a
# second suite. Anything not on this list must carry --no-fail-fast.
FAIL_FAST_ALLOWLIST: frozenset[str] = frozenset({"macos-check"})

WORKSPACE_SUITE_FLAGS = ("cargo test --workspace --exclude origa_landing", "--no-fail-fast")
LANDING_SUITE_FLAGS = ("cargo test -p origa_landing --features ssr", "--no-fail-fast")


def _workflow() -> dict:
    return yaml.safe_load(CI_FILE.read_text(encoding="utf-8"))


def _steps_of(job_name: str) -> list[dict]:
    jobs = _workflow()["jobs"]
    assert job_name in jobs, f"job {job_name!r} disappeared from ci.yml"
    steps = jobs[job_name].get("steps") or []
    assert steps, f"job {job_name!r} lost its steps"
    return steps


def _step(job_name: str, step_name: str) -> dict:
    matches = [s for s in _steps_of(job_name) if s.get("name") == step_name]
    assert len(matches) == 1, (
        f"expected exactly one step named {step_name!r} in job {job_name!r}, "
        f"found {len(matches)}"
    )
    return matches[0]


def _run_line(job_name: str, step_name: str) -> str:
    run = _step(job_name, step_name).get("run") or ""
    assert run, f"step {step_name!r} in job {job_name!r} lost its run command"
    return run


def _cargo_test_run_lines() -> list[tuple[str, str, str]]:
    """(job, step name, run line) for every ci.yml step invoking cargo test."""
    found: list[tuple[str, str, str]] = []
    for job_name, job in _workflow()["jobs"].items():
        for step in job.get("steps") or []:
            run = step.get("run") or ""
            if "cargo test" in run:
                found.append((job_name, step.get("name", "<unnamed>"), run))
    assert found, "no cargo test invocation left in ci.yml — suites moved?"
    return found


def test_workspace_suite_runs_no_fail_fast():
    """The workspace suite keeps the flag and the landing exclude."""
    run = _run_line("test", "Run tests")
    for flag in WORKSPACE_SUITE_FLAGS:
        assert flag in run, (
            f"the workspace suite command lost {flag!r}: without --no-fail-fast a "
            "red suite hides the remaining ones; without --exclude origa_landing "
            "the landing suites run twice"
        )


def test_landing_ssr_suite_runs_no_fail_fast():
    """The landing SSR suite keeps the flag."""
    run = _run_line("landing-test", "Run landing SSR tests")
    for flag in LANDING_SUITE_FLAGS:
        assert flag in run, (
            f"the landing SSR suite command lost {flag!r}: a red suite would "
            "again hide the remaining ones behind it"
        )


def test_every_cargo_test_step_has_no_fail_fast_or_is_allowlisted():
    """The fail-fast class cannot quietly return for a new suite.

    A new multi-binary cargo test step without the flag recreates the
    "one regression per CI round" behavior; anything not on the allowlist
    must carry the flag, and an allowlist entry that outgrows its
    single-binary justification must be consciously revised.
    """
    offenders = [
        (job, step)
        for job, step, run in _cargo_test_run_lines()
        if job not in FAIL_FAST_ALLOWLIST and "--no-fail-fast" not in run
    ]
    assert offenders == [], (
        "cargo test steps without --no-fail-fast "
        f"(jobs {sorted({job for job, _ in offenders})}): add the flag or a "
        "documented FAIL_FAST_ALLOWLIST entry for a genuinely single-binary suite"
    )


# --- Parser units (synthetic inputs, fail-loud contract) -------------------


def test_step_missing_fails_loud():
    with pytest.raises(AssertionError, match="exactly one step named"):
        _step("test", "Nonexistent Step")


def test_job_missing_fails_loud():
    with pytest.raises(AssertionError, match="disappeared from ci.yml"):
        _steps_of("nonexistent-job")


def test_step_run_missing_fails_loud():
    with pytest.raises(AssertionError, match="lost its run command"):
        _run_line("test", "Checkout repository")


def test_cargo_test_lines_exist():
    """Sanity: the pins read a ci.yml that still contains the suites."""
    jobs = {job for job, _, _ in _cargo_test_run_lines()}
    assert {"test", "landing-test"} <= jobs, (
        "the two suite jobs of PR #933 are missing from ci.yml"
    )
