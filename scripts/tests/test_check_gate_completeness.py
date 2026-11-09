"""Unit tests for the CI Gate completeness guard (task 20261104060555).

The script is the mechanical enforcement of the ci.yml doctrine invariant
"no job outside the gate": every workflow job must appear in ``check.needs``.
A job missing from the needs list runs outside the single required status
check, so its red result would never block a merge — the exact hole class
this repo hit before the guard landed (qlty findings outside the gate).

Tests run the script as a subprocess (the way the CI job invokes it:
``python3 .github/scripts/check_gate_completeness.py`` from the repo root)
against synthetic workflows; the final pin runs it against the real repo
workflow, so a drift of ci.yml itself turns pytest red before CI sees it.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = _REPO_ROOT / ".github" / "scripts" / "check_gate_completeness.py"

# A minimal legal workflow: both worker jobs are wired into check.needs.
WELL_WIRED = """\
name: CI
jobs:
    changes:
        steps: []
    lint:
        needs: [changes]
        steps: []
    check:
        needs: [changes, lint]
        steps: []
"""


def run_guard(cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(SCRIPT)],
        cwd=cwd,
        capture_output=True,
        text=True,
        timeout=60,
    )


def write_workflow(root: Path, body: str) -> Path:
    workflow_dir = root / ".github" / "workflows"
    workflow_dir.mkdir(parents=True)
    workflow_file = workflow_dir / "ci.yml"
    workflow_file.write_text(body)
    return workflow_file


def test_all_jobs_wired_passes(tmp_path):
    write_workflow(tmp_path, WELL_WIRED)

    result = run_guard(tmp_path)

    assert result.returncode == 0, result.stdout + result.stderr
    assert "All 3 jobs are wired into the CI Gate (check.needs)" in result.stdout


def test_rogue_job_fails_with_annotation(tmp_path):
    # A rogue job appended after the gate keeps the file valid YAML but
    # must be reported — this is the class the guard exists for. Wired
    # jobs (lint) must not appear in the error.
    body = WELL_WIRED + "    rogue-job:\n        steps: []\n"
    write_workflow(tmp_path, body)

    result = run_guard(tmp_path)

    assert result.returncode == 1, result.stdout + result.stderr
    annotation = result.stdout.split("::error::")[1]
    assert "rogue-job" in annotation
    assert "lint" not in annotation


def test_gate_job_itself_needs_no_self_reference(tmp_path):
    # check is the gate: it is never listed in its own needs. The guard
    # must not report the gate job as unwired ( WELL_WIRED covers the
    # positive path; this variant drops every other job).
    write_workflow(
        tmp_path,
        """\
name: CI
jobs:
    check:
        needs: []
        steps: []
""",
    )

    result = run_guard(tmp_path)

    assert result.returncode == 0, result.stdout + result.stderr
    assert "All 1 jobs are wired into the CI Gate (check.needs)" in result.stdout


def test_missing_gate_job_fails_loudly(tmp_path):
    # Fail-closed: without a check job the guard must crash, not report
    # a green "everything is wired" — an absent gate means nothing is
    # gated.
    write_workflow(
        tmp_path,
        """\
name: CI
jobs:
    lint:
        steps: []
""",
    )

    result = run_guard(tmp_path)

    assert result.returncode != 0


def test_real_repo_workflow_is_wired():
    # Pin against the live workflow: adding a job to ci.yml without
    # wiring it into check.needs must turn pytest red before CI runs.
    result = run_guard(_REPO_ROOT)

    assert result.returncode == 0, result.stdout + result.stderr
    assert "wired into the CI Gate" in result.stdout
