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
import threading
import time
from pathlib import Path

import pytest

import _cdn_materialize_io
import materialize_cdn_corpus
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
    assert (corpus / "phrases/precomputed/一.bin").stat().st_mode & 0o777 == 0o644
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


def test_temp_files_helper_sees_nested_leftovers(tmp_path: Path) -> None:
    corpus = make_corpus(tmp_path, {"dictionary/kanji.json": b"prod"})
    nested = corpus / "dictionary" / ".materialize-nested"
    nested.write_bytes(b"partial")

    assert temp_files(corpus) == [nested]


def test_write_on_verified_corpus_without_staging_dir(tmp_path: Path) -> None:
    corpus = make_corpus(tmp_path, {"dictionary/kanji.json": b"prod"})
    assert not staging_dir(corpus).exists()

    kept, fetched = materialize_versioned_files(
        FakeStore({"dictionary/kanji.json": b"prod"}), corpus, False
    )

    assert (kept, fetched) == (1, 0)
    assert staging_dir(corpus).is_dir()
    assert list(staging_dir(corpus).iterdir()) == []
    assert (corpus / "manifest.json").is_file()


def test_main_cleans_staging_leftovers(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    corpus = make_corpus(tmp_path, {"dictionary/kanji.json": b"prod"})
    store = FakeStore({"dictionary/kanji.json": b"prod"})
    monkeypatch.setattr(
        materialize_cdn_corpus._cdn_corpus, "corpus_dir", lambda: corpus
    )
    monkeypatch.setattr(materialize_cdn_corpus, "S3ProductionStore", lambda: store)
    directory = staging_dir(corpus)
    directory.mkdir()
    leftover = directory / ".materialize-old"
    leftover.write_bytes(b"partial")

    assert materialize_cdn_corpus.main([]) == 0

    assert not leftover.exists()
    assert store.calls == []


def test_failed_batch_keeps_earlier_commits(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(_cdn_materialize_io, "DOWNLOAD_WORKERS", 1)
    production = {f"dictionary/{name}.json": name.encode() for name in "abcd"}
    corpus = make_corpus(tmp_path, {})

    class FailingStore(FakeStore):
        def download(self, key: str, target: Path) -> str:
            if key.endswith("c.json"):
                raise ConnectionError("object failed")
            return super().download(key, target)

    with pytest.raises(ConnectionError):
        materialize_versioned_files(FailingStore(production), corpus, False)

    assert (corpus / "dictionary/a.json").is_file()
    assert (corpus / "dictionary/b.json").is_file()
    assert not (corpus / "dictionary/c.json").exists()
    assert not (corpus / "dictionary/d.json").exists()
    assert all(path.parent == staging_dir(corpus) for path in temp_files(corpus))
    clean_staging(corpus)
    assert temp_files(corpus) == []


def test_main_dry_run_then_write_materializes_partial_corpus(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """The user's whole path through the CLI: preview, write, run again."""
    production = {
        "counters/counters.json": b"counter-v2",
        "dictionary/kanji.json": b"kanji-v1",
        "phrases/data/p0000.json": b"exact-NEW",
        "phrases/data/p0001.json": b"good-data",
        "phrases/precomputed/x.bin": b"AAAA",
        "kanji_animations/\u4e00.svg": b"anim",
        "kanji_frames/\u4e00.svg": b"frame",
    }
    corpus = make_corpus(
        tmp_path,
        {
            "counters/counters.json": b"counter-v1",
            "dictionary/kanji.json": b"kanji-v1",
            "phrases/data/p0000.json": b"exact-OLD",
            "phrases/data/p0001.json": b"good-data",
            "phrases/precomputed/x.bin": b"BBBB",
            "kanji_frames/\u4e00.svg": b"frame",
            "phrases/data/legacy.json": b"legacy",
        },
    )
    store = FakeStore(
        production,
        manifest_keys=("counters/counters.json", "dictionary/kanji.json"),
    )
    monkeypatch.setattr(
        materialize_cdn_corpus._cdn_corpus, "corpus_dir", lambda: corpus
    )
    monkeypatch.setattr(materialize_cdn_corpus, "S3ProductionStore", lambda: store)

    assert materialize_cdn_corpus.main(["--dry-run"]) == 0
    preview = capsys.readouterr().out
    assert "Versioned files: 1 verified, 1 fetched/needed" in preview
    assert "phrases/data: 0 kept, 2 to verify" in preview
    assert "phrases/precomputed: 1 kept, 0 to fetch" in preview
    assert "local-only phrases/data/legacy.json" in preview
    assert store.calls == []
    assert not (corpus / "manifest.json").exists()
    assert (corpus / "counters/counters.json").read_bytes() == b"counter-v1"

    assert materialize_cdn_corpus.main([]) == 0
    capsys.readouterr()
    for key, payload in production.items():
        if key.startswith("phrases/precomputed/"):
            continue  # size-only policy keeps the same-size local bytes
        assert (corpus / key).read_bytes() == payload, key
    assert (corpus / "phrases/precomputed/x.bin").read_bytes() == b"BBBB"
    assert json.loads((corpus / "manifest.json").read_text(encoding="utf-8")) == (
        store.manifest()
    )
    assert set(store.calls) == {
        "counters/counters.json",
        "phrases/data/p0000.json",
        "phrases/data/p0001.json",
        "kanji_animations/\u4e00.svg",
    }
    assert temp_files(corpus) == []
    assert (corpus / "phrases/data/legacy.json").read_bytes() == b"legacy"

    snapshot = {
        path.relative_to(corpus): (path.stat().st_ino, path.stat().st_mtime_ns)
        for path in corpus.rglob("*")
        if path.is_file() and ".materialize-tmp" not in path.parts
    }
    store.calls.clear()

    assert materialize_cdn_corpus.main([]) == 0
    report = capsys.readouterr().out
    assert "Versioned files: 2 verified, 0 fetched/needed" in report
    assert "phrases/data: 2 kept, 0 to fetch" in report
    assert set(store.calls) == {
        "phrases/data/p0000.json",
        "phrases/data/p0001.json",
    }
    after = {
        path.relative_to(corpus): (path.stat().st_ino, path.stat().st_mtime_ns)
        for path in corpus.rglob("*")
        if path.is_file() and ".materialize-tmp" not in path.parts
    }
    assert {
        key: value for key, value in after.items() if key != Path("manifest.json")
    } == {key: value for key, value in snapshot.items() if key != Path("manifest.json")}


def test_early_exit_cancels_queued_downloads(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Ctrl+C on one object must not start the rest of the queued batch."""
    monkeypatch.setattr(_cdn_materialize_io, "DOWNLOAD_WORKERS", 3)
    production = {f"dictionary/{name}.json": name.encode() for name in "abcdefghijkl"}
    corpus = make_corpus(tmp_path, {})
    a_started, b_started, c_started = (threading.Event() for _ in "abc")
    release_a, release_b, release_c, release_queued = (
        threading.Event() for _ in "abcd"
    )
    a_finished, c_finished = threading.Event(), threading.Event()
    queued_calls: list[str] = []

    class BlockingStore(FakeStore):
        def download(self, key: str, target: Path) -> str:
            if key.endswith("a.json"):
                a_started.set()
                assert release_a.wait(10)
                a_finished.set()
            elif key.endswith("b.json"):
                b_started.set()
                assert release_b.wait(10)
                raise KeyboardInterrupt
            elif key.endswith("c.json"):
                c_started.set()
                assert release_c.wait(10)
                c_finished.set()
            else:
                # A queued task that does start stays busy, so one worker
                # cannot start the whole remaining queue before cancellation.
                queued_calls.append(key)
                assert release_queued.wait(10)
            return self.files[key]

    failure: list[BaseException] = []

    def run() -> None:
        try:
            materialize_versioned_files(BlockingStore(production), corpus, False)
        except BaseException as exc:
            failure.append(exc)

    worker = threading.Thread(target=run, daemon=True)
    worker.start()
    assert all(event.wait(5) for event in (a_started, b_started, c_started))
    release_b.set()
    worker.join(5)
    assert not worker.is_alive()
    assert failure and isinstance(failure[0], KeyboardInterrupt)
    assert len(queued_calls) <= 1

    # a and c were still busy when cancellation fired; once they finish,
    # the rest of the queue must stay cancelled.
    release_a.set()
    release_c.set()
    release_queued.set()
    assert a_finished.wait(5) and c_finished.wait(5)
    # A worker that had already dequeued one task may finish it, but the
    # rest of the queue must stay cancelled (without cancellation all nine
    # tasks would start here).
    deadline = time.monotonic() + 1
    while len(queued_calls) <= 2 and time.monotonic() < deadline:
        time.sleep(0.01)
    assert len(queued_calls) <= 2
    assert all(path.parent == staging_dir(corpus) for path in temp_files(corpus))


def test_batch_reports_progress_every_interval(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    production = {f"dictionary/{index:03}.json": b"x" for index in range(101)}
    corpus = make_corpus(tmp_path, {})

    materialize_versioned_files(FakeStore(production), corpus, False)

    report = capsys.readouterr().out
    assert "downloaded 100/101" in report
    assert "downloaded 101/101" in report
    assert report.index("downloaded 100/101") < report.index("downloaded 101/101")


def test_main_reports_failure_and_returns_error_code(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """A failed materialization must tell the agent/CI via stderr and exit code."""
    corpus = make_corpus(tmp_path, {})

    class BrokenStore(FakeStore):
        def manifest(self) -> dict[str, object]:
            raise ConnectionError("production unavailable")

    monkeypatch.setattr(
        materialize_cdn_corpus._cdn_corpus, "corpus_dir", lambda: corpus
    )
    monkeypatch.setattr(
        materialize_cdn_corpus, "S3ProductionStore", lambda: BrokenStore({})
    )

    assert materialize_cdn_corpus.main([]) == 1
    captured = capsys.readouterr()
    assert "ERROR: production unavailable" in captured.err
    assert "Done." not in captured.out
    assert not (corpus / "manifest.json").exists()
