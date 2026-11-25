"""CI gate for the make-guard decision table (uwuwu#1472).

The authoritative table lives in scripts/dev-guards.test.sh — plain sh
asserts over the pure verdict functions of scripts/dev-guards.sh, runnable
anywhere without python. This wrapper executes it inside the pytest job
(ci.yml python-tests, scripts/** component) so a guard-logic change cannot
merge without its regression running. One table, two carriers: the sh
script stays the source of truth, this file only runs it.
"""

import re
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]


def test_dev_guards_decision_table() -> None:
    result = subprocess.run(
        ["sh", "scripts/dev-guards.test.sh"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        timeout=120,
    )
    assert result.returncode == 0, (
        f"guards table failed:\n{result.stdout}\n{result.stderr}"
    )
    match = re.search(r"(\d+) passed, (\d+) failed", result.stdout)
    assert match, f"summary line not found in:\n{result.stdout}"
    passed, failed = int(match.group(1)), int(match.group(2))
    assert failed == 0, f"{failed} guard-table cases failed"
    assert passed >= 40, (
        f"decision table shrank to {passed} cases — the regression lost coverage"
    )
