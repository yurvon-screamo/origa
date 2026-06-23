"""One-time refresh of Cache-Control metadata on existing S3 objects.

After the tiered policy (``_cdn_cache``) ships, fresh uploads get the right
header, but objects already in the bucket keep whatever Cache-Control they
were originally given. This script walks every object, compares its current
Cache-Control against the policy, and rewrites the metadata in place via an
``s3api copy-object --metadata-directive REPLACE`` self-copy where they differ.

Only metadata changes — object bytes are copied server-side by S3, never
downloaded. Listing and HEAD always run (even in ``--dry-run``) so the preview
reflects what would actually change; only the copy is gated by ``--dry-run``.

Recovery: the walk is idempotent — ``needs_update`` decides per object from
live metadata, so re-running after a partial failure picks up exactly the
objects still mis-configured. A copy failure on one object is reported and
skipped, not fatal; the run exits non-zero only if any object failed, so a
re-run completes the remainder.

Usage::

    python scripts/refresh_cache_control.py --dry-run   # preview (read-only)
    python scripts/refresh_cache_control.py              # apply
"""

from __future__ import annotations

import argparse
import json
import sys
from typing import NamedTuple

import _cdn_cache
from _cdn_s3 import S3_BUCKET, S3_ENDPOINT, S3_PROFILE, run_aws_raw

# copy-object caps at 5 GiB. Only objects whose Cache-Control changes get
# copied, and those are the small release-updated JSON — but guard anyway so
# a future large object with wrong metadata fails with a clear message
# instead of an opaque T3 error mid-walk.
COPY_OBJECT_MAX_BYTES = 5 * 1024 * 1024 * 1024

# The aws CLI is invoked through ``pwsh -Command`` (see _cdn_s3.run_aws_raw),
# which re-parses argv as a PowerShell script. A key containing ; | & ` $ "
# ' could therefore execute as a command rather than a literal argument. CDN
# keys are always ASCII alphanumerics plus / _ . -, so anything outside that
# set is corruption or an injection attempt and must never reach the shell.
_ALLOWED_KEY_CHARS = frozenset(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789/_.-"
)


class ObjectMetadata(NamedTuple):
    cache_control: str | None
    content_length: int | None


def normalize_cache_control(cc: str | None) -> str:
    """Collapse a Cache-Control header for tolerant comparison.

    S3 stores the header verbatim, but objects set by different tooling over
    time may differ in spacing or case (``max-age=300`` vs ``max-age = 300``).
    Treat such variants as equal so we don't pointlessly rewrite metadata.
    """
    return cc.replace(" ", "").lower() if cc else ""


def needs_update(current_cc: str | None, target_cc: str) -> bool:
    return normalize_cache_control(current_cc) != normalize_cache_control(target_cc)


def is_safe_key(key: str) -> bool:
    return bool(key) and all(ch in _ALLOWED_KEY_CHARS for ch in key)


def filter_safe_keys(keys: list[str]) -> tuple[list[str], list[str]]:
    safe = [k for k in keys if is_safe_key(k)]
    unsafe = [k for k in keys if not is_safe_key(k)]
    return safe, unsafe


def list_all_keys() -> list[str]:
    """List every object key in the bucket, paginating fully.

    Drops keys outside the safe charset (see ``is_safe_key``) with a warning,
    since they can never be passed to the aws CLI safely. Aborts if S3 signals
    truncation without a continuation token — that would silently lose keys.
    """
    raw: list[str] = []
    token: str | None = None
    while True:
        args = [
            "s3api",
            "list-objects-v2",
            "--bucket",
            S3_BUCKET,
            "--profile",
            S3_PROFILE,
            "--endpoint-url",
            S3_ENDPOINT,
        ]
        if token:
            args += ["--continuation-token", token]
        result = run_aws_raw(args)
        if result.returncode != 0:
            print("ERROR: list-objects-v2 failed", file=sys.stderr)
            print(result.stderr, file=sys.stderr)
            sys.exit(1)

        data = json.loads(result.stdout) if result.stdout.strip() else {}
        for obj in data.get("Contents", []):
            key = obj.get("Key")
            if isinstance(key, str):
                raw.append(key)

        if data.get("IsTruncated"):
            token = data.get("NextContinuationToken")
            if not token:
                print(
                    "ERROR: S3 returned IsTruncated without NextContinuationToken; "
                    "cannot paginate safely",
                    file=sys.stderr,
                )
                sys.exit(1)
        else:
            break

    safe, unsafe = filter_safe_keys(raw)
    for key in unsafe:
        print(
            f"  WARNING: dropping unsafe key (not ASCII-safe): {key!r}", file=sys.stderr
        )
    return safe


def head_object(key: str) -> ObjectMetadata | None:
    result = run_aws_raw(
        [
            "s3api",
            "head-object",
            "--bucket",
            S3_BUCKET,
            "--key",
            key,
            "--profile",
            S3_PROFILE,
            "--endpoint-url",
            S3_ENDPOINT,
        ]
    )
    if result.returncode != 0:
        print(f"  WARNING: head-object failed for {key}", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        return None
    data = json.loads(result.stdout) if result.stdout.strip() else {}
    length = data.get("ContentLength")
    return ObjectMetadata(
        cache_control=data.get("CacheControl"),
        content_length=int(length) if isinstance(length, int) else None,
    )


def update_cache_control(key: str, target_cc: str, dry_run: bool) -> bool:
    """Rewrite one object's Cache-Control via a server-side self-copy.

    Returns True if applied (or previewed in dry-run), False if the copy
    failed — the caller continues with the remaining objects rather than
    aborting the whole walk.
    """
    args = [
        "s3api",
        "copy-object",
        "--bucket",
        S3_BUCKET,
        "--key",
        key,
        "--copy-source",
        f"{S3_BUCKET}/{key}",
        "--profile",
        S3_PROFILE,
        "--endpoint-url",
        S3_ENDPOINT,
        "--metadata-directive",
        "REPLACE",
        "--cache-control",
        target_cc,
    ]
    if dry_run:
        return True
    result = run_aws_raw(args)
    if result.returncode != 0:
        print(f"  ERROR: copy-object failed for {key}", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        return False
    return True


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Refresh Cache-Control on existing S3 objects to match "
        "the tiered policy (read-only listing/HEAD always runs).",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would change without rewriting metadata",
    )
    args = parser.parse_args()

    if args.dry_run:
        print("=== DRY RUN ===\n")

    print(f"Listing objects in s3://{S3_BUCKET} ...")
    keys = list_all_keys()
    print(f"  {len(keys)} safe object(s)\n")

    updated = 0
    already_ok = 0
    skipped_oversize = 0
    failed: list[str] = []

    for key in keys:
        meta = head_object(key)
        if meta is None:
            failed.append(key)
            continue

        if (
            meta.content_length is not None
            and meta.content_length > COPY_OBJECT_MAX_BYTES
        ):
            skipped_oversize += 1
            print(
                f"  SKIP   {key}  ({meta.content_length} B exceeds copy-object "
                f"{COPY_OBJECT_MAX_BYTES} B limit; update metadata manually)"
            )
            failed.append(key)
            continue

        target = _cdn_cache.cache_control_for(key)
        if needs_update(meta.cache_control, target):
            current_display = meta.cache_control or "(none)"
            print(f"  UPDATE {key}  [{current_display} -> {target}]")
            if update_cache_control(key, target, args.dry_run):
                updated += 1
            else:
                failed.append(key)
        else:
            already_ok += 1

    print("\nSummary:")
    print(f"  scanned:          {len(keys)}")
    print(f"  updated:          {updated}")
    print(f"  already OK:       {already_ok}")
    print(f"  skipped (> 5 GiB): {skipped_oversize}")
    if failed:
        print(f"  failed:           {len(failed)}")
        for key in failed:
            print(f"    - {key}")
        if args.dry_run:
            print("\n(dry-run: no metadata rewritten)")
        else:
            print("\nRe-run to retry the failed objects (walk is idempotent).")
        sys.exit(1)
    if args.dry_run:
        print("\n(dry-run: no metadata rewritten)")


if __name__ == "__main__":
    main()
