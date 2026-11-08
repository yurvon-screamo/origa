"""Guards for the TrailBase pin in the CI e2e jobs (``.github/workflows/ci.yml``).

The e2e jobs pin a direct TrailBase release download (URL + cache key) instead
of using the installer, and the version appears in exactly two places that must
stay in sync: a stale cache key would silently serve an old binary after the
URL moves (cache hit skips the install step), and a stale URL would break the
install outright. The comment in the workflow ("Keep the version in sync with
the cache key") keeps a human honest; these tests make the invariant machine
checked (task 20261108160559-p2imxh7, the v0.28.3 → v0.35.10 bump).

The tests are pure: they read the workflow file (and synthetic strings for the
parser units) and never hit the network. They deliberately do NOT compare the
CI pin with ``TRAILBASE_VERSION`` in ``origa_ui/Dockerfile`` — that is an
independent pin by design, and the two legitimately diverge while a prod
bump PR is still open.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
CI_RELPATH = Path(".github/workflows/ci.yml")

CACHE_KEY_RE = re.compile(r"trailbase-v(\d+)\.(\d+)\.(\d+)-verified")
DOWNLOAD_URL_RE = re.compile(
    r"releases/download/v(\d+)\.(\d+)\.(\d+)/trailbase_v(\d+)\.(\d+)\.(\d+)_x86_64_linux\.zip"
)


def _tuple(match: re.Match[str], group: int = 1) -> tuple[int, int, int]:
    return tuple(int(match.group(group + i)) for i in range(3))  # type: ignore[return-value]


def _extract_pin(text: str) -> tuple[tuple[int, int, int], tuple[int, int, int]]:
    """Extract (cache key version, download URL version) from workflow text.

    Raises AssertionError with a targeted message when either marker is
    missing or duplicated — both mean the workflow structure rotted.
    """
    key_matches = list(CACHE_KEY_RE.finditer(text))
    assert len(key_matches) == 1, (
        f"expected exactly one trailbase cache key in ci.yml, "
        f"found {len(key_matches)}: {[m.group(0) for m in key_matches]}"
    )
    url_matches = list(DOWNLOAD_URL_RE.finditer(text))
    assert len(url_matches) == 1, (
        f"expected exactly one trailbase x86_64 linux download URL in ci.yml, "
        f"found {len(url_matches)}"
    )
    assert _tuple(url_matches[0]) == _tuple(url_matches[0], 4), (
        "trailbase download URL embeds two different versions "
        f"({_tuple(url_matches[0])} in the path vs {_tuple(url_matches[0], 4)} "
        "in the filename)"
    )
    return _tuple(key_matches[0]), _tuple(url_matches[0])


def test_cache_key_and_download_url_stay_in_sync():
    """Cache key version == download URL version in the real ci.yml.

    A partial bump (one line without the other) is the silent failure mode:
    a stale key serves an old binary from cache while the URL points
    elsewhere, and neither the install nor the e2e run notices.
    """
    text = (REPO_ROOT / CI_RELPATH).read_text(encoding="utf-8")
    key_version, url_version = _extract_pin(text)
    assert key_version == url_version, (
        f"ci.yml trailbase cache key is v{'.'.join(map(str, key_version))} "
        f"but the download URL is v{'.'.join(map(str, url_version))} — bump "
        "both together or the cache will serve a binary the URL no longer "
        "describes (see 'Keep the version in sync with the cache key')"
    )


@pytest.mark.parametrize(
    "text",
    [
        # URL bumped, cache key left behind — the silent stale-binary case.
        (
            "key: trailbase-v0.35.10-verified\n"
            "https://github.com/trailbaseio/trailbase/releases/download/"
            "v0.36.0/trailbase_v0.36.0_x86_64_linux.zip"
        ),
        # Cache key bumped, URL left behind — needless permanent cache miss.
        (
            "key: trailbase-v0.36.0-verified\n"
            "https://github.com/trailbaseio/trailbase/releases/download/"
            "v0.35.10/trailbase_v0.35.10_x86_64_linux.zip"
        ),
    ],
)
def test_desync_between_key_and_url_fails(text: str):
    """Each half-bump direction trips the sync invariant."""
    key_version, url_version = _extract_pin(text)
    assert key_version != url_version


def test_missing_marker_fails():
    """Removing either marker entirely fails the parser, not silently passes."""
    with pytest.raises(AssertionError, match="cache key"):
        _extract_pin(
            "https://github.com/trailbaseio/trailbase/releases/download/"
            "v0.35.10/trailbase_v0.35.10_x86_64_linux.zip"
        )
    with pytest.raises(AssertionError, match="download URL"):
        _extract_pin("key: trailbase-v0.35.10-verified")


def test_duplicated_markers_fail():
    """Duplicated download URLs (e.g. a copy-pasted second job) fail loudly."""
    url = (
        "https://github.com/trailbaseio/trailbase/releases/download/"
        "v0.35.10/trailbase_v0.35.10_x86_64_linux.zip"
    )
    with pytest.raises(AssertionError, match="download URL"):
        _extract_pin(f"key: trailbase-v0.35.10-verified\n{url}\n{url}")


def test_url_with_mismatched_filename_version_fails():
    """A URL whose path and filename disagree fails the parser."""
    with pytest.raises(AssertionError, match="two different versions"):
        _extract_pin(
            "key: trailbase-v0.35.10-verified\n"
            "https://github.com/trailbaseio/trailbase/releases/download/"
            "v0.35.10/trailbase_v0.36.0_x86_64_linux.zip"
        )


def test_legacy_v0_28_3_pin_is_gone():
    """Regression pin for task 20261108160559-p2imxh7: the pre-bump v0.28.3
    pin (which drifted from prod) must not come back via a full revert of a
    future change — a revert that restores it fails here instead of silently
    reopening the e2e-vs-prod version gap."""
    text = (REPO_ROOT / CI_RELPATH).read_text(encoding="utf-8")
    assert "0.28.3" not in text, (
        "ci.yml is pinned back to trailbase v0.28.3 — the version the e2e "
        "jobs outgrew (task 20261108160559-p2imxh7); bumping back needs a "
        "conscious decision, not a revert side effect"
    )
