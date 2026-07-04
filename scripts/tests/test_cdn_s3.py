"""Unit tests for the boto3 upload path in ``_cdn_s3`` (T3 Storage fix).

T3 Storage drops single-PUT bodies larger than ~24KB; the aws CLI only
auto-multiparts above 8MB, so the 24KB–8MB band (fonts, audio, JSON) failed.
These tests pin the behaviour added to replace it: the multipart
``TransferConfig`` threshold, the content-type resolver, the per-file upload
metadata, and the directory sync diff that avoids re-uploading unchanged
static objects.

boto3 is an external system, so the S3 client and the network-touching
helpers are monkeypatched. The decision logic (which files to upload) is
exercised black-box against a fake remote size map.
"""

from __future__ import annotations

from pathlib import Path

import pytest

import _cdn_s3
from _cdn_s3 import (
    MULTIPART_THRESHOLD_BYTES,
    _TRANSFER_CONFIG,
    content_type_for,
    list_remote_sizes,
    sync_directory,
    upload_file,
)


class _FakeUploadClient:
    """Records boto3 ``upload_file`` calls without touching the network."""

    def __init__(self) -> None:
        self.calls: list[dict[str, object]] = []

    def upload_file(self, **kwargs: object) -> None:
        self.calls.append(kwargs)


class _FakePaginator:
    def __init__(self, pages: list[dict[str, object]]) -> None:
        self._pages = pages

    def paginate(self, **_kwargs: object) -> object:
        return iter(self._pages)


class _FakeListClient:
    """Yields canned list-objects-v2 pages for ``list_remote_sizes``."""

    def __init__(self, pages: list[dict[str, object]]) -> None:
        self._pages = pages

    def get_paginator(self, _name: str) -> _FakePaginator:
        return _FakePaginator(self._pages)


def _record_uploads() -> tuple[list[tuple[Path, str, str, bool]], object]:
    """Return (calls, fake_upload_file) capturing each forwarded upload."""
    calls: list[tuple[Path, str, str, bool]] = []

    def fake(local_path: Path, key: str, cache_control: str, dry_run: bool) -> None:
        calls.append((local_path, key, cache_control, dry_run))

    return calls, fake


def _build_sync_dir(root: Path) -> Path:
    """A sample ``fonts/`` dir: two files, a README, and a nested file."""
    d = root / "fonts"
    d.mkdir()
    (d / "a.woff2").write_bytes(b"AAAA")  # 4 bytes
    (d / "b.json").write_bytes(b"BBBBBBBB")  # 8 bytes
    (d / "README.md").write_text("readme")
    nested = d / "sub"
    nested.mkdir()
    (nested / "c.bin").write_bytes(b"CC")  # 2 bytes
    return d


# ---------------------------------------------------------------------------
# TransferConfig threshold — the actual fix parameter
# ---------------------------------------------------------------------------


def test_multipart_threshold_forces_multipart_under_cli_default():
    # The aws CLI's auto-multipart kicks in at 8MB; we must force it down to
    # 16KB so T3's ~24KB single-PUT limit never applies. A 2MB font (the
    # failing case) is now well above the threshold → multipart.
    assert MULTIPART_THRESHOLD_BYTES == 16 * 1024
    assert _TRANSFER_CONFIG.multipart_threshold == 16 * 1024
    assert _TRANSFER_CONFIG.multipart_chunksize == 16 * 1024
    assert 2 * 1024 * 1024 > MULTIPART_THRESHOLD_BYTES


# ---------------------------------------------------------------------------
# content_type_for
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "filename,expected",
    [
        ("noto-sans-jp-400.woff2", "font/woff2"),
        ("cormorant.woff", "font/woff"),
        ("grammar.json", "application/json"),
        # Override lookup is case-insensitive — real extensions vary in case.
        ("UPPER.WOFF2", "font/woff2"),
    ],
)
def test_content_type_override(filename: str, expected: str):
    assert content_type_for(Path(filename)) == expected


def test_content_type_unknown_falls_back_to_octet_stream():
    # mimetypes cannot guess .onnx → default binary type.
    assert content_type_for(Path("model.onnx")) == "application/octet-stream"


def test_content_type_cjk_filename_resolves():
    # Kanji SVGs use the kanji as the filename (一.svg); suffix lookup must
    # not be confused by the CJK base name.
    assert content_type_for(Path("一.svg")) == "image/svg+xml"


# ---------------------------------------------------------------------------
# upload_file
# ---------------------------------------------------------------------------


def test_upload_file_passes_metadata_and_transfer_config(tmp_path, monkeypatch):
    local = tmp_path / "noto.woff2"
    local.write_bytes(b"x" * 100)
    fake = _FakeUploadClient()
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: fake)

    upload_file(
        local, "fonts/noto.woff2", "public, max-age=1, immutable", dry_run=False
    )

    assert len(fake.calls) == 1
    call = fake.calls[0]
    assert call["Bucket"] == _cdn_s3.S3_BUCKET
    assert call["Key"] == "fonts/noto.woff2"
    assert call["Filename"] == str(local)
    assert call["ExtraArgs"] == {
        "CacheControl": "public, max-age=1, immutable",
        "ContentType": "font/woff2",
    }
    assert call["Config"] is _TRANSFER_CONFIG


def test_upload_file_dry_run_skips_client(tmp_path, monkeypatch, capsys):
    local = tmp_path / "index.json"
    local.write_text("{}")
    fake = _FakeUploadClient()
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: fake)

    upload_file(local, "grammar/grammar.json", "no-cache", dry_run=True)

    assert fake.calls == []
    out = capsys.readouterr().out
    assert "[DRY-RUN]" in out
    assert "grammar/grammar.json" in out
    assert "application/json" in out
    assert "no-cache" in out


# ---------------------------------------------------------------------------
# list_remote_sizes
# ---------------------------------------------------------------------------


def test_list_remote_sizes_paginates_and_normalizes_prefix(monkeypatch):
    pages = [
        {
            "Contents": [
                {"Key": "fonts/a.woff2", "Size": 4},
                {"Key": "fonts/b.json", "Size": 8},
            ]
        },
        {"Contents": [{"Key": "fonts/sub/c.bin", "Size": 2}]},
        {},  # empty trailing page
    ]
    monkeypatch.setattr(_cdn_s3, "_s3_upload_client", lambda: _FakeListClient(pages))

    sizes = list_remote_sizes("fonts")  # no trailing slash → normalized inside

    assert sizes == {
        "fonts/a.woff2": 4,
        "fonts/b.json": 8,
        "fonts/sub/c.bin": 2,
    }


# ---------------------------------------------------------------------------
# sync_directory — diff decision logic
# ---------------------------------------------------------------------------


def test_sync_directory_uploads_new_and_changed_skips_unchanged(tmp_path, monkeypatch):
    local_dir = _build_sync_dir(tmp_path)
    calls, fake_upload = _record_uploads()
    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload)
    # a.woff2 matches size (4) → unchanged; c.bin differs (999 vs 2) → changed;
    # b.json absent remotely → new.
    monkeypatch.setattr(
        _cdn_s3,
        "list_remote_sizes",
        lambda prefix: {"fonts/a.woff2": 4, "fonts/sub/c.bin": 999},
    )

    sync_directory(local_dir, "fonts", "public, immutable", dry_run=False)

    keys = [key for _, key, _, _ in calls]
    assert "fonts/b.json" in keys  # missing remotely
    assert "fonts/sub/c.bin" in keys  # size differs
    assert "fonts/a.woff2" not in keys  # unchanged → skipped
    assert "fonts/README.md" not in keys  # README always skipped
    assert all(cc == "public, immutable" for _, _, cc, _ in calls)
    assert all(dry is False for _, _, _, dry in calls)


def test_sync_directory_dry_run_does_nothing_offline(tmp_path, monkeypatch):
    # The deploy orchestrator prints a per-directory header itself; in dry-run
    # sync_directory must neither walk the (100k+) local tree nor list remote,
    # so a dry-run preview stays instant and offline.
    local_dir = _build_sync_dir(tmp_path)
    calls, fake_upload = _record_uploads()
    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload)

    def fail_list(prefix: str) -> dict[str, int]:
        raise AssertionError("dry-run must not list remote objects")

    monkeypatch.setattr(_cdn_s3, "list_remote_sizes", fail_list)

    sync_directory(local_dir, "fonts", "public, immutable", dry_run=True)

    assert calls == []


def test_sync_directory_uses_normalized_prefix_key(tmp_path, monkeypatch):
    # A prefix with a trailing slash must not double-slash the key.
    local_dir = tmp_path / "d"
    local_dir.mkdir()
    (local_dir / "x.woff2").write_bytes(b"data")
    calls, fake_upload = _record_uploads()
    monkeypatch.setattr(_cdn_s3, "upload_file", fake_upload)
    monkeypatch.setattr(_cdn_s3, "list_remote_sizes", lambda prefix: {})

    sync_directory(local_dir, "fonts/", "immutable", dry_run=False)

    assert [key for _, key, _, _ in calls] == ["fonts/x.woff2"]
