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

import json

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


# ---------------------------------------------------------------------------
# rkyv blob freshness (deploy-time guard against stale pre-parsed blobs)
# ---------------------------------------------------------------------------

import hashlib
from pathlib import Path

import pytest

from deploy_cdn import (
    RKYV_BLOB_SOURCES,
    RKYV_HEADER_LEN,
    RKYV_SCHEMA_VERSION,
    parse_rkyv_header,
    rkyv_blob_freshness_problems,
)


def _write_blob(path: Path, schema_version: int, source_sha: bytes) -> None:
    header = b"ORFG" + schema_version.to_bytes(4, "little") + source_sha + b"\x00" * 32
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(header + b"payload")


def _sources_with_content(tmp_path: Path, furigana: bytes, chunk: bytes) -> None:
    (tmp_path / "dictionaries").mkdir(exist_ok=True)
    (tmp_path / "dictionary").mkdir(exist_ok=True)
    (tmp_path / "dictionaries/JmdictFurigana.txt").write_bytes(furigana)
    for index in range(1, 12):
        (tmp_path / f"dictionary/chunk_{index:02d}.json").write_bytes(chunk)


def _concatenated_source_digest(furigana: bytes, chunk: bytes) -> bytes:
    del furigana  # vocabulary blob sources are the 11 chunks only
    hasher = hashlib.sha256()
    hasher.update(chunk * 11)
    return hasher.digest()


FURIGANA_CONTENT = b"\xe6\x8c\x87|yubi|0:yubi\n"
CHUNK_CONTENT = b'{"word": {}}'


def test_fresh_blobs_produce_no_problems(tmp_path: Path):
    # Arrange
    _sources_with_content(tmp_path, FURIGANA_CONTENT, CHUNK_CONTENT)
    furigana_sha = hashlib.sha256(FURIGANA_CONTENT).digest()
    _write_blob(tmp_path / "dictionaries/JmdictFurigana.rkyv", RKYV_SCHEMA_VERSION, furigana_sha)
    _write_blob(
        tmp_path / "dictionary/vocabulary.rkyv",
        RKYV_SCHEMA_VERSION,
        _concatenated_source_digest(FURIGANA_CONTENT, CHUNK_CONTENT),
    )

    # Act
    problems = rkyv_blob_freshness_problems(tmp_path)

    # Assert
    assert problems == []


def test_missing_blob_is_reported(tmp_path: Path):
    # Arrange: sources exist, no blobs at all
    _sources_with_content(tmp_path, FURIGANA_CONTENT, CHUNK_CONTENT)

    # Act
    problems = rkyv_blob_freshness_problems(tmp_path)

    # Assert
    assert len(problems) == len(RKYV_BLOB_SOURCES)
    assert all("missing" in problem for problem in problems)


def _write_fresh_blobs(tmp_path: Path) -> None:
    """Write both blobs with headers matching the on-disk sources."""
    _write_blob(
        tmp_path / "dictionaries/JmdictFurigana.rkyv",
        RKYV_SCHEMA_VERSION,
        hashlib.sha256(FURIGANA_CONTENT).digest(),
    )
    _write_blob(
        tmp_path / "dictionary/vocabulary.rkyv",
        RKYV_SCHEMA_VERSION,
        _concatenated_source_digest(FURIGANA_CONTENT, CHUNK_CONTENT),
    )


def test_stale_blob_hash_is_reported(tmp_path: Path):
    # Arrange: furigana blob header claims the hash of DIFFERENT sources
    _sources_with_content(tmp_path, FURIGANA_CONTENT, CHUNK_CONTENT)
    _write_fresh_blobs(tmp_path)
    _write_blob(
        tmp_path / "dictionaries/JmdictFurigana.rkyv",
        RKYV_SCHEMA_VERSION,
        hashlib.sha256(b"other content").digest(),
    )

    # Act
    problems = rkyv_blob_freshness_problems(tmp_path)

    # Assert
    assert problems == [
        "dictionaries/JmdictFurigana.rkyv: stale relative to "
        "['dictionaries/JmdictFurigana.txt']"
    ]


def test_future_schema_version_is_reported(tmp_path: Path):
    # Arrange
    _sources_with_content(tmp_path, FURIGANA_CONTENT, CHUNK_CONTENT)
    _write_fresh_blobs(tmp_path)
    _write_blob(
        tmp_path / "dictionaries/JmdictFurigana.rkyv",
        RKYV_SCHEMA_VERSION + 1,
        hashlib.sha256(FURIGANA_CONTENT).digest(),
    )

    # Act
    problems = rkyv_blob_freshness_problems(tmp_path)

    # Assert
    assert len(problems) == 1
    assert "schema" in problems[0]


def test_corrupted_header_is_reported(tmp_path: Path):
    # Arrange: magic destroyed after both blobs were written fresh
    _sources_with_content(tmp_path, FURIGANA_CONTENT, CHUNK_CONTENT)
    _write_fresh_blobs(tmp_path)
    blob_path = tmp_path / "dictionaries/JmdictFurigana.rkyv"
    blob_path.write_bytes(b"XXXX" + blob_path.read_bytes()[4:])

    # Act
    problems = rkyv_blob_freshness_problems(tmp_path)

    # Assert
    assert len(problems) == 1
    assert "unreadable" in problems[0]


def test_parse_rkyv_header_rejects_short_blob(tmp_path: Path):
    blob = tmp_path / "short.rkyv"
    blob.write_bytes(b"ORFG" + b"\x00" * 8)

    with pytest.raises(ValueError):
        parse_rkyv_header(blob)


def test_rkyv_blobs_are_registered_as_versioned_files():
    for blob_path in RKYV_BLOB_SOURCES:
        assert blob_path in VERSIONED_FILES, f"{blob_path} must be deployed"


def test_missing_source_file_is_reported(tmp_path: Path):
    # Arrange: blobs are fresh, but a source chunk is gone
    _sources_with_content(tmp_path, FURIGANA_CONTENT, CHUNK_CONTENT)
    _write_fresh_blobs(tmp_path)
    (tmp_path / "dictionary/chunk_05.json").unlink()

    # Act
    problems = rkyv_blob_freshness_problems(tmp_path)

    # Assert
    assert problems == [
        "dictionary/vocabulary.rkyv: source dictionary/chunk_05.json not found"
    ]


class TestGenerateKanjiArtManifest:
    """#540: the manifest must list exactly the single-kanji SVG stems per
    kind, sorted, so clients can filter doomed 404 art requests."""

    def test_lists_single_kanji_stems_sorted_per_kind(self, tmp_path):
        from deploy_cdn import generate_kanji_art_manifest

        frames = tmp_path / "kanji_frames"
        animations = tmp_path / "kanji_animations"
        frames.mkdir()
        animations.mkdir()
        (frames / "本.svg").write_text("x", encoding="utf-8")
        (frames / "日.svg").write_text("x", encoding="utf-8")
        (frames / "日語.svg").write_text("x", encoding="utf-8")  # multi-char: filtered
        (animations / "日.svg").write_text("x", encoding="utf-8")

        generate_kanji_art_manifest(tmp_path)

        manifest = json.loads(
            (tmp_path / "kanji_art_manifest.json").read_text(encoding="utf-8")
        )
        assert manifest == {"frames": ["日", "本"], "animations": ["日"]}

    def test_empty_directories_produce_empty_lists(self, tmp_path):
        from deploy_cdn import generate_kanji_art_manifest

        (tmp_path / "kanji_frames").mkdir()
        (tmp_path / "kanji_animations").mkdir()

        generate_kanji_art_manifest(tmp_path)

        manifest = json.loads(
            (tmp_path / "kanji_art_manifest.json").read_text(encoding="utf-8")
        )
        assert manifest == {"frames": [], "animations": []}


# ---------------------------------------------------------------------------
# sync_directories — per-dir size-only change detection (#551)
# ---------------------------------------------------------------------------


def test_sync_directories_size_only_for_truly_static_dirs(tmp_path, monkeypatch):
    """Immutable sync dirs compare by size only; release-updated keep mtime.

    The #551 regression: a git branch switch bulk-refreshes local mtimes, and
    the mtime rule then re-uploads 100k+ unchanged audio files on every
    deploy. Truly-static dirs (immutable cache tier) must trust byte size.
    """
    import deploy_cdn

    # One static and one release-updated dir, both present locally.
    static_dir = tmp_path / "phrases" / "audio"
    static_dir.mkdir(parents=True)
    release_dir = tmp_path / "well_known_set" / "minna_n5"
    release_dir.mkdir(parents=True)
    (static_dir / "a.opus").write_bytes(b"x")
    (release_dir / "set.json").write_text("{}", encoding="utf-8")

    received: list[tuple[str, bool]] = []

    def fake_sync(local_dir, prefix, cache_control, dry_run, ignore_mtime=False, force=False):
        received.append((prefix, ignore_mtime))

    monkeypatch.setattr(deploy_cdn._cdn_s3, "sync_directory", fake_sync)

    deploy_cdn.sync_directories(tmp_path, dry_run=False, ignore_mtime=False)

    by_prefix = dict(received)
    assert by_prefix["phrases/audio"] is True  # static: size-only
    assert by_prefix["well_known_set/minna_n5"] is False  # release: keep mtime


def test_sync_directories_ignore_mtime_flag_covers_release_and_static(
    tmp_path, monkeypatch
):
    """``--ignore-mtime`` overrides the mtime rule in every directory."""
    import deploy_cdn

    static_dir = tmp_path / "fonts"
    static_dir.mkdir()
    (static_dir / "f.woff2").write_bytes(b"x")
    release_dir = tmp_path / "well_known_set" / "minna_n5"
    release_dir.mkdir(parents=True)
    (release_dir / "set.json").write_text("{}", encoding="utf-8")

    received: list[tuple[str, bool]] = []

    def fake_sync(local_dir, prefix, cache_control, dry_run, ignore_mtime=False, force=False):
        received.append((prefix, ignore_mtime))

    monkeypatch.setattr(deploy_cdn._cdn_s3, "sync_directory", fake_sync)

    deploy_cdn.sync_directories(tmp_path, dry_run=False, ignore_mtime=True)

    by_prefix = dict(received)
    assert by_prefix["well_known_set/minna_n5"] is True
    assert by_prefix["fonts"] is True


def test_sync_directories_force_bypasses_diff_for_all_dirs(tmp_path, monkeypatch):
    """``--force`` re-uploads every sync-dir file — the recovery path when a
    size-only directory receives a same-size content edit. Both static and
    release dirs are covered so a future narrowing of force to size-only
    dirs cannot slip through."""
    import deploy_cdn

    static_dir = tmp_path / "fonts"
    static_dir.mkdir()
    (static_dir / "f.woff2").write_bytes(b"x")
    release_dir = tmp_path / "well_known_set" / "minna_n5"
    release_dir.mkdir(parents=True)
    (release_dir / "set.json").write_text("{}", encoding="utf-8")

    received: list[tuple[str, bool]] = []

    def fake_sync(local_dir, prefix, cache_control, dry_run, ignore_mtime=False, force=False):
        received.append((prefix, force))

    monkeypatch.setattr(deploy_cdn._cdn_s3, "sync_directory", fake_sync)

    deploy_cdn.sync_directories(tmp_path, dry_run=False, force=True)

    by_prefix = dict(received)
    assert by_prefix["fonts"] is True
    assert by_prefix["well_known_set/minna_n5"] is True


def test_size_only_sync_dirs_are_immutable_tier():
    """Two-way coupling guard between sync mode and cache policy.

    Forward: every size-only dir must be in SYNC_DIRS and map to the
    immutable Cache-Control tier. Reverse: every immutable-tier SYNC_DIR
    must be size-only — otherwise a new static directory added to SYNC_DIRS
    and _cdn_cache but forgotten in SIZE_ONLY_SYNC_DIRS silently
    re-introduces the #551 mtime-skew re-upload for that directory.
    """
    import deploy_cdn
    from _cdn_cache import IMMUTABLE, cache_control_for

    from deploy_cdn import SIZE_ONLY_SYNC_DIRS, SYNC_DIRS

    assert SIZE_ONLY_SYNC_DIRS, "static set must not be empty"
    for dir_name in SIZE_ONLY_SYNC_DIRS:
        assert dir_name in SYNC_DIRS, dir_name
        assert cache_control_for(dir_name + "/") == IMMUTABLE, dir_name

    for dir_name in SYNC_DIRS:
        if cache_control_for(dir_name + "/") == IMMUTABLE:
            assert dir_name in SIZE_ONLY_SYNC_DIRS, dir_name
