"""Guards for the TrailBase version contract in the CI e2e jobs.

The production image pins TrailBase exactly once — ``ARG TRAILBASE_VERSION``
in ``origa_ui/Dockerfile`` — and the e2e jobs must race exactly that binary:
the auth contract under test is the contract production serves. The workflow
reads the version from the Dockerfile at run time (the "Read TrailBase
version" step) instead of keeping a second literal, so a prod bump PR moves
e2e with it automatically and drift between the two pins is impossible by
construction (task 20261108203611; the drift observed 2026-11-08: e2e raced
v0.28.3 while production ran v0.33.16 — several breaking releases apart,
nothing failed, nobody noticed).

What the guards assert, in one place:

- the Dockerfile declares exactly one well-formed
  ``ARG TRAILBASE_VERSION=vX.Y.Z``;
- the workflow reads the version from the Dockerfile and feeds the same step
  output into the cache key, the download URL and the validate step;
- the read step precedes the cache step — otherwise the cache key would be
  built from an empty output, the silent-drift analogue;
- the cache step has no ``restore-keys``: a fallback key would restore an
  old binary under a fresh version key (the validate step is the second
  line of defense — it compares the binary version, not just executability);
- no literal TrailBase versions remain in the workflow: reintroducing one
  recreates the two-independent-pins drift class;
- ``run_e2e`` keeps referencing the ``wasm`` component: a Dockerfile-only
  bump must trigger e2e through the path filter, or the contract degrades
  from "verified at bump time" to "verified eventually".

The tests are pure: they read the workflow/Dockerfile files (and synthetic
strings for the parser units) and never hit the network.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
CI_RELPATH = Path(".github/workflows/ci.yml")
DOCKERFILE_RELPATH = Path("origa_ui/Dockerfile")

# The read step's parser marker: the exact sed expression the workflow uses
# to lift the version out of the Dockerfile (path included, so the guard
# fails if either half moves).
READ_MARKER = "sed -n 's/^ARG TRAILBASE_VERSION=//p' origa_ui/Dockerfile"
READ_STEP_NAME = "Read TrailBase version from origa_ui/Dockerfile"
CACHE_STEP_NAME = "Cache TrailBase CLI"
VALIDATE_STEP_NAME = "Validate TrailBase CLI"
INSTALL_STEP_NAME = "Install TrailBase CLI"

CACHE_KEY_LINE = "key: trailbase-${{ steps.trailbase-version.outputs.version }}"
INSTALL_ENV_LINE = "TRAILBASE_VERSION: ${{ steps.trailbase-version.outputs.version }}"
VALIDATE_ENV_LINE = "EXPECTED_VERSION: ${{ steps.trailbase-version.outputs.version }}"
URL_TEMPLATE = "releases/download/${TRAILBASE_VERSION}/trailbase_${TRAILBASE_VERSION}_x86_64_linux.zip"

DOCKERFILE_VERSION_RE = re.compile(r"^ARG TRAILBASE_VERSION=(v\d+\.\d+\.\d+)\s*$", re.M)

# Literal version literals that must NOT appear in the workflow anymore —
# reintroducing one recreates the two-independent-pins drift class.
LITERAL_VERSION_RES = (
    # A pinned cache key like `trailbase-v0.28.3-verified`.
    re.compile(r"trailbase-v\d+\.\d+"),
    # A pinned download URL like `releases/download/v0.28.3/trailbase_v…`.
    re.compile(r"trailbase/releases/download/v\d+\.\d+"),
)


def workflow_text() -> str:
    return (REPO_ROOT / CI_RELPATH).read_text(encoding="utf-8")


def dockerfile_text() -> str:
    return (REPO_ROOT / DOCKERFILE_RELPATH).read_text(encoding="utf-8")


def extract_dockerfile_version(text: str) -> str:
    """The single ``ARG TRAILBASE_VERSION=…`` value.

    Raises AssertionError when the declaration is missing, duplicated or
    malformed (the regex only accepts ``vX.Y.Z``) — same contract as the
    workflow's read step, so the test fails on anything the runner would
    fail on, just earlier and with a targeted message.
    """
    matches = DOCKERFILE_VERSION_RE.findall(text)
    assert len(matches) == 1, (
        "origa_ui/Dockerfile must declare exactly one "
        f"'ARG TRAILBASE_VERSION=vX.Y.Z', found {len(matches)}: {matches}"
    )
    return matches[0]


def step_block(text: str, step_name: str) -> str:
    """The full YAML block of one job step: from its heading to the next
    step heading at the same indent level.
    """
    matches = list(
        re.finditer(rf"^            - name: {re.escape(step_name)}$", text, re.M)
    )
    assert len(matches) == 1, (
        f"expected exactly one step named {step_name!r} in ci.yml, found {len(matches)}"
    )
    heading = matches[0]
    rest = text[heading.end() :]
    nxt = re.search(r"^            - name: ", rest, re.M)
    end = heading.end() + (nxt.start() if nxt else len(rest))
    return text[heading.start() : end]


def changes_output_block(text: str, output_name: str) -> str:
    """The multi-line expression of one ``run_*`` output of the changes job:
    from its folded-scalar heading to the next key at the same level.
    """
    matches = list(
        re.finditer(rf"^            {re.escape(output_name)}: >-$", text, re.M)
    )
    assert len(matches) == 1, (
        f"expected exactly one '{output_name}: >-' output in ci.yml, found {len(matches)}"
    )
    heading = matches[0]
    rest = text[heading.end() :]
    nxt = re.search(r"^            \S", rest, re.M)
    return rest[: nxt.start() if nxt else len(rest)]


def test_dockerfile_pins_exactly_one_semver_version():
    """The single source of truth exists and is well-formed."""
    assert extract_dockerfile_version(dockerfile_text())


def test_workflow_reads_version_from_dockerfile():
    """The read step exists and parses the Dockerfile via the known marker."""
    text = workflow_text()
    assert text.count(READ_MARKER) == 1, (
        "the e2e job must lift the version from origa_ui/Dockerfile with the "
        "documented sed marker; moving it breaks the drift guard"
    )
    assert "id: trailbase-version" in text, "the read step must keep its id"


def test_cache_key_uses_read_step_output():
    """The cache key is built from the read step's output, not a literal."""
    text = workflow_text()
    assert text.count(CACHE_KEY_LINE) == 1, (
        "the TrailBase cache key must be 'trailbase-<read step output>'; "
        "a literal key recreates the second-pin drift class"
    )


def test_download_url_uses_read_step_output():
    """The install step builds the download URL from the same output."""
    install = step_block(workflow_text(), INSTALL_STEP_NAME)
    assert INSTALL_ENV_LINE in install, (
        "the install step must take TRAILBASE_VERSION from the read step output"
    )
    assert URL_TEMPLATE in install, (
        "the download URL must be built from the TRAILBASE_VERSION env, "
        "not a literal release tag"
    )


def test_validate_step_compares_binary_version():
    """The validate step checks the binary version, not just executability.

    A stale cache entry would otherwise run an old TrailBase under a fresh
    key; the version comparison is the second line of defense. The semver
    is extracted from `trail --version` and compared exactly — substring
    matching (unescaped dots, no anchors) would let `0.35.1` pass a
    v0.35.10 binary.
    """
    validate = step_block(workflow_text(), VALIDATE_STEP_NAME)
    assert VALIDATE_ENV_LINE in validate, (
        "the validate step must receive the expected version from the read step"
    )
    assert (
        "got=\"$(trail --version 2>&1 | grep -oE 'v[0-9]+\\.[0-9]+\\.[0-9]+' | head -n1)\""
        in validate
    ), "the validate step must extract the semver from `trail --version` output"
    assert '[[ "$got" == "$EXPECTED_VERSION" ]]' in validate, (
        "the validate step must compare the extracted semver exactly, not by substring"
    )


def test_read_step_precedes_cache_step():
    """The read step runs before the cache step.

    A cache key referencing a step that has not run yet evaluates to an
    empty string — `trailbase-` would cache-hit nothing and silently pin
    nothing, the quiet analogue of the drift this guard prevents.
    """
    text = workflow_text()
    assert text.index(READ_MARKER) < text.index(CACHE_KEY_LINE), (
        "the TrailBase version read step must run before the cache step"
    )


def test_cache_step_has_no_restore_keys():
    """No restore-keys on the TrailBase cache.

    A fallback key would restore an old binary under a new version key;
    the validate step would catch it (and trigger a fresh install), but the
    restored entry still poisons the cache for every subsequent job in the
    matrix.
    """
    cache = step_block(workflow_text(), CACHE_STEP_NAME)
    assert "restore-keys:" not in cache, (
        "the TrailBase cache step must not define restore-keys"
    )


@pytest.mark.parametrize("pattern", LITERAL_VERSION_RES, ids=lambda p: p.pattern)
def test_no_literal_trailbase_versions_in_workflow(pattern: re.Pattern[str]):
    """No literal TrailBase versions anywhere in the workflow.

    The Dockerfile ARG is the only version literal in the repo; a literal
    here would recreate the two-independent-pins drift this task closes.
    """
    assert pattern.search(workflow_text()) is None, (
        f"literal TrailBase version found in ci.yml (matches {pattern.pattern}): "
        "the version must come from origa_ui/Dockerfile via the read step"
    )


def test_run_e2e_includes_wasm_component():
    """A Dockerfile-only bump triggers e2e through the wasm path filter.

    The sync contract ("e2e races the prod version at bump time") relies on
    origa_ui/** changes flipping run_e2e; narrowing that filter would
    silently degrade the contract to "verified eventually" without any pin
    test noticing.
    """
    block = changes_output_block(workflow_text(), "run_e2e")
    assert "wasm == 'true'" in block, (
        "run_e2e must include the wasm component: a Dockerfile-only bump "
        "(origa_ui/Dockerfile) has to trigger the e2e job"
    )


# --- Parser units (synthetic inputs, fail-loud contract) -------------------


def test_dockerfile_version_missing_fails():
    with pytest.raises(AssertionError, match="exactly one"):
        extract_dockerfile_version("FROM alpine:3.24\n")


def test_dockerfile_version_duplicated_fails():
    text = "ARG TRAILBASE_VERSION=v0.35.10\nARG TRAILBASE_VERSION=v0.36.0\n"
    with pytest.raises(AssertionError, match="exactly one.*found 2"):
        extract_dockerfile_version(text)


@pytest.mark.parametrize(
    "line",
    [
        "ARG TRAILBASE_VERSION=v0.35",  # missing patch
        "ARG TRAILBASE_VERSION=v0.35.10.1",  # four components
        "ARG TRAILBASE_VERSION=0.35.10",  # missing v prefix
        "ARG TRAILBASE_VERSION=vX.Y.Z",  # non-numeric
        "ARG TRAILBASE_VERSION=",  # empty
    ],
)
def test_dockerfile_version_malformed_fails(line: str):
    with pytest.raises(AssertionError, match="found 0"):
        extract_dockerfile_version(f"{line}\n")


def test_step_block_missing_step_fails():
    with pytest.raises(AssertionError, match="exactly one step named"):
        step_block(workflow_text(), "Nonexistent Step")


def test_changes_output_missing_fails():
    with pytest.raises(AssertionError, match="exactly one"):
        changes_output_block(workflow_text(), "run_nonexistent")
