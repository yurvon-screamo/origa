"""Production reads, path safety and atomic writes for the corpus materializer.

All temporary files live in ``.materialize-tmp/`` at the corpus root. That
directory is outside every ``SYNC_DIR``, so a killed download cannot leave a
partial object where ``deploy_cdn.py`` would upload it on the next deploy.
The next write run removes leftovers.
"""

from __future__ import annotations

import hashlib
import json
import os
import tempfile
from collections.abc import Iterable, Iterator
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Final, Protocol

import _cdn_s3

CHUNK_SIZE: Final = 64 * 1024
DOWNLOAD_WORKERS: Final = 8
PROGRESS_INTERVAL: Final = 100
STAGING_DIR_NAME: Final = ".materialize-tmp"


class MaterializationError(Exception):
    """Production state or a local path cannot be trusted; stop the run."""


class ProductionStore(Protocol):
    """Production reads only — no uploads, no local state."""

    def manifest(self) -> dict[str, object]: ...

    def object_sizes(self, prefix: str) -> dict[str, int]: ...

    def download(self, key: str, target: Path) -> str: ...


class S3ProductionStore:
    """Production via the shared S3 transport (profile/env credential contract)."""

    def manifest(self) -> dict[str, object]:
        manifest = _cdn_s3.download_remote_manifest(False)
        if manifest is None:
            raise MaterializationError("production manifest.json is unavailable")
        return manifest

    def object_sizes(self, prefix: str) -> dict[str, int]:
        objects = _cdn_s3.list_remote_objects(prefix)
        return {key: item.size for key, item in objects.items()}

    def download(self, key: str, target: Path) -> str:
        return _cdn_s3.download_object(key, target)


def local_path(corpus: Path, key: str) -> Path:
    """Reject traversal, directory markers and drive letters from remote keys."""
    parts = Path(key).parts
    if (
        not parts
        or key.startswith("/")
        or key.endswith("/")
        or ":" in key
        or "\\" in key
        or ".." in parts
    ):
        raise MaterializationError(f"unsafe CDN key: {key!r}")
    return corpus.joinpath(*parts)


def staging_dir(corpus: Path) -> Path:
    """Temp files live outside sync dirs: deploy_cdn never uploads them."""
    return corpus / STAGING_DIR_NAME


def clean_staging(corpus: Path) -> None:
    """Remove leftovers from a run that was killed before cleanup."""
    directory = staging_dir(corpus)
    if not directory.is_dir():
        return
    for path in directory.iterdir():
        if path.is_file() and path.name.startswith(".materialize-"):
            path.unlink()


def sha256_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(CHUNK_SIZE):
            hasher.update(chunk)
    return hasher.hexdigest()


def download_to_temp(
    store: ProductionStore, corpus: Path, key: str
) -> tuple[Path, str]:
    """Return a fresh temp file and its digest; the caller owns the file."""
    directory = staging_dir(corpus)
    directory.mkdir(parents=True, exist_ok=True)
    descriptor, name = tempfile.mkstemp(prefix=".materialize-", dir=directory)
    os.close(descriptor)
    temp_path = Path(name)
    try:
        digest = store.download(key, temp_path)
    except BaseException:
        temp_path.unlink(missing_ok=True)
        raise
    return temp_path, digest


def commit_download(temp_path: Path, target: Path) -> None:
    """Move a verified temp file into place, keeping the local file mode."""
    mode = target.stat().st_mode & 0o777 if target.is_file() else 0o644
    temp_path.chmod(mode)
    target.parent.mkdir(parents=True, exist_ok=True)
    os.replace(temp_path, target)


def iter_batch(
    store: ProductionStore, corpus: Path, entries: list[tuple[str, Path]]
) -> Iterator[tuple[str, Path, Path, str]]:
    """Yield (key, target, temp, digest) as downloads finish.

    Closing the iterator early cancels queued downloads without waiting for
    in-flight ones — Ctrl+C or a failed object stops the batch immediately.
    Temp files from in-flight workers stay in staging until the next write
    run; they are outside every sync dir, so no deploy can upload them.
    """
    targets = dict(entries)
    pool = ThreadPoolExecutor(max_workers=DOWNLOAD_WORKERS)
    futures: dict[object, str] = {}
    try:
        futures = {
            pool.submit(download_to_temp, store, corpus, key): key for key in targets
        }
        done = 0
        try:
            for future in as_completed(futures):
                key = futures[future]
                temp_path, digest = future.result()
                done += 1
                if done % PROGRESS_INTERVAL == 0 or done == len(targets):
                    print(f"    downloaded {done}/{len(targets)}", flush=True)
                yield key, targets[key], temp_path, digest
        except BaseException:
            pool.shutdown(wait=False, cancel_futures=True)
            raise
        else:
            pool.shutdown(wait=True)
    finally:
        for future in futures:
            future.cancel()


def commit_batch(
    results: Iterable[tuple[str, Path, Path, str]],
    expected: dict[str, str] | None,
    exact: bool,
) -> tuple[int, int]:
    """Verify and commit each download; returns (kept, fetched).

    A failure keeps earlier commits and stops the rest of the batch — the
    same fail-loud partial-progress contract as the sequential version.
    """
    kept = 0
    fetched = 0
    for key, target, temp_path, digest in results:
        try:
            if expected is not None and digest != expected[key]:
                raise MaterializationError(
                    f"{key}: production sha256 {expected[key]}, downloaded {digest}"
                )
            if exact and target.is_file() and sha256_file(target) == digest:
                kept += 1
                continue
            commit_download(temp_path, target)
            fetched += 1
        finally:
            temp_path.unlink(missing_ok=True)
    return kept, fetched


def write_manifest(corpus: Path, manifest: dict[str, object]) -> None:
    """Keep the local manifest in step with the production target list."""
    payload = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    staging_dir(corpus).mkdir(parents=True, exist_ok=True)
    descriptor, name = tempfile.mkstemp(prefix=".materialize-", dir=staging_dir(corpus))
    os.close(descriptor)
    temp_path = Path(name)
    try:
        temp_path.write_text(payload, encoding="utf-8")
        commit_download(temp_path, corpus / "manifest.json")
    finally:
        temp_path.unlink(missing_ok=True)
