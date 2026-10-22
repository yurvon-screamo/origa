"""Publish the store assets to the CDN and keep the registry in sync.

The store assets used to sit as binaries in ``.media/`` and were uploaded into
the store consoles by hand. They now live in the CDN under a
content-addressed key, and ``docs/media/store-assets.json`` keeps their
address, their hash and their provenance (ADR-068).

Three modes:

- **migration** (no arguments) — publish the assets of every group in
  ``_media_registry.GROUPS``. This is the one-shot move out of ``.media/``.
- **ad-hoc** (``--source``) — publish a freshly re-shot set from the local
  staging directory ``store-assets/``.
- **verify** (``--verify``) — read the committed registry, download every
  asset over HTTP and compare its SHA256. Read-only, needs no AWS
  credentials, and is the gate before anything is deleted from the tree.

The upload itself goes through ``_cdn_s3.upload_file``: the credential
contract (``AWS_ACCESS_KEY_ID`` from the environment wins over the
``origa-t3`` profile) and the multipart threshold that works around the T3
single-PUT limit are inherited from the existing CDN tooling.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import os
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import Final

from _cdn_cache import IMMUTABLE
from _cdn_s3 import is_safe_key, upload_file
from _media_registry import (
    Asset,
    AssetGroup,
    CHANNELS,
    GROUPS,
    KINDS,
    PERSONAL_DATA,
    REGISTRY_RELPATH,
    RegistrySchemaError,
    assets_from_registry,
    build_key,
    locale_for,
    merge_assets,
    png_size,
    read_registry,
    sha256_of,
    write_registry,
)

REPO_ROOT: Final[Path] = Path(__file__).resolve().parent.parent
DEFAULT_CDN_BASE: Final[str] = "https://s3.origa.uwuwu.net"
UI_PATHSPEC: Final[str] = "origa_ui/src/**"
HTTP_TIMEOUT_SECONDS: Final[int] = 60


class UnsafeKeyError(Exception):
    """A derived CDN key contains a character the aws CLI would re-parse."""


def cdn_base() -> str:
    """The public base URL of the CDN, without a trailing slash."""
    return os.environ.get("ORIGA_CDN_BASE_URL", DEFAULT_CDN_BASE).rstrip("/")


def now_iso() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")


# --- git provenance ----------------------------------------------------------


def git(repo: Path, *args: str) -> str:
    """Run a git command without a shell.

    The pathspec is passed as an argument, never through a shell: ``**`` in
    an unquoted shell word expands to the wrong pathspec and silently changes
    the count.
    """
    result = subprocess.run(
        ["git", "-C", str(repo), *args],
        capture_output=True,
        text=True,
        check=True,
        shell=False,
    )
    return result.stdout


def added_at_and_commit(repo: Path, rel_path: str) -> tuple[str, str]:
    """The date the file entered the repository and the commit that did it.

    The date of the capture itself is unknown for these assets — the binaries
    arrived as such — so the registry keeps the commit date under the honest
    name ``addedAt`` and leaves ``capturedAt`` null.
    """
    try:
        output = git(repo, "log", "--diff-filter=A", "--format=%cI|%H", "--", rel_path)
    except subprocess.CalledProcessError:
        return ("", "")
    lines = [line for line in output.splitlines() if line]
    if not lines:
        return ("", "")
    added_at, commit = lines[-1].split("|", 1)
    return (added_at, commit)


def _parse_iso(value: str) -> dt.datetime | None:
    try:
        return dt.datetime.fromisoformat(value)
    except ValueError:
        return None


def drift_label(added_at: str, ui_last_change_at: str) -> str:
    """How far the interface has moved since the asset was captured."""
    start = _parse_iso(added_at)
    end = _parse_iso(ui_last_change_at)
    if start is None or end is None:
        return "unknown"
    days = (end - start).days
    if days > 45:
        return "2-months"
    if days > 30:
        return "1-month"
    return "fresh"


def ui_change(repo: Path, since: str) -> tuple[str, int, str]:
    """When the interface last changed and how many commits touched it."""
    try:
        last = git(repo, "log", "-1", "--format=%cI", "--", UI_PATHSPEC).strip()
    except subprocess.CalledProcessError:
        last = ""
    count = 0
    if since:
        try:
            output = git(
                repo, "log", "--format=%H", f"--since={since}", "--", UI_PATHSPEC
            )
            count = len([line for line in output.splitlines() if line])
        except subprocess.CalledProcessError:
            count = 0
    return (last, count, drift_label(since, last))


# --- collection --------------------------------------------------------------


def build_asset(
    repo: Path, group: AssetGroup, path: Path, now: str
) -> tuple[Asset, Path]:
    """Turn a file into the record the registry stores, plus its local path.

    A file outside the repository (an absolute ``--source``) is published too,
    but it carries no legacy path and no git provenance: it was never in the
    history, so there is nothing to point at.
    """
    try:
        rel_path = path.resolve().relative_to(repo).as_posix()
    except ValueError:
        rel_path = path.resolve().as_posix()
    sha256 = sha256_of(path)
    width, height = png_size(path)
    slug = path.stem
    key = build_key(group.prefix, slug, sha256)
    legacy_path = rel_path if group.legacy else ""
    added_at, commit = added_at_and_commit(repo, rel_path) if group.legacy else ("", "")
    ui_last, ui_commits, ui_drift = ui_change(repo, added_at)

    asset = Asset(
        id=f"{group.prefix}/{slug}",
        kind=group.kind,
        channel=group.channel,
        locale=locale_for(group, path.name),
        slug=slug,
        width=width,
        height=height,
        bytes=path.stat().st_size,
        sha256=sha256,
        key=key,
        url=f"{cdn_base()}/{key}",
        addedAt=added_at,
        sourceCommit=commit,
        uiLastChangeAt=ui_last,
        uiDriftCommits=ui_commits,
        uiDrift=ui_drift,
        personalData=rel_path in PERSONAL_DATA,
        uploadedAt=now,
        uploadedSha256=sha256,
        legacyPath=legacy_path,
    )
    return asset, path


def collect(repo: Path, group: AssetGroup, now: str) -> list[tuple[Asset, Path]]:
    """Every asset of a group, in a stable order."""
    source = Path(group.source)
    if not source.is_absolute():
        source = repo / group.source
    if not source.is_dir():
        return []
    return [
        build_asset(repo, group, path, now)
        for path in sorted(source.glob(group.pattern))
        if path.is_file()
    ]


def adhoc_group(
    source: str,
    prefix: str,
    kind: str,
    channel: str,
    locale: str | None = None,
) -> AssetGroup:
    """The group of a freshly re-shot set published from the staging dir."""
    if kind not in KINDS:
        raise ValueError(f"unknown kind: {kind} (expected one of {sorted(KINDS)})")
    if channel not in CHANNELS:
        raise ValueError(
            f"unknown channel: {channel} (expected one of {sorted(CHANNELS)})"
        )
    return AssetGroup(
        source=source,
        prefix=prefix.rstrip("/"),
        kind=kind,
        channel=channel,
        locale=locale,
        legacy=False,
    )


# --- upload ------------------------------------------------------------------


def preflight(assets: list[Asset]) -> None:
    """Refuse an unsafe key before a single byte leaves the machine.

    Checked up front, not inside the upload loop: a bad asset in the middle
    of the set would otherwise leave its already-uploaded siblings in the CDN
    without a registry entry.
    """
    unsafe = [asset for asset in assets if not is_safe_key(asset.key)]
    if unsafe:
        names = ", ".join(asset.key for asset in unsafe)
        raise UnsafeKeyError(names)


def upload_all(items: list[tuple[Asset, Path]], published: list[Asset]) -> None:
    """Upload every asset; ``published`` grows with each success.

    The caller owns ``published`` so that a failure half-way through still
    leaves a registry with everything that did reach the CDN.
    """
    for asset, path in items:
        upload_file(path, asset.key, IMMUTABLE, False)
        published.append(asset)


def verify(registry_path: Path) -> int:
    """Check every registered asset against the CDN; return the problem count.

    Reads the committed registry from disk and downloads the assets over
    HTTP — an independent check that does not trust the uploading process
    and needs no AWS credentials.
    """
    assets = assets_from_registry(read_registry(registry_path))
    base = cdn_base()
    problems = 0
    for asset in assets:
        url = f"{base}/{asset.key}"
        try:
            with urllib.request.urlopen(url, timeout=HTTP_TIMEOUT_SECONDS) as response:
                payload = response.read()
        except (urllib.error.URLError, OSError) as exc:
            print(f"FAIL {asset.id}: {url} — {exc}")
            problems += 1
            continue
        actual = hashlib.sha256(payload).hexdigest()
        if actual != asset.sha256:
            print(f"FAIL {asset.id}: sha256 {actual[:12]} != {asset.sha256[:12]}")
            problems += 1
        else:
            print(f"ok   {asset.id} {asset.width}x{asset.height} {asset.bytes} B")
    print(f"\n{len(assets) - problems}/{len(assets)} assets verified against {base}")
    return problems


# --- CLI ---------------------------------------------------------------------


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print the plan and upload nothing",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="check the committed registry against the CDN (read-only)",
    )
    parser.add_argument(
        "--source",
        help="publish this directory instead of the migration groups",
    )
    parser.add_argument("--prefix", help="CDN prefix of the ad-hoc group")
    parser.add_argument("--kind", help="kind of the ad-hoc group")
    parser.add_argument("--channel", help="channel of the ad-hoc group")
    parser.add_argument("--locale", help="locale of the ad-hoc group")
    parser.add_argument(
        "--registry",
        default=REGISTRY_RELPATH,
        help=f"registry file (default: {REGISTRY_RELPATH})",
    )
    args = parser.parse_args()

    registry_path = (
        Path(args.registry)
        if Path(args.registry).is_absolute()
        else REPO_ROOT / args.registry
    )
    if args.verify:
        try:
            problems = verify(registry_path)
        except RegistrySchemaError as exc:
            print(f"ERROR: {exc}", file=sys.stderr)
            return 2
        return 1 if problems else 0

    if args.source:
        if not args.prefix or not args.kind or not args.channel:
            parser.error("--source requires --prefix, --kind and --channel")
        try:
            groups = [
                adhoc_group(
                    args.source, args.prefix, args.kind, args.channel, args.locale
                )
            ]
        except ValueError as exc:
            parser.error(str(exc))
    else:
        groups = list(GROUPS)

    now = now_iso()
    items = [item for group in groups for item in collect(REPO_ROOT, group, now)]
    if not items:
        print("nothing to publish")
        return 0
    assets = [asset for asset, _ in items]

    try:
        preflight(assets)
    except UnsafeKeyError as exc:
        print(f"ERROR: refusing an unsafe CDN key: {exc}", file=sys.stderr)
        return 2

    print(f"{len(assets)} assets, {sum(a.bytes for a in assets) // 1024} KiB")
    for asset in assets:
        print(
            f"  {asset.key}  {asset.width}x{asset.height}  personalData={asset.personalData}"
        )

    if args.dry_run:
        # A dry run is a plan, not a state change: the registry records what
        # the CDN holds, and nothing was uploaded.
        print("\ndry run: nothing was uploaded, the registry was not modified")
        return 0

    published: list[Asset] = []
    try:
        upload_all(items, published)
    except SystemExit:
        _write(registry_path, published, now)
        raise

    try:
        _write(registry_path, published, now)
    except RegistrySchemaError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2
    print(f"\n{len(published)} assets published")
    return 0


def _write(registry_path: Path, published: list[Asset], now: str) -> None:
    """Merge the published assets into the registry and write it."""
    existing = assets_from_registry(read_registry(registry_path))
    write_registry(registry_path, merge_assets(existing, published), now)


if __name__ == "__main__":
    sys.exit(main())
