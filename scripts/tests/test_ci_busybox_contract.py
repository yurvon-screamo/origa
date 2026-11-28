"""Tester pins for the recurring BusyBox CI check (uwuwu#932).

The feature is a CI gate, so a regression can be silent: if the path filter
stops routing shell/test edits to ``busybox-tests``, the job is skipped and
the single required check stays green. The engineer's guard pins the job body
and the gate line; these tests pin the parts that decide *whether* the job
runs, and the part that decides *whether it detects anything*:

- ``origa_ui/**`` (wasm) and ``scripts/**`` (python) edits flip
  ``run_busybox_tests``; an unrelated edit does not; non-PR events force it;
- the job's ``if`` flag, its ``check.needs`` entry and its gate assertion all
  use the same flag, and the gate-completeness guard accepts the real workflow;
- the exact CI run blocks, executed against a real Alpine container, pass on
  the current ``geoip.sh`` and turn red on a GNU-only ``tail --bytes=`` edit.

The container case is skipped when Docker is unavailable. The other tests are
pure file reads.
"""

from __future__ import annotations

import fnmatch
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

import pytest
import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]
CI_FILE = REPO_ROOT / ".github" / "workflows" / "ci.yml"
DOCKERFILE = REPO_ROOT / "origa_ui" / "Dockerfile"
GEOIP_SCRIPT = REPO_ROOT / "origa_ui" / "geoip.sh"
GEOIP_TEST = REPO_ROOT / "scripts" / "tests" / "test_geoip.py"
CONFTEST = REPO_ROOT / "scripts" / "tests" / "conftest.py"
GATE_COMPLETENESS = REPO_ROOT / ".github" / "scripts" / "check_gate_completeness.py"
JOB_NAME = "busybox-tests"
RUN_FLAG = "run_busybox_tests"

RUN_TERM = re.compile(r"steps\.components\.outputs\.([a-z_]+) == 'true'")


def _workflow() -> dict:
    return yaml.safe_load(CI_FILE.read_text(encoding="utf-8"))


def _filters() -> dict[str, list[str]]:
    steps = _workflow()["jobs"]["changes"]["steps"]
    detectors = [s for s in steps if s.get("id") == "components"]
    assert len(detectors) == 1, "expected one component detector in changes"
    return yaml.safe_load(detectors[0]["with"]["filters"])


def _components_for(path: str) -> set[str]:
    """Components dorny/paths-filter would report for one changed file."""
    return {
        name
        for name, patterns in _filters().items()
        if any(fnmatch.fnmatchcase(path, pattern) for pattern in patterns)
    }


def _flag_is_true_on_pr(expr: str, components: set[str]) -> bool:
    """Evaluate a run_* expression for a PR (the non-PR term is false)."""
    return any(match.group(1) in components for match in RUN_TERM.finditer(expr))


def test_shell_edit_triggers_busybox_job() -> None:
    expr = _workflow()["jobs"]["changes"]["outputs"][RUN_FLAG]
    for path in ("origa_ui/geoip.sh", "origa_ui/Dockerfile"):
        assert _flag_is_true_on_pr(expr, _components_for(path)), (
            f"a PR touching only {path} must run the BusyBox job"
        )


def test_contract_edit_triggers_busybox_job() -> None:
    expr = _workflow()["jobs"]["changes"]["outputs"][RUN_FLAG]
    for path in ("scripts/tests/test_geoip.py", ".github/workflows/ci.yml"):
        assert _flag_is_true_on_pr(expr, _components_for(path)), (
            f"a PR touching only {path} must run the BusyBox job"
        )


def test_unrelated_edit_does_not_trigger_but_release_does() -> None:
    expr = _workflow()["jobs"]["changes"]["outputs"][RUN_FLAG]
    assert not _flag_is_true_on_pr(expr, _components_for("README.md"))
    assert "github.event_name != 'pull_request'" in expr, (
        "tags and manual dispatch must force a full run"
    )


def test_job_flag_gate_needs_and_assertion_agree() -> None:
    wf = _workflow()
    job = wf["jobs"][JOB_NAME]
    if_terms = re.findall(
        r"needs\.changes\.outputs\.([a-z_]+) == 'true'", str(job.get("if"))
    )
    assert if_terms == [RUN_FLAG]

    gate = wf["jobs"]["check"]
    assert JOB_NAME in gate["needs"]
    gate_script = "\n".join(str(s.get("run") or "") for s in gate["steps"])
    gate_terms = re.findall(
        r'gate "' + JOB_NAME + r'" .*?needs\.changes\.outputs\.([a-z_]+)',
        gate_script,
    )
    assert gate_terms == [RUN_FLAG], (
        "the gate assertion must use the same flag as the job's if"
    )


def test_gate_completeness_guard_accepts_real_workflow() -> None:
    result = subprocess.run(
        [sys.executable, str(GATE_COMPLETENESS)],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    assert "::error::" not in result.stdout
    assert "wired into the CI Gate" in result.stdout


def _alpine_from_dockerfile() -> str:
    images = [
        line.split()[1]
        for line in DOCKERFILE.read_text(encoding="utf-8").splitlines()
        if line.startswith("FROM alpine:")
    ]
    assert images, "no production FROM alpine: in Dockerfile"
    return images[-1]


def _fixture(root: Path, gnu_only: bool) -> Path:
    """A minimal repo copy the CI docker command can mount."""
    (root / "scripts" / "tests").mkdir(parents=True)
    (root / "origa_ui").mkdir()
    shutil.copy(GEOIP_TEST, root / "scripts" / "tests" / "test_geoip.py")
    shutil.copy(CONFTEST, root / "scripts" / "tests" / "conftest.py")
    shutil.copy(GEOIP_SCRIPT, root / "origa_ui" / "geoip.sh")
    script = (root / "origa_ui" / "geoip.sh").read_text(encoding="utf-8")
    if gnu_only:
        assert "tail -c " in script, "the mutation target disappeared from geoip.sh"
        script = script.replace("tail -c ", "tail --bytes=")
    (root / "origa_ui" / "geoip.sh").write_text(script, encoding="utf-8")
    return root


@pytest.mark.skipif(shutil.which("docker") is None, reason="docker unavailable")
def test_busybox_job_detects_gnu_only_script(tmp_path: Path) -> None:
    """Run the exact CI steps: clean script green, GNU-only edit red."""
    info = subprocess.run(
        ["docker", "info"], capture_output=True, text=True, timeout=60
    )
    if info.returncode != 0:
        pytest.skip("docker daemon unavailable")

    steps = _workflow()["jobs"][JOB_NAME]["steps"]
    read_step = [s for s in steps if s.get("id") == "alpine"]
    assert len(read_step) == 1, "the Alpine-tag step disappeared"
    docker_steps = [s for s in steps if "docker run" in (s.get("run") or "")]
    assert len(docker_steps) == 1, "the container run step disappeared"

    gh_output = tmp_path / "gh_output"
    detected = subprocess.run(
        ["bash", "-ec", read_step[0]["run"]],
        cwd=REPO_ROOT,
        env=os.environ | {"GITHUB_OUTPUT": str(gh_output)},
        capture_output=True,
        text=True,
        timeout=120,
    )
    assert detected.returncode == 0, detected.stdout + detected.stderr
    assert gh_output.read_text(encoding="utf-8").strip() == (
        f"image={_alpine_from_dockerfile()}"
    ), "the job must follow the production Alpine tag, not a second pin"
    image = gh_output.read_text(encoding="utf-8").strip().split("=", 1)[1]

    def run_case(name: str, gnu_only: bool) -> subprocess.CompletedProcess[str]:
        case = _fixture(tmp_path / name, gnu_only)
        return subprocess.run(
            ["bash", "-ec", docker_steps[0]["run"]],
            cwd=case,
            env=os.environ | {"ALPINE_IMAGE": image},
            capture_output=True,
            text=True,
            timeout=420,
        )

    clean = run_case("clean", gnu_only=False)
    assert clean.returncode == 0, clean.stdout + clean.stderr
    assert re.search(r"\d+ passed", clean.stdout), clean.stdout + clean.stderr

    mutant = run_case("mutant", gnu_only=True)
    assert mutant.returncode != 0, "a GNU-only tail edit must turn the BusyBox job red"
    assert re.search(r"\d+ failed", mutant.stdout), mutant.stdout + mutant.stderr
