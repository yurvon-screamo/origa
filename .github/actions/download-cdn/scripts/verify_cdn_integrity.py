#!/usr/bin/env python3
"""Verify integrity of a downloaded CDN mirror against its manifests.

Contract used by the ``download-cdn`` composite action (issue #544):

- Presence: every path in the file list must exist under the CDN dir and be
  non-empty. The CDN never serves zero-byte files for file-list entries
  (confirmed by a HEAD probe over all paths on 2026-09-15); a file that is
  missing or empty therefore means an incomplete or corrupted mirror and must
  fail loudly instead of reaching the e2e jobs or poisoning the cache.
- Hashes: every entry of the hash manifest that is also present in the file
  list must match its SHA256. Hash-manifest entries outside the file list are
  reported as ``SKIP`` and are NOT errors: the prod manifest covers
  app-visible content the CI mirror intentionally does not download
  (kanji animations/frames, phrase data bundles).

Output goes to stdout so it is visible in the CI log; callers act only on the
exit code (0 = clean, 1 = at least one problem).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

CHUNK_SIZE = 1 << 20


class UnusableHashManifestError(Exception):
    """The hash manifest cannot be read or lacks the expected structure."""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Verify presence (and optionally SHA256) of a CDN mirror."
    )
    parser.add_argument(
        "--cdn-dir",
        type=Path,
        required=True,
        help="Root of the downloaded CDN mirror.",
    )
    parser.add_argument(
        "--file-list",
        type=Path,
        default=Path("end2end/cdn-manifest.txt"),
        help="Path list; blank lines and #-comments are ignored.",
    )
    parser.add_argument(
        "--hash-manifest",
        type=Path,
        default=None,
        help="manifest.json with a 'files' map of path -> SHA256.",
    )
    return parser.parse_args()


def read_file_list(path: Path) -> set[str]:
    entries: set[str] = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        entries.add(stripped)
    return entries


def check_presence(
    cdn_dir: Path, entries: set[str]
) -> tuple[list[str], set[str]]:
    problems: list[str] = []
    missing: set[str] = set()
    for entry in sorted(entries):
        target = cdn_dir / entry
        if not target.is_file() or target.stat().st_size == 0:
            problems.append(f"MISSING: {target}")
            missing.add(entry)
    return problems, missing


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(CHUNK_SIZE), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_hash_manifest(path: Path) -> dict[str, str]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise UnusableHashManifestError(
            f"hash manifest {path} not found"
        ) from error
    except json.JSONDecodeError as error:
        raise UnusableHashManifestError(
            f"hash manifest {path} is not valid JSON: {error}"
        ) from error
    if not isinstance(payload, dict) or not isinstance(
        payload.get("files"), dict
    ):
        raise UnusableHashManifestError(
            f"hash manifest {path} lacks a 'files' object mapping paths to hashes"
        )
    return payload["files"]


def check_hashes(
    cdn_dir: Path,
    hash_manifest: Path,
    entries: set[str],
    already_missing: set[str],
) -> list[str]:
    try:
        recorded = load_hash_manifest(hash_manifest)
    except UnusableHashManifestError as error:
        return [f"ERROR: {error}"]
    problems: list[str] = []
    for entry, expected in sorted(recorded.items()):
        if entry not in entries:
            print(f"SKIP (not in file list): {entry}")
            continue
        target = cdn_dir / entry
        if entry in already_missing:
            continue
        actual = sha256_of(target)
        if actual != expected:
            problems.append(
                f"STALE: {target} (cached={actual}, expected={expected})"
            )
    return problems


def main() -> int:
    args = parse_args()
    entries = read_file_list(args.file_list)
    if not entries:
        print(f"ERROR: file list {args.file_list} is empty")
        return 1

    problems, missing_entries = check_presence(args.cdn_dir, entries)
    if args.hash_manifest is not None:
        problems.extend(
            check_hashes(
                args.cdn_dir, args.hash_manifest, entries, missing_entries
            )
        )

    for problem in problems:
        print(problem)

    mode = "presence + SHA256" if args.hash_manifest else "presence only"
    if problems:
        print(f"FAILED: {len(problems)} problem(s) across {len(entries)} files ({mode})")
        return 1
    print(f"OK: {len(entries)} files verified ({mode})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
