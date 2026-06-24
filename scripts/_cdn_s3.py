"""S3 CLI wrappers shared by ``deploy_cdn.py``.

Centralises ``aws`` CLI invocation (shelled out via ``pwsh`` on Windows because
that is how the operator's PowerShell environment resolves the AWS wrapper on
the deployment host) so that ``deploy_cdn.py`` and
``refresh_cache_control.py`` stay orchestrators rather than a transport layer.
All shared S3 paths route through here: upload, sync, manifest download.
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import NamedTuple

S3_BUCKET = "adaptable-foodbox-ucep7wx"
S3_PROFILE = "origa"
S3_ENDPOINT = "https://t3.storageapi.dev"

# copy-object caps at 5 GiB; surfaced so callers can skip oversize objects with
# a clear message instead of an opaque T3 error mid-walk.
COPY_OBJECT_MAX_BYTES = 5 * 1024 * 1024 * 1024

# The aws CLI is invoked through ``pwsh -Command`` (see run_aws_raw), which
# re-parses argv as a PowerShell script. A key containing a PowerShell
# metacharacter could therefore execute as a command/statement rather than a
# literal argument. This is a DENYLIST, not an ASCII allowlist: CJK and other
# Unicode letters are not command separators and pass through safely, which
# matters because kanji_animations/ uses the kanji themselves as filenames
# (一.svg, 丁.svg, ...).
_UNSAFE_KEY_CHARS = frozenset(" \t\r\n;|&`$\"'<>()@\\")


class ObjectMetadata(NamedTuple):
    cache_control: str | None
    content_length: int | None


def s3_uri(key: str) -> str:
    return f"s3://{S3_BUCKET}/{key}"


def run_aws_raw(args: list[str]) -> subprocess.CompletedProcess[str]:
    cmd = ["pwsh", "-Command", "aws", *args]
    try:
        return subprocess.run(
            cmd,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError:
        print("ERROR: 'aws' CLI not found.", file=sys.stderr)
        sys.exit(1)


def run_aws(args: list[str], dry_run: bool) -> subprocess.CompletedProcess[str]:
    if dry_run:
        print(f"  [DRY-RUN] aws {' '.join(args)}")
        return subprocess.CompletedProcess(args, 0, "", "")

    result = run_aws_raw(args)
    if result.returncode != 0:
        print(f"ERROR: aws {' '.join(args)}", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)
    return result


def download_remote_manifest(dry_run: bool) -> dict[str, object] | None:
    # delete=False because the aws CLI runs as a separate process and re-opens
    # the path by name on Windows; cleanup happens in the ``finally`` below so
    # the temp file cannot leak even on dry-run / error paths.
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:
        if dry_run:
            print("  [DRY-RUN] would download remote manifest")
            return None

        result = run_aws_raw(
            [
                "s3",
                "cp",
                s3_uri("manifest.json"),
                str(tmp_path),
                "--profile",
                S3_PROFILE,
                "--endpoint-url",
                S3_ENDPOINT,
            ]
        )

        if result.returncode != 0:
            if "404" in result.stderr or "NoSuchKey" in result.stderr:
                print("  Remote manifest not found (first deployment)")
                return None
            print("ERROR: failed to download remote manifest", file=sys.stderr)
            print(result.stderr, file=sys.stderr)
            sys.exit(1)

        content = tmp_path.read_text(encoding="utf-8")
        return json.loads(content)
    finally:
        tmp_path.unlink(missing_ok=True)


def is_safe_key(key: str) -> bool:
    return bool(key) and not any(ch in _UNSAFE_KEY_CHARS for ch in key)


def filter_safe_keys(keys: list[str]) -> tuple[list[str], list[str]]:
    safe = [k for k in keys if is_safe_key(k)]
    unsafe = [k for k in keys if not is_safe_key(k)]
    return safe, unsafe


def _list_key_page(
    token: str | None, prefix: str | None = None
) -> tuple[list[str], str | None, bool]:
    """Fetch one list-objects-v2 page.

    Returns the page's keys, the continuation token for the next page (None if
    not truncated), and whether more pages remain.
    """
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
    if prefix:
        args += ["--prefix", prefix]
    if token:
        args += ["--continuation-token", token]
    result = run_aws_raw(args)
    if result.returncode != 0:
        print("ERROR: list-objects-v2 failed", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    data = json.loads(result.stdout) if result.stdout.strip() else {}
    keys = [
        obj["Key"]
        for obj in data.get("Contents", [])
        if isinstance(obj.get("Key"), str)
    ]
    return keys, data.get("NextContinuationToken"), bool(data.get("IsTruncated"))


def list_keys(prefix: str | None = None) -> tuple[list[str], int]:
    """List object keys (optionally under ``prefix``), paginating fully.

    Returns the safe keys plus a count of keys dropped as unsafe (shell
    metacharacters). Aborts if S3 signals truncation without a continuation
    token — that would silently lose keys.
    """
    raw: list[str] = []
    token: str | None = None
    while True:
        keys, next_token, truncated = _list_key_page(token, prefix)
        raw.extend(keys)
        if not truncated:
            break
        if not next_token:
            print(
                "ERROR: S3 returned IsTruncated without NextContinuationToken; "
                "cannot paginate safely",
                file=sys.stderr,
            )
            sys.exit(1)
        token = next_token

    safe, unsafe = filter_safe_keys(raw)
    for key in unsafe:
        print(f"  WARNING: dropping unsafe key: {key!r}", file=sys.stderr)
    return safe, len(unsafe)


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


def copy_object_cache_control(key: str, target_cc: str, dry_run: bool) -> bool:
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
