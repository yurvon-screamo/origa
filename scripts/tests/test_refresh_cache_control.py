"""Unit tests for ``refresh_cache_control.py`` decision logic.

Only the pure helpers are exercised — S3 transport (list/HEAD/copy-object)
is an operator-run side effect and is not mocked here.
"""

from __future__ import annotations

import pytest

from refresh_cache_control import (
    filter_safe_keys,
    is_safe_key,
    needs_update,
    normalize_cache_control,
)


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
    assert normalize_cache_control("Public, Max-Age=300") == "public,max-age=300"
    assert normalize_cache_control(None) == ""
    assert normalize_cache_control("") == ""


@pytest.mark.parametrize(
    "key",
    [
        "manifest.json",
        "grammar/grammar.json",
        "dictionaries/char_def.bin",
        "kanji_animations/anim_0001.svg",
        "phrases/audio/phrase_0001.mp3",
        "dictionary/chunk_11.json",
    ],
)
def test_safe_key_accepted(key: str):
    assert is_safe_key(key)


@pytest.mark.parametrize(
    "key",
    [
        # Shell metacharacters that would break out of `pwsh -Command`.
        "evil;rm",
        'evil"quote',
        "evil`backtick",
        "evil|pipe",
        "evil$var",
        "evil&amp",
        "evil space",
        "",
    ],
)
def test_unsafe_key_rejected(key: str):
    assert not is_safe_key(key)


def test_filter_safe_keys_partitions():
    keys = [
        "dictionary/chunk_01.json",
        "evil;rm",
        "phrases/audio/x.mp3",
        'bad"quote',
    ]
    safe, unsafe = filter_safe_keys(keys)
    assert safe == ["dictionary/chunk_01.json", "phrases/audio/x.mp3"]
    assert unsafe == ["evil;rm", 'bad"quote']
