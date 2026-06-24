"""Refresh Cache-Control metadata on existing S3 objects to match the tiered policy.

After the tiered policy (``_cdn_cache``) ships, fresh uploads get the right
header, but objects already in the bucket keep whatever Cache-Control they
were originally given. This script fetches each target object's metadata,
compares its current Cache-Control against the policy, and rewrites the
metadata in place via an ``s3api copy-object --metadata-directive REPLACE``
self-copy where they differ.

Only metadata changes — object bytes are copied server-side by S3, never
downloaded. HEAD always runs (even in ``--dry-run``) so the preview reflects
what would actually change; only the copy is gated by ``--dry-run``.

Scope
-----
Three scopes select which objects to refresh. The default avoids walking the
bucket's 100k+ truly-static objects (audio, kanji art, ML models), which took
hours and incurred real S3 LIST/HEAD cost when the original implementation
listed the whole bucket:

- **default** — download ``manifest.json`` and refresh only the files it lists
  plus ``manifest.json`` itself (~33 keys). Fast and effectively free.
- ``--prefix PREFIX`` — walk objects under one S3 prefix. Small content
  prefixes (``grammar/``, ``dictionary/``) run without a prompt; a prefix into
  a heavy directory (``kanji_animations/``, ``phrases/audio/``, ...) holds the
  same 100k+ objects as ``--all`` and gets the same cost-guard.
- ``--all`` — walk EVERY object in the bucket. Always prompts for explicit
  confirmation, including under ``--dry-run``: the cost is LIST+HEAD on 100k+
  objects, which ``--dry-run`` does NOT avoid (it only gates the copy).

The manifest tracks release-updated versioned files plus the immutable
lindera ``dictionaries/`` binaries; the latter are HEAD'd as no-ops (already
``immutable``) but cost nothing at ~33 keys. The truly-static bulk that is
never in the manifest — ``kanji_animations/``, ``kanji_frames/``, ``ndlocr/``,
``phrases/audio/``, ``whisper/`` — keeps an immutable Cache-Control that does
not change between releases, so the default scope deliberately skips it.

Recovery: the walk is idempotent — ``needs_update`` decides per object from
live metadata, so re-running after a partial failure picks up exactly the
objects still mis-configured. A failure on one object (HEAD error, oversize,
copy error) is reported and skipped, not fatal; the run exits non-zero only if
any object failed, so a re-run completes the remainder.

S3 transport (list / HEAD / copy-object, key-safety) lives in ``_cdn_s3``;
this module is the scope selector + decision layer.

Usage::

    python scripts/refresh_cache_control.py --dry-run                  # manifest only
    python scripts/refresh_cache_control.py                             # manifest only, apply
    python scripts/refresh_cache_control.py --dry-run --prefix grammar/
    python scripts/refresh_cache_control.py --all                      # prompts even with --dry-run
"""

from __future__ import annotations

import argparse
import sys
from collections import Counter
from typing import NamedTuple

import _cdn_cache
from _cdn_s3 import (
    COPY_OBJECT_MAX_BYTES,
    S3_BUCKET,
    copy_object_cache_control,
    download_remote_manifest,
    filter_safe_keys,
    head_object,
    is_safe_key,
    list_keys,
)

# Prefixes under these directories hold the 100k+ truly-static objects
# (kanji art, audio, ML models). A ``--prefix`` into them is as expensive as
# ``--all``, so it shares the same cost-guard; content prefixes skip it.
_HEAVY_PREFIXES = frozenset(
    {
        "kanji_animations/",
        "kanji_frames/",
        "ndlocr/",
        "phrases/audio/",
        "whisper/",
    }
)


class WalkOutcome(NamedTuple):
    category: str
    failed: bool


class WalkSummary(NamedTuple):
    scanned: int
    counts: Counter[str]
    retriable: list[str]
    oversize: list[str]


def normalize_cache_control(cc: str | None) -> str:
    """Collapse a Cache-Control header for tolerant comparison.

    S3 stores the header verbatim, but objects set by different tooling over
    time may differ in spacing or case (``max-age=300`` vs ``max-age = 300``).
    Treat such variants as equal so we don't pointlessly rewrite metadata.
    """
    return cc.replace(" ", "").lower() if cc else ""


def needs_update(current_cc: str | None, target_cc: str) -> bool:
    return normalize_cache_control(current_cc) != normalize_cache_control(target_cc)


def is_heavy_prefix(prefix: str) -> bool:
    """Whether ``--prefix`` points into a 100k+ truly-static directory."""
    normalized = prefix if prefix.endswith("/") else prefix + "/"
    return any(normalized.startswith(heavy) for heavy in _HEAVY_PREFIXES)


def load_manifest_files() -> tuple[list[str], int]:
    """Default scope: keys listed in remote ``manifest.json`` plus the manifest itself.

    The manifest tracks release-updated versioned files (and the immutable
    ``dictionaries/`` binaries, which HEAD as no-ops). The truly-static bulk —
    kanji art, audio, ML models — lives in SYNC_DIRS and never appears here, so
    this refreshes exactly the files whose Cache-Control policy changed with
    PR #182 at ~33 keys. Downloading the manifest is read-only and runs even
    under ``--dry-run``.
    """
    print("Downloading remote manifest.json ...")
    manifest = download_remote_manifest(dry_run=False)
    if manifest is None:
        print(
            "ERROR: remote manifest.json not found. Run deploy_cdn.py first, "
            "or use --all / --prefix to refresh by listing.",
            file=sys.stderr,
        )
        sys.exit(1)

    files = manifest.get("files") if isinstance(manifest, dict) else None
    if not isinstance(files, dict) or not files:
        print("ERROR: manifest has no 'files' object to enumerate", file=sys.stderr)
        sys.exit(1)

    raw = [*files.keys(), "manifest.json"]
    safe, unsafe = filter_safe_keys(raw)
    for key in unsafe:
        print(f"  WARNING: dropping unsafe key: {key!r}", file=sys.stderr)
    return safe, len(unsafe)


def _walk_one(key: str, dry_run: bool) -> WalkOutcome:
    meta = head_object(key)
    if meta is None:
        return WalkOutcome("head_failed", failed=True)

    if meta.content_length is not None and meta.content_length > COPY_OBJECT_MAX_BYTES:
        print(
            f"  SKIP   {key}  ({meta.content_length} B exceeds copy-object "
            f"{COPY_OBJECT_MAX_BYTES} B limit; update metadata manually)"
        )
        # Not retriable — re-running cannot beat the copy-object 5 GiB cap; the
        # object needs a manual metadata update. Reported separately in main().
        return WalkOutcome("oversize", failed=False)

    target = _cdn_cache.cache_control_for(key)
    if not needs_update(meta.cache_control, target):
        return WalkOutcome("already_ok", failed=False)

    current_display = meta.cache_control or "(none)"
    print(f"  UPDATE {key}  [{current_display} -> {target}]")
    if copy_object_cache_control(key, target, dry_run):
        return WalkOutcome("updated", failed=False)
    return WalkOutcome("copy_failed", failed=True)


def _confirm_expensive_walk(scope_desc: str) -> bool:
    """Gate an expensive LIST+HEAD walk behind explicit confirmation.

    The cost this PR eliminates is LIST+HEAD across 100k+ objects — NOT the
    copy-object, which ``--dry-run`` gates. So confirmation is required for
    ``--all`` (and heavy ``--prefix``) regardless of ``--dry-run``. A missing
    interactive stdin fails closed (abort) rather than crashing.
    """
    print(
        f"WARNING: {scope_desc} will LIST + HEAD every matching S3 object "
        "(~100k+ audio/kanji/model files). This may take hours and incur "
        "significant S3 API costs (dry-run does NOT avoid listing cost).",
        file=sys.stderr,
    )
    try:
        answer = input("Continue? [y/N] ").strip().lower()
    except EOFError:
        print("\nAborted (no interactive stdin).", file=sys.stderr)
        return False
    if answer != "y":
        print("Aborted.", file=sys.stderr)
        return False
    return True


def collect_targets(args: argparse.Namespace) -> tuple[list[str], int]:
    """Resolve which S3 keys to refresh for the selected scope.

    Precedence: ``--all`` > ``--prefix`` > default (manifest files).
    ``--all`` and ``--prefix`` are mutually exclusive at the argparse layer;
    the if-chain here is the dispatch logic and stays defensive. Returns
    ``(safe_keys, unsafe_count)``.
    """
    if args.all:
        print(f"Listing ALL objects in s3://{S3_BUCKET} ...")
        if not _confirm_expensive_walk("--all"):
            sys.exit(0)
        return list_keys(prefix=None)
    if args.prefix:
        print(f"Listing objects under s3://{S3_BUCKET}/{args.prefix} ...")
        if is_heavy_prefix(args.prefix) and not _confirm_expensive_walk(
            f"--prefix {args.prefix}"
        ):
            sys.exit(0)
        return list_keys(prefix=args.prefix)
    return load_manifest_files()


def _run_walk(keys: list[str], dry_run: bool) -> WalkSummary:
    counts: Counter[str] = Counter()
    retriable: list[str] = []
    oversize: list[str] = []
    for key in keys:
        outcome = _walk_one(key, dry_run)
        counts[outcome.category] += 1
        if outcome.category == "oversize":
            oversize.append(key)
        elif outcome.failed:
            retriable.append(key)
    return WalkSummary(
        scanned=len(keys),
        counts=counts,
        retriable=retriable,
        oversize=oversize,
    )


def _print_summary(summary: WalkSummary, dropped: int, dry_run: bool) -> None:
    counts = summary.counts
    print("\nSummary:")
    print(f"  scanned:           {summary.scanned}")
    print(f"  updated:           {counts['updated']}")
    print(f"  already OK:        {counts['already_ok']}")
    print(f"  dropped (unsafe):  {dropped}")

    if summary.retriable:
        print(f"  failed (retriable): {len(summary.retriable)}")
        for key in summary.retriable:
            print(f"    - {key}")
    if summary.oversize:
        print(f"  oversize (> 5 GiB, manual): {len(summary.oversize)}")
        for key in summary.oversize:
            print(f"    - {key}")

    if dry_run:
        print("\n(dry-run: no metadata rewritten)")
    elif summary.retriable:
        print("\nRe-run to retry the failed objects (walk is idempotent).")
    if summary.oversize:
        print(
            "\nOversize objects exceed the 5 GiB copy-object limit and need a "
            "manual metadata update (re-running will not fix them)."
        )


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Refresh Cache-Control on existing S3 objects to match "
        "the tiered policy (HEAD always runs; copy gated by --dry-run).",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would change without rewriting metadata",
    )
    scope = parser.add_mutually_exclusive_group()
    scope.add_argument(
        "--prefix",
        default=None,
        help="Refresh only objects under this S3 prefix (e.g. 'grammar/' or "
        "'phrases/data/'). Faster than --all for a single directory; a prefix "
        "into a heavy dir (kanji_animations/, audio/) prompts like --all.",
    )
    scope.add_argument(
        "--all",
        action="store_true",
        help="Walk ALL S3 objects, not just manifest files. WARNING: with "
        "100k+ audio files this can take hours and incur significant S3 API "
        "costs. Prompts for confirmation (even with --dry-run). Use only when "
        "changing Cache-Control for truly-static files (kanji_animations, "
        "audio, models).",
    )
    return parser


def _validate_prefix(prefix: str) -> None:
    if not is_safe_key(prefix):
        print(
            f"ERROR: --prefix contains unsafe characters: {prefix!r}",
            file=sys.stderr,
        )
        sys.exit(2)


def main() -> None:
    parser = _build_parser()
    args = parser.parse_args()

    if args.prefix is not None:
        _validate_prefix(args.prefix)

    if args.dry_run:
        print("=== DRY RUN ===\n")

    keys, dropped = collect_targets(args)
    print(f"  {len(keys)} target(s), {dropped} dropped (unsafe charset)\n")

    summary = _run_walk(keys, args.dry_run)
    _print_summary(summary, dropped, args.dry_run)

    if summary.retriable or summary.oversize:
        sys.exit(1)


if __name__ == "__main__":
    main()
