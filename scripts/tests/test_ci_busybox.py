"""Guards for the recurring BusyBox shell check in CI.

``busybox-tests`` runs the ``geoip.sh`` contract suite inside the Alpine image
of the production Dockerfile stage, so ``sh``, ``tail``, ``grep`` and ``stat``
are busybox applets rather than the GNU coreutils of the pytest runner. The
guard pins the contract that makes that check real:

- a shell edit (``wasm``) or a test edit (``python``) flips the run flag;
- the job reads the Alpine tag from the Dockerfile instead of keeping a
  second, drifting literal;
- the suite runs inside that container, through ``sh``;
- the job is wired into the fail-closed CI Gate with its own flag.

Pure file tests — no Docker, no network.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
CI_RELPATH = Path(".github/workflows/ci.yml")
GEOIP_TEST_RELPATH = Path("scripts/tests/test_geoip.py")
JOB_NAME = "busybox-tests"
RUN_FLAG = "run_busybox_tests"

FOLDING_MARKER = "            image=\"$(awk '/^FROM alpine:/"
PYTEST_MARKER = "python3 -m pytest -q -p no:cacheprovider scripts/tests/test_geoip.py"


def workflow_text() -> str:
    return (REPO_ROOT / CI_RELPATH).read_text(encoding="utf-8")


def changes_output_block(text: str, output_name: str) -> str:
    """The folded expression of one ``changes`` output, up to the next key."""
    matches = list(
        re.finditer(rf"^            {re.escape(output_name)}: >-$", text, re.M)
    )
    assert len(matches) == 1, (
        f"expected exactly one '{output_name}: >-' output in ci.yml, found {len(matches)}"
    )
    rest = text[matches[0].end() :]
    nxt = re.search(r"^            \S", rest, re.M)
    return rest[: nxt.start() if nxt else len(rest)]


def job_block(text: str, job_name: str) -> str:
    """One top-level job block, from its heading to the next job heading."""
    matches = list(re.finditer(rf"^    {re.escape(job_name)}:$", text, re.M))
    assert len(matches) == 1, (
        f"expected exactly one job {job_name!r} in ci.yml, found {len(matches)}"
    )
    rest = text[matches[0].end() :]
    nxt = re.search(r"^    [a-z][a-z0-9-]*:$", rest, re.M)
    return text[
        matches[0].start() : matches[0].end() + (nxt.start() if nxt else len(rest))
    ]


def test_run_flag_covers_shell_and_test_edits() -> None:
    """A geoip.sh edit and a suite edit each trigger the BusyBox job."""
    block = changes_output_block(workflow_text(), RUN_FLAG)
    assert "github.event_name != 'pull_request'" in block
    for component in ("wasm", "python", "ci"):
        assert f"steps.components.outputs.{component} == 'true'" in block, (
            f"{RUN_FLAG} must include the {component} component"
        )


def test_job_runs_suite_in_dockerfile_alpine_image() -> None:
    """The job exercises busybox through the production Alpine tag."""
    job = job_block(workflow_text(), JOB_NAME)
    assert "needs: [changes]" in job
    assert f"needs.changes.outputs.{RUN_FLAG} == 'true'" in job
    assert FOLDING_MARKER in job, (
        "the Alpine tag must be read from the last 'FROM alpine:' in origa_ui/Dockerfile"
    )
    assert "origa_ui/Dockerfile" in job
    assert "ALPINE_IMAGE: ${{ steps.alpine.outputs.image }}" in job
    assert '"$ALPINE_IMAGE"' in job
    assert "docker run --rm" in job
    assert PYTEST_MARKER in job, "the suite must run inside the container"
    assert re.search(r"alpine:\d+\.\d+", job) is None, (
        "a literal Alpine tag in the job recreates the two-pins drift class"
    )


def test_gate_asserts_busybox_job_under_its_own_flag() -> None:
    """The job is inside the single required check, with a matching assert."""
    text = workflow_text()
    gate = text[text.index("    check:\n") :]
    assert "                busybox-tests,\n" in gate
    gate_line = (
        '                  gate "busybox-tests" "${{ needs.busybox-tests.result }}" '
        '"${{ needs.changes.outputs.run_busybox_tests }}"'
    )
    assert gate.count(gate_line) == 1


def test_geoip_suite_invokes_script_through_sh() -> None:
    """The suite invokes geoip.sh via ``sh``, which is busybox in the image."""
    suite = (REPO_ROOT / GEOIP_TEST_RELPATH).read_text(encoding="utf-8")
    assert '["sh", str(SCRIPT)]' in suite, (
        "the contract suite must run geoip.sh through sh, not an explicit GNU shell"
    )


def test_job_block_missing_fails() -> None:
    with pytest.raises(AssertionError, match="exactly one job"):
        job_block(workflow_text(), "nonexistent-job")


def test_changes_output_missing_fails() -> None:
    with pytest.raises(AssertionError, match="exactly one"):
        changes_output_block(workflow_text(), "run_nonexistent")
