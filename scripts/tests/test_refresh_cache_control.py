"""Unit tests for ``refresh_cache_control.py`` decision logic.

Only the pure comparison helpers are exercised — S3 transport (list/HEAD/
copy-object) is an operator-run side effect and is not mocked here.
"""

from __future__ import annotations

import pytest

from refresh_cache_control import needs_update, normalize_cc


@pytest.mark.parametrize(
    "current,target",
    [
        (
            "public, max-age=31536000, immutable",
            "public, max-age=300, must-revalidate",
        ),
        (None, "public, max-age=300, must-revalidate"),
        ("", "no-cache"),
    ],
)
def test_needs_update_when_different(current: str | None, target: str):
    assert needs_update(current, target)


@pytest.mark.parametrize(
    "current,target",
    [
        (
            "public, max-age=300, must-revalidate",
            "public, max-age=300, must-revalidate",
        ),
        # Spacing/case variants must count as equal so we don't churn metadata.
        (
            "public,max-age=300,must-revalidate",
            "public, max-age=300, must-revalidate",
        ),
        (
            "PUBLIC, MAX-AGE=300, MUST-REVALIDATE",
            "public, max-age=300, must-revalidate",
        ),
        (None, ""),
    ],
)
def test_no_update_when_equivalent(current: str | None, target: str):
    assert not needs_update(current, target)


def test_normalize_strips_spaces_and_lowercases():
    assert normalize_cc("Public, Max-Age=300") == "public,max-age=300"
    assert normalize_cc(None) == ""
    assert normalize_cc("") == ""
