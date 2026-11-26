"""Regression pins for the shared sccache compile cache (uwuwu#1495, PR origa#976).

The change replaced Swatinem/rust-cache with a single sccache-backed composite
(`.github/actions/setup-rust/action.yml`), wired S3 credentials through every
compile job and removed the dead v0-rust family from cleanup-cache.yml. Layer
YAML is the only runtime here — cargo/bun tests never see it — so the tests
below read the live workflow files and execute the real embedded bash/python,
the same way the sibling CI-guard suites do. Each group pins a declared
acceptance criterion or a finding from the review rounds:

1. UC-1: with both secrets the composite exports the wrapper and the S3
   backend config; without a full pair it degrades to an uncached build and
   stays green (Dependabot PRs have no repo secrets).
2. Release builds must not silently pick up a new sccache: the action is
   pinned by SHA and the compiler version is explicit (review round 1 Common).
3. NFT-1: no second compile-cache mechanism survives in .github — no direct
   rust-cache use, no removed inputs at the composite call sites, no WASM
   opt-out and no duplicate stats step (review round 2 Low).
4. UC-2: secrets reach the Windows/MSIX, iOS and macOS release builders —
   declared in `workflow_call.secrets`, mapped to env, passed by every caller.
5. NFT-2: cleanup-cache no longer groups v0-rust keys as a family; the keys
   left behind by the old mechanism are reaped by the orphan rule.
"""

from __future__ import annotations

import os
import re
import subprocess
from pathlib import Path

import pytest
import yaml

_REPO_ROOT = Path(__file__).resolve().parents[2]
_GH = _REPO_ROOT / ".github"
ACTION = _GH / "actions" / "setup-rust" / "action.yml"
CLEANUP = _GH / "workflows" / "cleanup-cache.yml"
SECRET_KEYS = ("SCCACHE_S3_ACCESS_KEY_ID", "SCCACHE_S3_SECRET_KEY")

# The env that makes sccache a shared, bounded, content-addressed cache rather
# than a per-job local one. A drift in any line silently kills the pilot while
# leaving CI green, so the whole set is pinned as observable output.
EXPECTED_SCCACHE_ENV = {
    "CARGO_INCREMENTAL": "0",
    "RUSTC_WRAPPER": "sccache",
    "SCCACHE_IDLE_TIMEOUT": "60",
    "SCCACHE_CACHE_SIZE": "2G",
    "SCCACHE_BUCKET": "origa-sccache",
    "SCCACHE_ENDPOINT": "https://t3.storage.dev",
    "SCCACHE_REGION": "auto",
    "AWS_REQUEST_CHECKSUM_CALCULATION": "when_required",
    "AWS_RESPONSE_CHECKSUM_CALCULATION": "when_required",
}


def _load(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


# The files this PR owns. Scoping explicitly (instead of globbing all of
# .github/workflows) keeps unrelated workflows merged into master in the
# meantime — e.g. seed-cache.yml from #922 — out of a contract owned here.
PR_WORKFLOWS = (
    "ci.yml",
    "tauri.yml",
    "weekly-quality.yml",
    "cleanup-cache.yml",
    "_build-tauri.yml",
    "_build-tauri-apple.yml",
    "_build-windows-store.yml",
)


def _yaml_files() -> list[Path]:
    return [ACTION, *(_GH / "workflows" / name for name in PR_WORKFLOWS)]


def _iter_steps(doc: dict):
    """Yield every step of a composite action or a workflow, including reusable."""
    if "runs" in doc:
        yield from doc["runs"]["steps"]
    for job in doc.get("jobs", {}).values():
        yield from job.get("steps", []) or []


def _enable_step() -> dict:
    steps = [
        s
        for s in _load(ACTION)["runs"]["steps"]
        if s.get("name") == "Enable sccache (S3 backend)"
    ]
    assert len(steps) == 1, "the composite must have exactly one sccache enable step"
    return steps[0]


def _run_enable(tmp_path: Path, **secrets: str) -> tuple[int, str, dict[str, str]]:
    """Run the composite's real bash step against a throwaway GITHUB_ENV."""
    env_file = tmp_path / "github_env"
    env_file.write_text("", encoding="utf-8")
    env = os.environ.copy()
    env["GITHUB_ENV"] = str(env_file)
    for key in SECRET_KEYS:
        env.pop(key, None)
    env.update(secrets)
    proc = subprocess.run(
        ["bash", "-c", _enable_step()["run"]],
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    lines = [line for line in env_file.read_text(encoding="utf-8").splitlines() if line]
    return proc.returncode, proc.stdout, dict(line.split("=", 1) for line in lines)


def _workflow_env(doc: dict) -> dict:
    """All env mappings that can feed a job (top level + per job)."""
    combined: dict = {}
    if isinstance(doc.get("env"), dict):
        combined.update(doc["env"])
    for job in doc.get("jobs", {}).values():
        if isinstance(job.get("env"), dict):
            combined.update(job["env"])
    return combined


def _on(doc: dict) -> dict:
    # PyYAML 1.1 parses the bare `on:` key as boolean True.
    return doc.get("on", doc.get(True, {}))


# --- UC-1: the composite actually turns sccache on, and fails soft -----------


def test_enable_step_exports_sccache_backend_env(tmp_path: Path):
    rc, stdout, written = _run_enable(
        tmp_path,
        SCCACHE_S3_ACCESS_KEY_ID="HFAK_test_id",
        SCCACHE_S3_SECRET_KEY="test_secret",
    )
    assert rc == 0, stdout
    assert "uncached" not in stdout
    assert written == {
        **EXPECTED_SCCACHE_ENV,
        "AWS_ACCESS_KEY_ID": "HFAK_test_id",
        "AWS_SECRET_ACCESS_KEY": "test_secret",
    }


@pytest.mark.parametrize(
    "secrets",
    [
        {},
        {"SCCACHE_S3_ACCESS_KEY_ID": "HFAK_only"},
        {"SCCACHE_S3_SECRET_KEY": "only"},
    ],
    ids=["no-secrets", "id-only", "secret-only"],
)
def test_enable_step_is_fail_soft_on_missing_credentials(tmp_path: Path, secrets: dict):
    rc, stdout, written = _run_enable(tmp_path, **secrets)
    assert rc == 0, "a missing secret must degrade to an uncached build, not a red one"
    assert written == {}, "no wrapper may be installed without a full credential pair"
    assert "compiling uncached" in stdout


def test_sccache_action_pinned_by_sha_and_compiler_version():
    steps = [
        s for s in _load(ACTION)["runs"]["steps"] if s.get("name") == "Install sccache"
    ]
    assert len(steps) == 1
    step = steps[0]
    assert re.fullmatch(
        r"mozilla-actions/sccache-action@[0-9a-f]{40}", str(step.get("uses"))
    ), f"sccache action is not SHA-pinned: {step.get('uses')}"
    # Cache keys are version-specific: an unpinned version silently changes
    # release builds whenever upstream publishes a new compiler.
    assert step.get("with", {}).get("version") == "v0.20.1"


# --- NFT-1: one cache mechanism only -----------------------------------------


def test_no_direct_rust_cache_use_in_github():
    for path in _yaml_files():
        for step in _iter_steps(_load(path)):
            assert "Swatinem/rust-cache" not in str(step.get("uses", "")), (
                f"{path.name} still calls rust-cache directly"
            )


def test_composite_declares_only_targets():
    declared = set(_load(ACTION).get("inputs", {}))
    assert declared == {"targets"}, (
        "a rust-cache-era input is back in the composite — two mechanisms again"
    )


def test_setup_rust_callsites_pass_no_removed_inputs():
    calls = [
        step
        for path in _yaml_files()
        for step in _iter_steps(_load(path))
        if step.get("uses") == "./.github/actions/setup-rust"
    ]
    assert calls, "setup-rust call sites not found — the contract check is blind"
    for step in calls:
        assert set(step.get("with") or {}) <= {"targets"}, (
            f"removed rust-cache input still passed to setup-rust: {step.get('with')}"
        )


def test_no_wasm_opt_out_or_duplicate_stats_step():
    for path in _yaml_files():
        assert "SCCACHE_DISABLE" not in path.read_text(encoding="utf-8"), (
            f"{path.name} reintroduced a second cache mode"
        )
    for step in _iter_steps(_load(_GH / "workflows" / "ci.yml")):
        assert "sccache --show-stats" not in str(step.get("run", "")), (
            "a separate stats step duplicates the sccache-action post-step"
        )


# --- UC-2: release builders receive the credentials --------------------------


@pytest.mark.parametrize(
    "name",
    ["_build-tauri.yml", "_build-tauri-apple.yml", "_build-windows-store.yml"],
)
def test_reusable_builders_declare_and_map_sccache_secrets(name: str):
    doc = _load(_GH / "workflows" / name)
    declared = _on(doc).get("workflow_call", {}).get("secrets", {})
    for key in SECRET_KEYS:
        assert key in declared, (
            f"{name} does not declare {key} in workflow_call.secrets"
        )
        assert declared[key].get("required") is False, f"{name} made {key} mandatory"
    env = _workflow_env(doc)
    for key in SECRET_KEYS:
        assert env.get(key) == "${{ secrets." + key + " }}", (
            f"{name} does not expose {key} to its composite steps"
        )


def test_every_reusable_builder_caller_passes_sccache_secrets():
    checked = 0
    for path in (_GH / "workflows").glob("*.yml"):
        for job_name, job in _load(path).get("jobs", {}).items():
            if "_build-" not in str(job.get("uses", "")):
                continue
            declared = job.get("secrets")
            assert isinstance(declared, dict), (
                f"{path.name}:{job_name} does not pass secrets to the reusable builder"
            )
            for key in SECRET_KEYS:
                assert declared.get(key) == "${{ secrets." + key + " }}", (
                    f"{path.name}:{job_name} drops {key} — release builds go uncached"
                )
            checked += 1
    # ci.yml ×2 + tauri.yml ×3 on this branch; the loop covers future callers.
    assert checked >= 4, f"only {checked} reusable builder call sites found"


@pytest.mark.parametrize("name", ["ci.yml", "tauri.yml", "weekly-quality.yml"])
def test_compile_workflows_expose_sccache_secrets(name: str):
    env = _workflow_env(_load(_GH / "workflows" / name))
    for key in SECRET_KEYS:
        assert env.get(key) == "${{ secrets." + key + " }}", (
            f"{name} does not forward {key} to setup-rust"
        )


# --- NFT-2: the dead v0-rust family is gone from cleanup-cache ---------------


def test_v0_rust_keys_are_no_longer_grouped_as_a_family():
    run = _load(CLEANUP)["jobs"]["scheduled-cleanup"]["steps"][0]["run"]
    marker = re.search(r"python3 - <<'PY'\n(.*?)\nPY\s*$", run, re.DOTALL)
    assert marker, "embedded cleanup script not found in cleanup-cache.yml"
    head = marker.group(1).rsplit("\nmain()", 1)[0]
    namespace: dict = {}
    exec(compile(head, str(CLEANUP), "exec"), namespace)  # noqa: S102 - test rig
    family_of = namespace["family_of"]
    assert family_of("v0-rust-Linux-x86_64-abc-def") is None, (
        "old rust-cache keys are grouped again — they never reach the orphan rule"
    )
    # The function as a whole must still work for the families that remain.
    assert family_of("Linux-gradle-abc123") == "Linux-gradle"
