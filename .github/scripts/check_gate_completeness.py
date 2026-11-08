#!/usr/bin/env python3
"""Fail loudly when a workflow job is not wired into the CI Gate.

The `check` job is the single required status check (branch ruleset).
A job missing from `check.needs` runs OUTSIDE the gate: its red result
would not block the merge — the exact class of hole this repo hit with
qlty checks (task 20261104060555, 2026-11). Static parsing of the
workflow source catches the failure mode at the moment it is
introduced: someone adds a job but forgets the needs list.

Usage: python3 check_gate_completeness.py  (repo root, in CI or locally)
"""

import sys
from pathlib import Path

import yaml

WORKFLOW = Path(".github/workflows/ci.yml")
GATE_JOB = "check"


def main() -> int:
    workflow = yaml.safe_load(WORKFLOW.read_text())
    jobs = set(workflow.get("jobs", {}))
    gate_needs = set(workflow["jobs"][GATE_JOB].get("needs", []))
    unwired = sorted(jobs - gate_needs - {GATE_JOB})
    if unwired:
        print(
            "::error::Jobs not wired into the CI Gate ("
            + GATE_JOB
            + ".needs): "
            + ", ".join(unwired)
        )
        return 1
    print(f"All {len(jobs)} jobs are wired into the CI Gate ({GATE_JOB}.needs)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
