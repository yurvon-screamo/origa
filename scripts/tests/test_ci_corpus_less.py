"""Regression pins for the corpus-less half of the skip contract (ADR-083, PR #944).

The skip contract ("corpus absent -> graceful skip with a marker, corpus
broken -> loud panic") is only half-machine without the corpus-less CI step:
the Test job's later suites always run corpus-present (artifact unpack), so
the absent branch — fixtures probing and bailing out — is exercised exactly
once per run, by the "Run tests (corpus-less)" step. If that step quietly
disappears, or loses its empty ``ORIGA_CDN_DIR`` override, CI stays green and
the known ADR-083 hole ("a regression back to unwrap()/expect() on a corpus
fixture is invisible in CI") reopens with no red anywhere.

The step's behavior lives in the workflow runner, so the workflow text is the
only executable statement the repo makes about it — same doctrine as the
--no-fail-fast pins (test_ci_no_fail_fast.py). These pins read ci.yml and
assert, per pin:

1. The step exists in the ``test`` job — the absent branch stays on every
   PR run, not only on the one PR that introduced it.
2. The step env sets ``ORIGA_CDN_DIR`` to the empty string — the canonical
   unset (ADR-024, unit-pinned in origa/src/corpus.rs). The override is what
   makes the step deterministic: resolution falls back to ``~/origa-cdn``,
   which never exists on a fresh runner, regardless of step order or cache
   state. A deleted key would make the corpus state order-dependent
   (fail-open on reorder); a non-empty value pointing at an existing path
   would silently turn the step corpus-present.
3. The step runs the lib suite (``cargo test -p origa --lib``) — the
   contract scope: all skip-macro call-sites live in the lib target.
   (--no-fail-fast on the same command is pinned by test_ci_no_fail_fast.py;
   that file also pins the flag for this step — no duplication here.)
4. The step sits BEFORE "Download CDN artifact" — absent-by-construction:
   the corpus seeding of this job happens at the unpack, so anything running
   before it sees no corpus even without the override. Losing both the
   override and the position at once is the only way to make the step
   corpus-present; this pin keeps the second half loud.

Pure file reads, no network.
"""

from __future__ import annotations

from pathlib import Path

import pytest
import yaml

_REPO_ROOT = Path(__file__).resolve().parents[2]
CI_FILE = _REPO_ROOT / ".github" / "workflows" / "ci.yml"

JOB = "test"
CORPUS_LESS_STEP = "Run tests (corpus-less)"
ARTIFACT_UNPACK_STEP = "Download CDN artifact"
LIB_SUITE_COMMAND = "cargo test -p origa --lib"


def _workflow() -> dict:
    return yaml.safe_load(CI_FILE.read_text(encoding="utf-8"))


def _steps_of(job_name: str) -> list[dict]:
    jobs = _workflow()["jobs"]
    assert job_name in jobs, f"job {job_name!r} disappeared from ci.yml"
    steps = jobs[job_name].get("steps") or []
    assert steps, f"job {job_name!r} lost its steps"
    return steps


def _step_index(job_name: str, step_name: str) -> int:
    steps = _steps_of(job_name)
    matches = [i for i, s in enumerate(steps) if s.get("name") == step_name]
    assert len(matches) == 1, (
        f"expected exactly one step named {step_name!r} in job {job_name!r}, "
        f"found {len(matches)}"
    )
    return matches[0]


def test_corpus_less_step_exists():
    """The absent branch of the skip contract runs on every PR."""
    _step_index(JOB, CORPUS_LESS_STEP)  # raises with a fail-loud message


def test_corpus_less_step_overrides_cdn_dir_to_empty():
    """The step's corpus state must not depend on step order or runner state.

    The empty string is the canonical unset (ADR-024): resolution falls back
    to ~/origa-cdn, which a fresh runner never has. A missing key makes the
    step's behavior order-dependent (fail-open on a reorder); a non-empty
    value pointing at a seeded path turns the step corpus-present — both
    silently reopen the ADR-083 hole this step exists to close.
    """
    step = _steps_of(JOB)[_step_index(JOB, CORPUS_LESS_STEP)]
    env = step.get("env") or {}
    assert env.get("ORIGA_CDN_DIR") == "", (
        'the corpus-less step lost its ORIGA_CDN_DIR="" override: without '
        "the canonical empty-unset its corpus state depends on step order and "
        "runner state, and an absent-branch regression can hide again"
    )


def test_corpus_less_step_runs_the_lib_suite():
    """The step keeps the declared contract scope (all skip call-sites are lib)."""
    run = _steps_of(JOB)[_step_index(JOB, CORPUS_LESS_STEP)].get("run") or ""
    assert run, f"step {CORPUS_LESS_STEP!r} in job {JOB!r} lost its run command"
    assert LIB_SUITE_COMMAND in run, (
        f"the corpus-less step no longer runs {LIB_SUITE_COMMAND!r}: the "
        "absent branch must exercise the lib suite where every skip-macro "
        "call-site lives"
    )


def test_corpus_less_step_runs_before_cdn_artifact_unpack():
    """Absent-by-construction: the job seeds its corpus at the unpack.

    With the override this position is redundant by design — the pin keeps
    it that way (belt and suspenders against the override being dropped in
    the same edit that moves the step).
    """
    corpus_less = _step_index(JOB, CORPUS_LESS_STEP)
    unpack = _step_index(JOB, ARTIFACT_UNPACK_STEP)
    assert corpus_less < unpack, (
        f"step {CORPUS_LESS_STEP!r} must run before {ARTIFACT_UNPACK_STEP!r}: "
        "the unpack is this job's corpus seeding — a step after it only sees "
        "the absent corpus thanks to the env override, not by construction"
    )


# --- Parser units (synthetic inputs, fail-loud contract) -------------------


def test_missing_unpack_step_fails_loud():
    """The position pin cannot silently pass when the reference step dies."""
    with pytest.raises(AssertionError, match="exactly one step named"):
        _step_index(JOB, "Nonexistent Step")


def test_job_missing_fails_loud():
    with pytest.raises(AssertionError, match="disappeared from ci.yml"):
        _steps_of("nonexistent-job")
