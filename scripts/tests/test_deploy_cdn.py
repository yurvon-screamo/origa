"""Regression tests for ``deploy_cdn.py``.

The ``compute_files_to_upload(force=True)`` behaviour is the fix for the CDN
consistency bug (issue #178 follow-up): when the remote manifest is current but
the underlying S3 objects are stale, manifest-only comparison would falsely
report "no changes" and the CDN would stay broken. ``--force`` must mark every
local file as changed regardless of remote state.

These tests are pure-dict and have no dependency on the gitignored ``cdn/``
store, the network, or AWS credentials.
"""

from __future__ import annotations

from deploy_cdn import VERSIONED_FILES, compare_manifests, compute_files_to_upload
from _cdn_verify import MANIFEST_ERROR


def _manifest(overrides: dict[str, str] | None = None) -> dict[str, object]:
    """Build a manifest dict over the full VERSIONED_FILES list.

    Every file defaults to ``"default"``; pass ``overrides`` to vary specific
    entries. This keeps tests independent of how many versioned files ship.
    """
    files = {path: "default" for path in VERSIONED_FILES}
    if overrides:
        files.update(overrides)
    return {"version": 1, "files": files}


# ---------------------------------------------------------------------------
# compute_files_to_upload — force mode (the bug fix)
# ---------------------------------------------------------------------------


def test_force_marks_all_files_changed_with_current_remote():
    local = _manifest()
    remote = _manifest()

    changed, unchanged = compute_files_to_upload(local, remote, force=True)

    assert len(changed) == len(VERSIONED_FILES)
    assert set(changed) == set(VERSIONED_FILES)
    assert unchanged == []


def test_force_marks_all_files_changed_with_none_remote():
    local = _manifest()

    changed, unchanged = compute_files_to_upload(local, None, force=True)

    assert set(changed) == set(VERSIONED_FILES)
    assert unchanged == []


def test_force_ignores_hash_mismatch_with_remote():
    local = _manifest({"grammar/grammar.json": "local_hash"})
    remote = _manifest({"grammar/grammar.json": "remote_hash"})

    changed, unchanged = compute_files_to_upload(local, remote, force=True)

    assert unchanged == []
    assert len(changed) == len(VERSIONED_FILES)


# ---------------------------------------------------------------------------
# compute_files_to_upload — default mode (backward compat)
# ---------------------------------------------------------------------------


def test_default_with_none_remote_treats_all_as_changed():
    local = _manifest()

    changed, unchanged = compute_files_to_upload(local, None, force=False)

    assert set(changed) == set(VERSIONED_FILES)
    assert unchanged == []


def test_default_reports_only_hash_diff_as_changed():
    changed_path = "grammar/grammar.json"
    local = _manifest({changed_path: "new_hash"})
    remote = _manifest()

    changed, unchanged = compute_files_to_upload(local, remote, force=False)

    assert changed == [changed_path]
    assert changed_path not in unchanged
    assert len(unchanged) == len(VERSIONED_FILES) - 1


def test_default_with_identical_remote_reports_no_changes():
    local = _manifest()

    changed, unchanged = compute_files_to_upload(local, local, force=False)

    assert changed == []
    assert set(unchanged) == set(VERSIONED_FILES)


def test_default_treats_missing_remote_entry_as_changed():
    changed_path = "dictionary/kanji.json"
    local = _manifest()
    remote = _manifest()
    remote["files"].pop(changed_path)

    changed, unchanged = compute_files_to_upload(local, remote, force=False)

    assert changed == [changed_path]
    assert changed_path not in unchanged


# ---------------------------------------------------------------------------
# compare_manifests (delegates for force=False)
# ---------------------------------------------------------------------------


def test_compare_manifests_returns_all_keys_when_remote_is_none():
    local = _manifest()

    changed, unchanged = compare_manifests(local, None)

    assert set(changed) == set(VERSIONED_FILES)
    assert unchanged == []


# ---------------------------------------------------------------------------
# _cdn_verify sentinel contract
# ---------------------------------------------------------------------------


def test_manifest_error_sentinel_is_distinct_from_problem_counts():
    # A valid problem count is >= 0; the sentinel must not collide with that
    # so callers can tell "manifest broken" from "N files mismatched".
    assert MANIFEST_ERROR < 0
