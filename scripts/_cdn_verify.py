"""Read-only CDN verification: download files via HTTP and compare their
actual SHA256 against the manifest hashes.

Used by ``deploy_cdn.py --verify-remote`` to diagnose CDN/S3 inconsistencies
where the manifest is current but the underlying object is stale (e.g. partial
deploy, immutable-cache poisoning). Reads from the public CDN base URL
(``ORIGA_CDN_BASE_URL``) rather than S3 directly so it observes what clients
actually see.

Read-only by design: never uploads, never requires AWS credentials. The
top-level ``verify_remote`` returns a problem count (or ``MANIFEST_ERROR``
sentinel) and never calls ``sys.exit`` — the CLI entry point in
``deploy_cdn.py`` owns the exit code so the helper stays callable from tests
and other tooling.
"""

from __future__ import annotations

import hashlib
import json
import os
import sys
import urllib.error
import urllib.request
from typing import cast

CHUNK_SIZE = 8192
HTTP_TIMEOUT_SECONDS = 60
USER_AGENT = "origa-deploy-cdn/1.0"

# Report layout. Width of the "file" column: longest versioned path is
# "well_known_set/well_known_types_meta.json" (41 chars); 50 keeps alignment
# readable without forcing a wide terminal. Hash columns show a 10-char prefix
# plus a 2-char guard band for the "manifest"/"actual" headers.
FILE_COLUMN_WIDTH = 50
HASH_COLUMN_WIDTH = 12
STATUS_HEADER = "status"

# Returned by ``verify_remote`` when the manifest itself cannot be fetched or
# parsed — there is nothing meaningful to compare against. Distinct from a
# per-file mismatch count so callers can tell "CDN manifest broken" from
# "CDN serving a wrong object".
MANIFEST_ERROR = -1


def get_cdn_base_url() -> str | None:
    return os.environ.get("ORIGA_CDN_BASE_URL")


def _cdn_url(cdn_base_url: str, relative_path: str) -> str:
    return f"{cdn_base_url.rstrip('/')}/{relative_path}"


def download_manifest(cdn_base_url: str) -> dict[str, object] | None:
    url = _cdn_url(cdn_base_url, "manifest.json")
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(request, timeout=HTTP_TIMEOUT_SECONDS) as response:
            raw = response.read()
    except (urllib.error.URLError, OSError) as exc:
        print(f"  ERROR: {url}: {exc}", file=sys.stderr)
        return None
    try:
        return cast("dict[str, object]", json.loads(raw))
    except json.JSONDecodeError as exc:
        print(f"  ERROR: manifest.json is not valid JSON: {exc}", file=sys.stderr)
        return None


def compute_remote_hash(cdn_base_url: str, relative_path: str) -> str | None:
    """Stream file from CDN and compute SHA256 incrementally.

    Streaming (rather than materializing the whole body) keeps peak memory
    bounded for the multi-megabyte dictionary chunks. Returns hex digest, or
    None if the download failed.
    """
    url = _cdn_url(cdn_base_url, relative_path)
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(request, timeout=HTTP_TIMEOUT_SECONDS) as response:
            hasher = hashlib.sha256()
            while chunk := response.read(CHUNK_SIZE):
                hasher.update(chunk)
            return hasher.hexdigest()
    except (urllib.error.URLError, OSError) as exc:
        print(f"  ERROR: {url}: {exc}", file=sys.stderr)
        return None


def _separator_width() -> int:
    # "file" + space + "manifest" col + space + "actual" col + space + status.
    return (
        FILE_COLUMN_WIDTH
        + 1
        + HASH_COLUMN_WIDTH
        + 1
        + HASH_COLUMN_WIDTH
        + 1
        + len(STATUS_HEADER)
    )


def verify_remote(cdn_base_url: str) -> int:
    """Download CDN manifest + each versioned file, compare actual hash.

    Returns the number of problems (hash mismatches + fetch failures); 0 means
    every file the manifest references matches what the CDN is actually
    serving. Returns ``MANIFEST_ERROR`` (-1) if the manifest itself cannot be
    downloaded or is malformed. Never calls ``sys.exit`` — the caller owns the
    exit code.
    """
    print(f"Fetching manifest from {cdn_base_url} ...")
    manifest = download_manifest(cdn_base_url)
    if manifest is None:
        print("ERROR: could not download manifest.json from CDN", file=sys.stderr)
        return MANIFEST_ERROR

    files = manifest.get("files")
    if not isinstance(files, dict):
        print("ERROR: manifest 'files' is not a JSON object", file=sys.stderr)
        return MANIFEST_ERROR

    file_items = sorted(files.items())
    print(f"\nVerifying {len(file_items)} file(s) listed in manifest:\n")
    print(
        f"{'file':<{FILE_COLUMN_WIDTH}} {'manifest':<{HASH_COLUMN_WIDTH}} "
        f"{'actual':<{HASH_COLUMN_WIDTH}} {STATUS_HEADER}"
    )
    print("-" * _separator_width())

    mismatches = 0
    fetch_failures = 0
    skipped = 0
    oks = 0

    for path, expected_hash in file_items:
        if not isinstance(path, str) or not isinstance(expected_hash, str):
            skipped += 1
            continue
        actual_hash = compute_remote_hash(cdn_base_url, path)
        expected_display = expected_hash[:10]
        if actual_hash is None:
            fetch_failures += 1
            print(
                f"{path:<{FILE_COLUMN_WIDTH}} {expected_display:<{HASH_COLUMN_WIDTH}} "
                f"{'--':<{HASH_COLUMN_WIDTH}} FETCH-FAIL"
            )
        elif actual_hash == expected_hash:
            oks += 1
            print(
                f"{path:<{FILE_COLUMN_WIDTH}} {expected_display:<{HASH_COLUMN_WIDTH}} "
                f"{actual_hash[:10]:<{HASH_COLUMN_WIDTH}} OK"
            )
        else:
            mismatches += 1
            print(
                f"{path:<{FILE_COLUMN_WIDTH}} {expected_display:<{HASH_COLUMN_WIDTH}} "
                f"{actual_hash[:10]:<{HASH_COLUMN_WIDTH}} MISMATCH"
            )

    print()
    print(
        f"Total: {len(file_items)} file(s), "
        f"{oks} OK, {mismatches} hash mismatch(es), "
        f"{fetch_failures} fetch failure(s), {skipped} skipped"
    )
    return mismatches + fetch_failures + skipped
