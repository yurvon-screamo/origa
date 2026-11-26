"""Contract tests for the production CDN corpus materializer.

S3 is an external system, so a fake store supplies production bytes and
records downloads. The tests pin the failure modes that made the original
scratch scripts worth canonicalizing: SHA mismatch must not corrupt a good
seed, size-only trees must not re-download on matching sizes, and dry-run
must leave the corpus untouched.
"""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path

import pytest

from _cdn_materialize_io import (
    MaterializationError,
    clean_staging,
    local_path,
    staging_dir,
)
from materialize_cdn_corpus import materialize_sync_dir, materialize_versioned_files


def digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def temp_files(corpus: Path) -> list[Path]:
    """Temp files may live in any subdirectory, not just the corpus root."""
    return [path for path in corpus.rglob(".materialize-*") if path.is_file()]


class FakeStore:
    def __init__(
        self,
        files: dict[str, bytes],
        *,
        corrupt: frozenset[str] = frozenset(),
        manifest_keys: tuple[str, ...] | None = None,
    ) -> None:
        self.files = files
        self.corrupt = corrupt
        self.manifest_keys = manifest_keys or tuple(files)
        self.calls: list[str] = []

    def manifest(self) -> dict[str, object]:
        return {
            "version": 1,
            "files": {key: digest(self.files[key]) for key in self.manifest_keys},
        }

    def object_sizes(self, prefix: str) -> dict[str, int]:
        base = prefix.rstrip("/") + "/"
        return {
            key: len(value) for key, value in self.files.items() if key.startswith(base)
        }

    def download(self, key: str, target: Path) -> str:
        self.calls.append(key)
        payload = b"corrupt" if key in self.corrupt else self.files[key]
        target.write_bytes(payload)
        return digest(payload)


def make_corpus(root: Path, files: dict[str, bytes]) -> Path:
    corpus = root / "origa-cdn"
    for key, payload in files.items():
        target = corpus / key
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(payload)
    return corpus


def test_versioned_files_fetches_only_mismatches(tmp_path: Path) -> None:
    production = {
        "dictionary/kanji.json": b"prod-kanji",
        "grammar/grammar_v3.json": b"prod-grammar",
    }
    corpus = make_corpus(
        tmp_path,
        {"dictionary/kanji.json": b"prod-kanji", "grammar/grammar_v3.json": b"stale"},
    )
    kept_path = corpus / "dictionary/kanji.json"
    os.utime(kept_path, ns=(1_000_000_000, 1_000_000_000))
    store = FakeStore(production)

    kept, fetched = materialize_versioned_files(store, corpus, False)

    assert (kept, fetched) == (1, 1)
    assert kept_path.stat().st_mtime_ns == 1_000_000_000
    assert (corpus / "grammar/grammar_v3.json").read_bytes() == b"prod-grammar"
    assert store.calls == ["grammar/grammar_v3.json"]
    assert json.loads((corpus / "manifest.json").read_text(encoding="utf-8"))[
        "files"
    ] == {key: digest(value) for key, value in production.items()}
    assert (corpus / "grammar/grammar_v3.json").stat().st_mode & 0o777 == 0o644


def test_sha_mismatch_keeps_good_copy_and_leaves_no_temp(tmp_path: Path) -> None:
    corpus = make_corpus(tmp_path, {"dictionary/kanji.json": b"good"})
    store = FakeStore(
        {"dictionary/kanji.json": b"target"},
        corrupt=frozenset({"dictionary/kanji.json"}),
    )

    with pytest.raises(MaterializationError, match="sha256"):
        materialize_versioned_files(store, corpus, False)

    assert (corpus / "dictionary/kanji.json").read_bytes() == b"good"
    assert temp_files(corpus) == []
    assert not (corpus / "manifest.json").exists()


def test_interrupt_leaves_no_temp_outside_staging(tmp_path: Path) -> None:
    corpus = make_corpus(tmp_path, {"dictionary/kanji.json": b"good"})

    class InterruptStore(FakeStore):
        def download(self, key: str, target: Path) -> str:
            raise KeyboardInterrupt

    with pytest.raises(KeyboardInterrupt):
        materialize_versioned_files(
            InterruptStore({"dictionary/kanji.json": b"new"}), corpus, False
        )

    assert temp_files(corpus) == []
    staged_list = list(staging_dir(corpus).iterdir())
    assert staged_list == []
    assert (corpus / "dictionary/kanji.json").read_bytes() == b"good"


def test_stale_staging_leftover_is_removed(tmp_path: Path) -> None:
    corpus = make_corpus(tmp_path, {"dictionary/kanji.json": b"good"})
    directory = staging_dir(corpus)
    directory.mkdir()
    leftover = directory / ".materialize-old"
    leftover.write_bytes(b"partial")

    clean_staging(corpus)

    assert not leftover.exists()


def test_dry_run_writes_nothing(tmp_path: Path) -> None:
    corpus = make_corpus(tmp_path, {"dictionary/kanji.json": b"stale"})
    store = FakeStore({"dictionary/kanji.json": b"target"})

    kept, fetched = materialize_versioned_files(store, corpus, True)
    sync_kept, sync_fetched, stale = materialize_sync_dir(
        store, corpus, "phrases/data", True
    )

    assert (kept, fetched, sync_kept, sync_fetched, stale) == (0, 1, 0, 0, [])
    assert store.calls == []
    assert (corpus / "dictionary/kanji.json").read_bytes() == b"stale"
    assert not (corpus / "manifest.json").exists()


@pytest.mark.parametrize("prefix", ["phrases/data", "phrases/data/"])
def test_exact_sync_dir_compares_bytes_and_replaces_same_size_stale(
    tmp_path: Path, prefix: str
) -> None:
    corpus = make_corpus(tmp_path, {"phrases/data/一.json": b"OLD11"})
    store = FakeStore({"phrases/data/一.json": b"NEW22"})

    kept, fetched, stale = materialize_sync_dir(store, corpus, prefix, False)

    assert (kept, fetched, stale) == (0, 1, [])
    assert (corpus / "phrases/data/一.json").read_bytes() == b"NEW22"
    assert store.calls == ["phrases/data/一.json"]


def test_size_only_sync_dir_trusts_matching_size(tmp_path: Path) -> None:
    corpus = make_corpus(tmp_path, {"kanji_frames/一.svg": b"BBBB"})
    store = FakeStore({"kanji_frames/一.svg": b"AAAA"})

    kept, fetched, stale = materialize_sync_dir(store, corpus, "kanji_frames", False)

    assert (kept, fetched, stale) == (1, 0, [])
    assert store.calls == []
    assert (corpus / "kanji_frames/一.svg").read_bytes() == b"BBBB"


def test_sync_dir_fetches_missing_object_and_reports_local_only(tmp_path: Path) -> None:
    corpus = make_corpus(
        tmp_path,
        {
            "phrases/precomputed/existing.bin": b"1234",
            "phrases/data/legacy.json": b"obsolete",
            "phrases/data/README.md": b"notes",
        },
    )
    store = FakeStore(
        {
            "phrases/precomputed/existing.bin": b"1234",
            "phrases/precomputed/一.bin": b"5678",
        }
    )

    kept, fetched, stale = materialize_sync_dir(
        store, corpus, "phrases/precomputed", False
    )

    assert (kept, fetched) == (1, 1)
    assert (corpus / "phrases/precomputed/一.bin").read_bytes() == b"5678"
    data_kept, data_fetched, data_stale = materialize_sync_dir(
        store, corpus, "phrases/data", False
    )
    assert (data_kept, data_fetched, data_stale) == (0, 0, ["phrases/data/legacy.json"])
    assert (corpus / "phrases/data/legacy.json").read_bytes() == b"obsolete"


def test_second_write_run_is_idempotent(tmp_path: Path) -> None:
    production = {
        "dictionary/kanji.json": b"prod",
        "phrases/precomputed/x.bin": b"1234",
    }
    corpus = make_corpus(tmp_path, {"dictionary/kanji.json": b"stale"})
    store = FakeStore(production, manifest_keys=("dictionary/kanji.json",))

    materialize_versioned_files(store, corpus, False)
    materialize_sync_dir(store, corpus, "phrases/precomputed", False)
    store.calls.clear()

    kept, fetched = materialize_versioned_files(store, corpus, False)
    dir_kept, dir_fetched, stale = materialize_sync_dir(
        store, corpus, "phrases/precomputed", False
    )

    assert (kept, fetched, dir_kept, dir_fetched, stale) == (1, 0, 1, 0, [])
    assert store.calls == []
    assert temp_files(corpus) == []


@pytest.mark.parametrize(
    "key",
    ["../outside", "/absolute", "dir\\file", "a/../b", "phrases/data/", "C:/evil.json"],
)
def test_unsafe_remote_key_is_rejected(key: str) -> None:
    with pytest.raises(MaterializationError, match="unsafe CDN key"):
        local_path(Path("/tmp/corpus"), key)
