"""Unit tests for the local CDN corpus resolver (``_cdn_corpus``).

Mirrors the Rust contract in ``origa/src/corpus.rs`` (the source of truth):
``$ORIGA_CDN_DIR`` wins, an empty value counts as unset, the default is
``~/origa-cdn``. Pure tests — no corpus, network, or repository state.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from _cdn_corpus import corpus_dir


def test_explicit_env_dir_wins_over_home(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("ORIGA_CDN_DIR", "/data/origa-cdn")
    monkeypatch.setenv("HOME", "/home/tester")
    assert corpus_dir() == Path("/data/origa-cdn")


def test_empty_env_dir_counts_as_unset(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("ORIGA_CDN_DIR", "")
    monkeypatch.setenv("HOME", "/home/tester")
    assert corpus_dir() == Path("/home/tester") / "origa-cdn"


def test_home_is_the_default_location(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("ORIGA_CDN_DIR", raising=False)
    monkeypatch.setenv("HOME", "/home/tester")
    assert corpus_dir() == Path("/home/tester") / "origa-cdn"


def test_missing_home_falls_back_to_user_home(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("ORIGA_CDN_DIR", raising=False)
    monkeypatch.delenv("HOME", raising=False)
    assert corpus_dir() == Path.home() / "origa-cdn"
