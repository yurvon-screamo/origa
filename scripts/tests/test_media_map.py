"""Guards for the media map (``docs/media/README.md``).

The map replaced a hand-written inventory that had rotted in place: it pointed
at a file that had been renamed, it did not know about two zones of the
repository, and it described sets as parallel that were not. These tests keep
the replacement honest — a zone without a consumer, a path that does not
exist, or a media directory that the map does not know about fail the build.

The tests are pure: they read the map, the registry and the working tree, and
run git read-only.
"""

from __future__ import annotations

import glob
import re
import subprocess
from pathlib import Path

import pytest

from _media_registry import GROUPS, assets_from_registry, read_registry

REPO_ROOT = Path(__file__).resolve().parents[2]
MAP_RELPATH = Path("docs/media/README.md")
REGISTRY_RELPATH = Path("docs/media/store-assets.json")

MEDIA_SUFFIXES = (
    ".png",
    ".jpg",
    ".jpeg",
    ".webp",
    ".svg",
    ".ico",
    ".icns",
    ".pdf",
    ".wav",
)

# Backticked token that looks like a repository path: contains a separator
# and a known suffix, or ends with a separator (a directory).
_PATH_TOKEN = re.compile(r"`([^`]+)`")


def _read_map() -> str:
    return (REPO_ROOT / MAP_RELPATH).read_text(encoding="utf-8")


def _map_rows() -> list[list[str]]:
    """The zone table of the map, as raw cells (backticks preserved).

    Only the table under the «Зоны репозитория» heading is parsed: another
    four-column table elsewhere in the file is prose, not a zone list.
    """
    lines = _read_map().splitlines()
    start = next(
        (
            index
            for index, line in enumerate(lines)
            if line.strip().startswith("## Зоны репозитория")
        ),
        None,
    )
    if start is None:
        return []
    rows: list[list[str]] = []
    for line in lines[start + 1 :]:
        stripped = line.strip()
        if stripped.startswith("## "):
            break  # the next section begins
        if not stripped.startswith("|"):
            continue
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if len(cells) < 4:
            continue
        if set(cells[0]) <= {"-", ":", " "}:
            continue  # the header separator
        rows.append(cells)
    return rows


def _exists(path: Path) -> bool:
    """Existence that accepts the glob patterns the map uses (`tauri/gen/**`)."""
    if any(char in path.name for char in "*?[") or "**" in str(path):
        return bool(glob.glob(str(path), recursive=True))
    return path.exists()


def _tracked_media_files() -> list[str]:
    result = subprocess.run(
        ["git", "-C", str(REPO_ROOT), "ls-files", "-z"],
        capture_output=True,
        text=True,
        check=True,
        shell=False,
    )
    return [
        name
        for name in result.stdout.split("\0")
        if name and name.lower().endswith(MEDIA_SUFFIXES)
    ]


def _zone_key(file_path: str) -> str:
    parts = file_path.split("/")
    if len(parts) <= 2:
        return file_path
    return "/".join(parts[:2])


# --- the map exists ----------------------------------------------------------


def _in_git(cell: str) -> bool:
    """Whether the row claims the zone is tracked.

    The cell may carry a note («да (только метаданные)»); a note must not
    silently lift the guard, so the test looks at the leading word.
    """
    return cell.strip().lower().startswith("да")


def test_the_media_map_exists() -> None:
    assert (REPO_ROOT / MAP_RELPATH).is_file(), (
        "docs/media/README.md is the map of the media zones (ADR-068); "
        "without it the zones are undocumented again"
    )


# --- every listed path exists ------------------------------------------------


@pytest.mark.parametrize("row", _map_rows(), ids=lambda row: row[0])
def test_a_zone_marked_in_git_exists(row: list[str]) -> None:
    zone, _, in_git, _ = row
    if not _in_git(in_git):
        return  # a CDN prefix or a staging directory may be absent locally

    tokens = _PATH_TOKEN.findall(zone) or [zone]
    for token in tokens:
        assert _exists(REPO_ROOT / token), (
            f"the map lists {token} but it is not in the repository"
        )


# --- every listed consumer exists -------------------------------------------


@pytest.mark.parametrize("row", _map_rows(), ids=lambda row: row[0])
def test_a_zone_in_git_has_a_consumer_that_exists(row: list[str]) -> None:
    zone, _, in_git, consumer = row
    if not _in_git(in_git):
        return  # an out-of-repo zone is consumed out of band

    tokens = _PATH_TOKEN.findall(consumer)
    paths = [
        token
        for token in tokens
        if "/" in token and not token.startswith(("/", "npm ", "python "))
    ]
    assert paths, (
        f"the map lists {zone} without a concrete consumer path: "
        "put the consuming file or directory in backticks"
    )
    for consumer_path in paths:
        assert _exists(REPO_ROOT / consumer_path), (
            f"the map says {zone} is consumed by {consumer_path}, "
            "but that path is not in the repository"
        )


# --- the map knows every media file -----------------------------------------


def _mapped_paths() -> list[str]:
    """Every backticked path of the zone table's first column."""
    mapped: list[str] = []
    for zone, _, _, _ in _map_rows():
        mapped.extend(_PATH_TOKEN.findall(zone))
    return mapped


def _covers(mapped: str, name: str) -> bool:
    """Whether a mapped path of the map covers a tracked file.

    Three shapes appear in the map: a directory (`images/`), a recursive glob
    (`tauri/gen/**`) and a name glob (`logo*.png`). A plain path matches
    itself exactly.
    """
    if mapped.endswith("**"):
        return name.startswith(mapped.rstrip("*"))
    if "*" in mapped:
        return name.startswith(mapped.split("*", 1)[0])
    if mapped.endswith("/"):
        return name.startswith(mapped)
    return name == mapped or name.startswith(f"{mapped}/")


def test_every_media_file_is_under_a_mapped_zone() -> None:
    # A substring match over the whole map would let a new directory hide
    # behind an already-mentioned prefix (origa_landing/public/badges/ under
    # origa_landing/public/images/). The check is per file: some mapped path
    # must cover it.
    mapped = _mapped_paths()
    orphans: dict[str, list[str]] = {}
    for name in _tracked_media_files():
        if not any(_covers(path, name) for path in mapped):
            orphans.setdefault(_zone_key(name), []).append(name)

    assert not orphans, (
        "the repository holds media the map does not cover: "
        f"{orphans}\nadd the zone to docs/media/README.md"
    )


# --- the registry covers every channel --------------------------------------


def test_the_registry_covers_every_published_channel() -> None:
    registry = assets_from_registry(read_registry(REPO_ROOT / REGISTRY_RELPATH))
    channels = {asset.channel for asset in registry}
    expected = {group.channel for group in GROUPS}

    assert expected <= channels, (
        f"the registry is missing the channels {sorted(expected - channels)}: "
        "publish them or drop the group"
    )
    unknown = channels - expected
    assert not unknown, (
        f"the registry holds channels no group publishes: {sorted(unknown)}"
    )


def test_the_registry_holds_no_legacy_path_that_still_exists() -> None:
    # legacyPath is a historical pointer into git history: after the cleanup
    # the file must be gone from the tree. A legacyPath that still exists
    # means the binary was published but not removed — the opposite of #797.
    registry = assets_from_registry(read_registry(REPO_ROOT / REGISTRY_RELPATH))
    still_here = [
        asset.legacyPath
        for asset in registry
        if asset.legacyPath and (REPO_ROOT / asset.legacyPath).exists()
    ]

    assert not still_here, (
        f"published assets still sit in the tree: {still_here}\nremove them "
        "(the CDN copy is the live one)"
    )
