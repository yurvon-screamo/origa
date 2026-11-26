"""Bring a partial local CDN corpus up to production without a full seed.

Production ``manifest.json`` is the target list: every versioned file is
SHA-checked against it and downloaded only when missing or stale. Downloads
land in a staging directory at the corpus root — outside every sync dir, so
an interrupted run cannot leak a partial object into the next deploy — and
are moved into place only after the hash check. The next write run removes
leftovers from an interrupted run.

Sync directories come from a separate origin listing (they are not in the
manifest). ``phrases/data`` is compared byte-for-byte on every write run;
the other three dirs trust a matching byte size (the partial-seed policy in
the runbook; ``deploy_cdn.py`` classifies ``phrases/precomputed`` as
release-updated, so this is a deliberate tradeoff, not a shared rule).
Local-only files are reported, never deleted: this is a materializer, not a
cleanup tool. Downloads run in a small bounded pool because the S3 endpoint
charges ~250 ms of round-trip per object.

Usage::

    uv run --project scripts python scripts/materialize_cdn_corpus.py --dry-run
    uv run --project scripts python scripts/materialize_cdn_corpus.py

The corpus root is ``$ORIGA_CDN_DIR`` or ``~/origa-cdn`` (one seed per
machine), resolved by ``_cdn_corpus`` exactly like every other consumer.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Final

import _cdn_corpus
from _cdn_materialize_io import (
    MaterializationError,
    ProductionStore,
    S3ProductionStore,
    clean_staging,
    commit_batch,
    fetch_batch,
    local_path,
    sha256_file,
    write_manifest,
)

DEFAULT_SYNC_DIRS: Final = (
    "phrases/data",
    "phrases/precomputed",
    "kanji_animations",
    "kanji_frames",
)
EXACT_SYNC_DIRS: Final = frozenset({"phrases/data"})


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
    pending: list[tuple[str, Path]] = []
    for key, expected in sorted(files.items()):
        target = local_path(corpus, key)
        if target.is_file() and sha256_file(target) == expected:
            kept += 1
        else:
            pending.append((key, target))

    fetched = len(pending) if dry_run else 0
    if not dry_run and pending:
        print(f"  downloading {len(pending)} versioned file(s)", flush=True)
        results = fetch_batch(store, corpus, pending)
        _, fetched = commit_batch(results, expected=files, exact=False)

    if not dry_run:
        write_manifest(corpus, manifest)
    return kept, fetched


def materialize_sync_dir(
    store: ProductionStore, corpus: Path, prefix: str, dry_run: bool
) -> tuple[int, int, list[str]]:
    """Reconcile one sync dir with production. Returns (kept, fetched, stale)."""
    prefix = prefix.strip("/")
    if not prefix:
        raise MaterializationError("empty sync dir prefix")
    exact = prefix in EXACT_SYNC_DIRS
    base = prefix + "/"
    objects = store.object_sizes(prefix)
    kept = 0
    pending: list[tuple[str, Path]] = []

    for key, size in sorted(objects.items()):
        if not key.startswith(base):
            raise MaterializationError(f"object {key!r} outside prefix {prefix!r}")
        target = local_path(corpus, key)
        local_size = target.stat().st_size if target.is_file() else None
        if local_size == size and not exact:
            kept += 1
        else:
            pending.append((key, target))

    fetched = len(pending) if dry_run else 0
    if not dry_run and pending:
        print(f"  downloading {len(pending)} object(s) in {prefix}/", flush=True)
        results = fetch_batch(store, corpus, pending)
        batch_kept, fetched = commit_batch(results, expected=None, exact=exact)
        kept += batch_kept

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
    print(f"Corpus: {corpus} ({mode})", flush=True)

    try:
        if not args.dry_run:
            clean_staging(corpus)
        kept, fetched = materialize_versioned_files(store, corpus, args.dry_run)
        print(f"Versioned files: {kept} verified, {fetched} fetched/needed", flush=True)
        for prefix in sync_dirs:
            dir_kept, dir_fetched, stale = materialize_sync_dir(
                store, corpus, prefix, args.dry_run
            )
            label = (
                "to verify"
                if args.dry_run and prefix.strip("/") in EXACT_SYNC_DIRS
                else "to fetch"
            )
            print(f"{prefix}: {dir_kept} kept, {dir_fetched} {label}", flush=True)
            for key in stale:
                print(
                    f"  WARNING: local-only {key} (not in production; left in place)",
                    flush=True,
                )
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr, flush=True)
        return 1

    print("Done.", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
