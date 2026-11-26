"""Bring a partial local CDN corpus up to production without a full seed.

Production ``manifest.json`` is the target list: every versioned file is
SHA-checked against it and downloaded only when missing or stale. Downloads
land in a temporary file in the target directory and are moved into place
only after the hash check, so a failed run never corrupts a good seed.

Sync directories come from a separate origin listing (they are not in the
manifest). ``phrases/data`` is compared byte-for-byte; immutable seed trees
(``kanji_animations``, ``kanji_frames``, ``phrases/precomputed``) trust a
matching byte size, as ``deploy_cdn.py`` does. Local-only files are reported,
never deleted: this is a materializer, not a cleanup tool.

Usage::

    uv run --project scripts python scripts/materialize_cdn_corpus.py --dry-run
    uv run --project scripts python scripts/materialize_cdn_corpus.py

The corpus root is ``$ORIGA_CDN_DIR`` or ``~/origa-cdn`` (one seed per
machine), resolved by ``_cdn_corpus`` exactly like every other consumer.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import tempfile
from pathlib import Path
from typing import Final, Protocol

import _cdn_corpus
import _cdn_s3

CHUNK_SIZE: Final = 64 * 1024
DEFAULT_SYNC_DIRS: Final = (
    "phrases/data",
    "phrases/precomputed",
    "kanji_animations",
    "kanji_frames",
)
EXACT_SYNC_DIRS: Final = frozenset({"phrases/data"})


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
    """Reject traversal from a remote key before touching the local tree."""
    parts = Path(key).parts
    if not parts or key.startswith("/") or "\\" in key or ".." in parts:
        raise MaterializationError(f"unsafe CDN key: {key!r}")
    return corpus.joinpath(*parts)


def sha256_file(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(CHUNK_SIZE):
            hasher.update(chunk)
    return hasher.hexdigest()


def download_to_temp(
    store: ProductionStore, key: str, directory: Path
) -> tuple[Path, str]:
    """Return a fresh temp file and its digest; the caller owns the file."""
    directory.mkdir(parents=True, exist_ok=True)
    descriptor, name = tempfile.mkstemp(prefix=".materialize-", dir=directory)
    os.close(descriptor)
    temp_path = Path(name)
    try:
        digest = store.download(key, temp_path)
    except Exception:
        temp_path.unlink(missing_ok=True)
        raise
    return temp_path, digest


def write_manifest(corpus: Path, manifest: dict[str, object]) -> None:
    """Keep the local manifest in step with the production target list."""
    payload = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    descriptor, name = tempfile.mkstemp(prefix=".materialize-", dir=corpus)
    os.close(descriptor)
    temp_path = Path(name)
    try:
        temp_path.write_text(payload, encoding="utf-8")
        os.replace(temp_path, corpus / "manifest.json")
    finally:
        temp_path.unlink(missing_ok=True)


def materialize_versioned_files(
    store: ProductionStore, corpus: Path, dry_run: bool
) -> tuple[int, int]:
    """Verify each manifest file; fetch only mismatches. Returns (kept, fetched)."""
    manifest = store.manifest()
    files = manifest.get("files")
    if (
        not isinstance(files, dict)
        or not files
        or not all(
            isinstance(key, str) and isinstance(digest, str)
            for key, digest in files.items()
        )
    ):
        raise MaterializationError("production manifest has no valid 'files' map")

    kept = 0
    fetched = 0
    for key, expected in sorted(files.items()):
        target = local_path(corpus, key)
        if target.is_file() and sha256_file(target) == expected:
            kept += 1
            continue
        if dry_run:
            fetched += 1
            continue
        print(f"  download {key}")
        temp_path, digest = download_to_temp(store, key, target.parent)
        try:
            if digest != expected:
                raise MaterializationError(
                    f"{key}: production sha256 {expected}, downloaded {digest}"
                )
            os.replace(temp_path, target)
            fetched += 1
        finally:
            temp_path.unlink(missing_ok=True)

    if not dry_run:
        write_manifest(corpus, manifest)
    return kept, fetched


def materialize_sync_dir(
    store: ProductionStore, corpus: Path, prefix: str, dry_run: bool
) -> tuple[int, int, list[str]]:
    """Reconcile one sync dir with production. Returns (kept, fetched, stale)."""
    exact = prefix in EXACT_SYNC_DIRS
    base = prefix.rstrip("/") + "/"
    objects = store.object_sizes(prefix)
    kept = 0
    fetched = 0

    for key, size in sorted(objects.items()):
        if not key.startswith(base):
            raise MaterializationError(f"object {key!r} outside prefix {prefix!r}")
        target = local_path(corpus, key)
        local_size = target.stat().st_size if target.is_file() else None
        if local_size == size and not exact:
            kept += 1
            continue
        if dry_run:
            fetched += 1
            continue
        print(f"  download {key}")
        temp_path, digest = download_to_temp(store, key, target.parent)
        try:
            if exact and target.is_file() and sha256_file(target) == digest:
                kept += 1
                continue
            os.replace(temp_path, target)
            fetched += 1
        finally:
            temp_path.unlink(missing_ok=True)

    local_dir = corpus / prefix
    stale: list[str] = []
    if local_dir.is_dir():
        stale = sorted(
            path.relative_to(corpus).as_posix()
            for path in local_dir.rglob("*")
            if path.is_file()
            and path.name != "README.md"
            and path.relative_to(corpus).as_posix() not in objects
        )
    return kept, fetched, stale


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--dry-run", action="store_true", help="report without writing to the corpus"
    )
    parser.add_argument(
        "--sync-dir",
        action="append",
        help="sync dir to reconcile (repeatable; defaults to the partial-seed set)",
    )
    args = parser.parse_args(argv)

    corpus = _cdn_corpus.corpus_dir()
    store = S3ProductionStore()
    sync_dirs = args.sync_dir or list(DEFAULT_SYNC_DIRS)
    mode = "DRY-RUN" if args.dry_run else "WRITE"
    print(f"Corpus: {corpus} ({mode})")

    try:
        kept, fetched = materialize_versioned_files(store, corpus, args.dry_run)
        print(f"Versioned files: {kept} verified, {fetched} fetched/needed")
        for prefix in sync_dirs:
            dir_kept, dir_fetched, stale = materialize_sync_dir(
                store, corpus, prefix, args.dry_run
            )
            print(f"{prefix}: {dir_kept} kept, {dir_fetched} to fetch")
            for key in stale:
                print(f"  WARNING: local-only {key} (not in production; left in place)")
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    print("Done.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
