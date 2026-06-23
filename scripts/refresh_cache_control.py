"""One-time refresh of Cache-Control metadata on existing S3 objects.

After the tiered policy (``_cdn_cache``) ships, fresh uploads get the right
header, but objects already in the bucket keep whatever Cache-Control they
were originally given. This script walks every object, compares its current
Cache-Control against the policy, and rewrites the metadata in place via an
``s3api copy-object --metadata-directive REPLACE`` self-copy where they differ.

Only metadata changes — object bytes are copied server-side by S3, never
downloaded. Read-only listing/HEAD always runs (even in ``--dry-run``) so the
preview reflects what would actually change.

Usage::

    python scripts/refresh_cache_control.py --dry-run   # preview (read-only)
    python scripts/refresh_cache_control.py              # apply
"""

from __future__ import annotations

import argparse
import json
import sys

import _cdn_cache
from _cdn_s3 import S3_BUCKET, S3_ENDPOINT, S3_PROFILE, run_aws, run_aws_raw


def normalize_cc(cc: str | None) -> str:
    """Collapse a Cache-Control header for tolerant comparison.

    S3 stores the header verbatim, but objects set by different tooling over
    time may differ in spacing or case (``max-age=300`` vs ``max-age = 300``).
    Treat such variants as equal so we don't pointlessly rewrite metadata.
    """
    return cc.replace(" ", "").lower() if cc else ""


def needs_update(current_cc: str | None, target_cc: str) -> bool:
    return normalize_cc(current_cc) != normalize_cc(target_cc)


def list_all_keys() -> list[str]:
    keys: list[str] = []
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
                keys.append(key)

        if data.get("IsTruncated") and data.get("NextContinuationToken"):
            token = data["NextContinuationToken"]
        else:
            break

    return keys


def head_cache_control(key: str) -> str | None:
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
    return data.get("CacheControl")


def update_cache_control(key: str, target_cc: str, dry_run: bool) -> None:
    # Server-side self-copy with metadata replacement; object bytes never
    # traverse the client. copy-object caps at 5 GB, but only objects whose
    # Cache-Control actually changes get copied — and those are the small
    # release-updated JSON, never the multi-GB ML models (which stay immutable).
    run_aws(
        [
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
        ],
        dry_run,
    )


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
    print(f"  {len(keys)} object(s)\n")

    updated = 0
    already_ok = 0

    for key in keys:
        target = _cdn_cache.cache_control_for(key)
        current = head_cache_control(key)
        if needs_update(current, target):
            updated += 1
            current_display = current or "(none)"
            print(f"  UPDATE  {key}  [{current_display} -> {target}]")
            update_cache_control(key, target, args.dry_run)
        else:
            already_ok += 1

    print("\nSummary:")
    print(f"  scanned:    {len(keys)}")
    print(f"  updated:    {updated}")
    print(f"  already OK: {already_ok}")
    if args.dry_run:
        print("\n(dry-run: no metadata rewritten)")


if __name__ == "__main__":
    main()
