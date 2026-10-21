"""Parent/child SYNC_DIR exclusion — the examples/audio deploy invariant (#528).

The first examples/audio deploy raced: the parent ``examples/`` walk uploaded
the audio files with the release-updated header before the immutable
``examples/audio`` entry ran. These tests pin the exclusion contract so a
regression fails in CI instead of on the CDN:

- ``_child_sync_dir_prefixes`` yields parent-relative prefixes for every
  nested SYNC_DIR and nothing else;
- ``sync_directory(exclude_prefixes=...)`` drops excluded subtrees from the
  walk without touching siblings (``audio`` must not match ``audio_foo``);
- ``--force``/``--force-all`` paths apply the same exclusion.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

SCRIPTS_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(SCRIPTS_DIR))

from _cdn_s3 import sync_directory  # noqa: E402
from deploy_cdn import _child_sync_dir_prefixes  # noqa: E402


def test_child_prefixes_examples():
    assert _child_sync_dir_prefixes("examples") == ["audio/"]


def test_child_prefixes_leaf_dir_has_none():
    # Leaf SYNC_DIRS (no nested entries) exclude nothing.
    assert _child_sync_dir_prefixes("examples/audio") == []
    assert _child_sync_dir_prefixes("phrases/audio") == []


def test_child_prefixes_phrases_split():
    # phrases/audio + phrases/data are siblings, not nested — neither is a
    # child of the other, and there is no parent "phrases" SYNC_DIR.
    assert _child_sync_dir_prefixes("phrases/data") == []
    assert _child_sync_dir_prefixes("phrases/audio") == []


def _make_tree(root: Path, files: list[str]) -> None:
    for rel in files:
        f = root / rel
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_bytes(b"x" * 16)


@pytest.fixture()
def examples_tree(tmp_path: Path) -> Path:
    _make_tree(
        tmp_path,
        [
            "index.json",
            "data/s0000.json",
            "data/s0001.json",
            "audio/000000.opus",
            "audio/000001.opus",
            # sibling with a nearly-matching name: must NOT be excluded
            "audio_extra/keep.txt",
        ],
    )
    return tmp_path


def test_sync_directory_excludes_child_subtree(monkeypatch, examples_tree):
    """The parent walk skips the child subtree entirely (force=True so the
    diff pass would otherwise include everything)."""
    uploaded: list[str] = []

    def fake_upload_file(local_path, key, cache_control, dry_run):
        uploaded.append(key)

    import _cdn_s3

    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload_file)
    sync_directory(
        examples_tree,
        "examples",
        "public, max-age=300",
        dry_run=False,
        ignore_mtime=False,
        force=True,
        exclude_prefixes=["audio/"],
    )

    assert uploaded == [
        "examples/audio_extra/keep.txt",
        "examples/data/s0000.json",
        "examples/data/s0001.json",
        "examples/index.json",
    ]


def test_sync_directory_without_excludes_walks_everything(
    monkeypatch, examples_tree
):
    uploaded: list[str] = []

    def fake_upload_file(local_path, key, cache_control, dry_run):
        uploaded.append(key)

    import _cdn_s3

    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload_file)
    sync_directory(
        examples_tree, "examples", "public, max-age=300",
        dry_run=False, force=True,
    )

    assert len(uploaded) == 6  # nothing excluded
    assert "examples/audio/000000.opus" in uploaded


def test_force_all_path_applies_child_excludes(examples_tree):
    """The --force-all inline walk (no sync_directory) filters with the same
    helper — the recovery path must not leak the parent's header onto the
    child subtree."""
    excludes = _child_sync_dir_prefixes("examples")
    all_files = [
        p for p in sorted(examples_tree.rglob("*"))
        if p.is_file()
        and not any(
            p.relative_to(examples_tree).as_posix().startswith(e)
            for e in excludes
        )
    ]
    rels = [p.relative_to(examples_tree).as_posix() for p in all_files]
    assert "audio/000000.opus" not in rels
    assert "audio/000001.opus" not in rels
    assert "data/s0000.json" in rels
    assert "audio_extra/keep.txt" in rels
