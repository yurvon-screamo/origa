"""Unit tests for ``refresh_cache_control.py``.

Pure decision helpers are exercised directly. Scope dispatch
(``collect_targets`` / ``load_manifest_files`` / ``_confirm_expensive_walk``)
and the walk aggregation (``_run_walk`` / ``_print_summary``) are tested with
the S3 transport monkeypatched, since the live walk is an operator-run side
effect.
"""

from __future__ import annotations

import argparse
from collections import Counter

import pytest

import refresh_cache_control as rcc
from _cdn_s3 import filter_safe_keys, is_safe_key
from refresh_cache_control import (
    WalkOutcome,
    WalkSummary,
    is_heavy_prefix,
    needs_update,
    normalize_cache_control,
)


def _args(**overrides: object) -> argparse.Namespace:
    """Build a Namespace matching the argparse contract of ``main``.

    Mirrors the dest names collect_targets reads (``all``, ``prefix``,
    ``dry_run``) so dispatch tests stay independent of argparse wiring. The
    argparse↔dispatch contract is asserted separately via ``_build_parser``.
    """
    base: dict[str, object] = {"dry_run": False, "all": False, "prefix": None}
    base.update(overrides)
    return argparse.Namespace(**base)


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
        # Real kanji filenames use the kanji themselves (CJK) — must be
        # accepted, not rejected as "non-ASCII". This is the regression that
        # an ASCII allowlist introduced in cycle 2.
        "kanji_animations/一.svg",
        "kanji_frames/丁.json",
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
        "evil\ttab",
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
        "kanji_animations/一.svg",
        'bad"quote',
    ]
    safe, unsafe = filter_safe_keys(keys)
    assert safe == [
        "dictionary/chunk_01.json",
        "phrases/audio/x.mp3",
        "kanji_animations/一.svg",
    ]
    assert unsafe == ["evil;rm", 'bad"quote']


@pytest.mark.parametrize(
    "prefix",
    [
        "kanji_animations/",
        "kanji_animations",  # trailing-slash tolerant
        "kanji_animations/sub/",
        "phrases/audio/",
        "phrases/audio",
        "ndlocr/",
        "whisper/",
        "kanji_frames/",
    ],
)
def test_heavy_prefix_detected(prefix: str):
    assert is_heavy_prefix(prefix)


@pytest.mark.parametrize(
    "prefix",
    [
        "grammar/",
        "dictionary/",
        "phrases/data/",
        "pitch/",
        "well_known_set/",
        "manifest.json",
    ],
)
def test_content_prefix_not_heavy(prefix: str):
    assert not is_heavy_prefix(prefix)


# ---------------------------------------------------------------------------
# Scope dispatch (collect_targets)
# ---------------------------------------------------------------------------


def test_collect_targets_default_uses_manifest_files(monkeypatch):
    """Default scope must NOT trigger a full bucket walk — the original bug."""
    called = {"list_keys": False, "manifest": False}

    def fake_manifest() -> tuple[list[str], int]:
        called["manifest"] = True
        return ["manifest.json", "grammar/grammar.json"], 0

    def fake_list(prefix: str | None = None) -> tuple[list[str], int]:
        called["list_keys"] = True
        return ["should-not-happen"], 0

    monkeypatch.setattr(rcc, "load_manifest_files", fake_manifest)
    monkeypatch.setattr(rcc, "list_keys", fake_list)

    keys, dropped = rcc.collect_targets(_args())

    assert called["manifest"] is True
    assert called["list_keys"] is False
    assert keys == ["manifest.json", "grammar/grammar.json"]
    assert dropped == 0


def test_collect_targets_light_prefix_skips_confirmation(monkeypatch):
    captured: dict[str, str | None] = {}

    def fake_list(prefix: str | None = None) -> tuple[list[str], int]:
        captured["prefix"] = prefix
        return ["grammar/grammar.json"], 0

    monkeypatch.setattr(rcc, "list_keys", fake_list)
    monkeypatch.setattr(
        rcc, "_confirm_expensive_walk", lambda desc: pytest.fail("should not prompt")
    )

    keys, _ = rcc.collect_targets(_args(prefix="grammar/"))

    assert captured["prefix"] == "grammar/"
    assert keys == ["grammar/grammar.json"]


def test_collect_targets_heavy_prefix_prompts(monkeypatch):
    captured: dict[str, str | None] = {}

    def fake_list(prefix: str | None = None) -> tuple[list[str], int]:
        captured["prefix"] = prefix
        return ["kanji_animations/一.svg"], 0

    confirmed = {"value": False}

    def fake_confirm(desc: str) -> bool:
        confirmed["value"] = True
        assert "kanji_animations" in desc
        return True

    monkeypatch.setattr(rcc, "list_keys", fake_list)
    monkeypatch.setattr(rcc, "_confirm_expensive_walk", fake_confirm)

    keys, _ = rcc.collect_targets(_args(prefix="kanji_animations/"))

    assert confirmed["value"] is True
    assert captured["prefix"] == "kanji_animations/"
    assert keys == ["kanji_animations/一.svg"]


def test_collect_targets_heavy_prefix_aborts_when_not_confirmed(monkeypatch):
    monkeypatch.setattr(rcc, "list_keys", lambda prefix=None: ["should-not-run"])
    monkeypatch.setattr(rcc, "_confirm_expensive_walk", lambda desc: False)

    with pytest.raises(SystemExit) as exc:
        rcc.collect_targets(_args(prefix="phrases/audio/"))

    assert exc.value.code == 0


def test_collect_targets_all_calls_list_keys_without_prefix(monkeypatch):
    captured: dict[str, str | None] = {}

    def fake_list(prefix: str | None = None) -> tuple[list[str], int]:
        captured["prefix"] = prefix
        return ["a", "b"], 1

    monkeypatch.setattr(rcc, "list_keys", fake_list)
    monkeypatch.setattr(rcc, "_confirm_expensive_walk", lambda desc: True)

    keys, dropped = rcc.collect_targets(_args(all=True))

    assert captured["prefix"] is None
    assert keys == ["a", "b"]
    assert dropped == 1


def test_collect_targets_all_aborts_when_not_confirmed(monkeypatch):
    """Refusing the --all prompt exits cleanly without touching S3."""
    monkeypatch.setattr(rcc, "list_keys", lambda prefix=None: ["should-not-run"])
    monkeypatch.setattr(rcc, "_confirm_expensive_walk", lambda desc: False)

    with pytest.raises(SystemExit) as exc:
        rcc.collect_targets(_args(all=True))

    assert exc.value.code == 0


# ---------------------------------------------------------------------------
# argparse ↔ collect_targets contract
# ---------------------------------------------------------------------------


def test_build_parser_assigns_dests_collect_targets_reads():
    """Guards against dest-name drift between the parser and dispatch."""
    all_args = rcc._build_parser().parse_args(["--all"])
    assert all_args.all is True
    assert all_args.prefix is None
    assert all_args.dry_run is False

    prefix_args = rcc._build_parser().parse_args(["--prefix", "grammar/"])
    assert prefix_args.all is False
    assert prefix_args.prefix == "grammar/"

    dry_args = rcc._build_parser().parse_args(["--dry-run", "--all"])
    assert dry_args.all is True
    assert dry_args.dry_run is True


def test_build_parser_rejects_all_and_prefix_together(capsys):
    with pytest.raises(SystemExit):
        rcc._build_parser().parse_args(["--all", "--prefix", "grammar/"])


def test_validate_prefix_rejects_unsafe_chars(capsys):
    with pytest.raises(SystemExit) as exc:
        rcc._validate_prefix("evil;rm")
    assert exc.value.code == 2
    assert "unsafe" in capsys.readouterr().err


# ---------------------------------------------------------------------------
# --all confirmation (_confirm_expensive_walk)
# ---------------------------------------------------------------------------


def test_confirm_prompts_and_aborts_on_no(capsys, monkeypatch):
    captured: dict[str, str] = {}

    def fake_input(prompt: str = "") -> str:
        captured["prompt"] = prompt
        return "n"

    monkeypatch.setattr("builtins.input", fake_input)

    assert rcc._confirm_expensive_walk("--all") is False

    err = capsys.readouterr().err
    assert "WARNING" in err
    assert "Aborted" in err
    assert "Continue? [y/N]" in captured["prompt"]


def test_confirm_accepts_explicit_yes(capsys, monkeypatch):
    monkeypatch.setattr("builtins.input", lambda *a: "y")
    assert rcc._confirm_expensive_walk("--all") is True
    assert "WARNING" in capsys.readouterr().err


def test_confirm_prompts_even_in_dry_run(monkeypatch):
    """The cost is LIST+HEAD, which --dry-run does NOT avoid (regression guard)."""
    prompted = {"value": False}

    def fake_input(prompt: str = "") -> str:
        prompted["value"] = True
        return "n"

    monkeypatch.setattr("builtins.input", fake_input)
    rcc._confirm_expensive_walk("--all")
    assert prompted["value"] is True


def test_confirm_fails_closed_on_eof(capsys, monkeypatch):
    """Non-interactive stdin (CI, pipe) must abort, not crash with a traceback."""

    def raise_eof(_prompt: str = "") -> str:
        raise EOFError

    monkeypatch.setattr("builtins.input", raise_eof)

    assert rcc._confirm_expensive_walk("--all") is False
    assert "no interactive stdin" in capsys.readouterr().err


# ---------------------------------------------------------------------------
# Default scope source (load_manifest_files)
# ---------------------------------------------------------------------------


def test_load_manifest_files_returns_manifest_keys_plus_manifest_itself(monkeypatch):
    manifest = {
        "version": 1,
        "files": {
            "grammar/grammar.json": "h1",
            "dictionary/chunk_01.json": "h2",
        },
    }
    monkeypatch.setattr(rcc, "download_remote_manifest", lambda dry_run=False: manifest)

    keys, dropped = rcc.load_manifest_files()

    # manifest.json is tracked nowhere in its own files dict but still needs
    # its no-cache header refreshed.
    assert "manifest.json" in keys
    assert "grammar/grammar.json" in keys
    assert "dictionary/chunk_01.json" in keys
    assert dropped == 0


def test_load_manifest_files_exits_when_manifest_missing(monkeypatch, capsys):
    monkeypatch.setattr(rcc, "download_remote_manifest", lambda dry_run=False: None)

    with pytest.raises(SystemExit):
        rcc.load_manifest_files()

    assert "not found" in capsys.readouterr().err


def test_load_manifest_files_exits_when_files_missing(monkeypatch):
    monkeypatch.setattr(
        rcc, "download_remote_manifest", lambda dry_run=False: {"version": 1}
    )

    with pytest.raises(SystemExit):
        rcc.load_manifest_files()


def test_load_manifest_files_drops_unsafe_keys(monkeypatch, capsys):
    manifest = {"version": 1, "files": {"evil;rm": "h1", "ok.json": "h2"}}
    monkeypatch.setattr(rcc, "download_remote_manifest", lambda dry_run=False: manifest)

    keys, dropped = rcc.load_manifest_files()

    assert keys == ["ok.json", "manifest.json"]
    assert dropped == 1
    assert "evil;rm" in capsys.readouterr().err


# ---------------------------------------------------------------------------
# Walk aggregation (_run_walk / _print_summary)
# ---------------------------------------------------------------------------


def test_run_walk_classifies_outcomes(monkeypatch):
    sequence = iter(
        [
            WalkOutcome("updated", failed=False),
            WalkOutcome("already_ok", failed=False),
            WalkOutcome("head_failed", failed=True),  # retriable
            WalkOutcome("copy_failed", failed=True),  # retriable
            WalkOutcome("oversize", failed=False),  # not retriable, separate bucket
        ]
    )
    monkeypatch.setattr(rcc, "_walk_one", lambda key, dry_run: next(sequence))

    summary = rcc._run_walk(["a", "b", "c", "d", "e"], dry_run=False)

    assert isinstance(summary, WalkSummary)
    assert summary.scanned == 5
    assert summary.counts["updated"] == 1
    assert summary.counts["already_ok"] == 1
    assert summary.retriable == ["c", "d"]
    assert summary.oversize == ["e"]


def _summary(**counts: int) -> WalkSummary:
    """Build a WalkSummary pre-populated with the given category counts."""
    return WalkSummary(
        scanned=sum(counts.values()),
        counts=Counter(counts),
        retriable=[],
        oversize=[],
    )


def test_print_summary_reports_retriable_and_oversize(capsys):
    summary = _summary(updated=1, already_ok=0)
    summary = summary._replace(retriable=["bad.json"], oversize=["huge.bin"])

    rcc._print_summary(summary, dropped=2, dry_run=False)

    out = capsys.readouterr().out
    assert "scanned:           1" in out
    assert "updated:           1" in out
    assert "dropped (unsafe):  2" in out
    assert "failed (retriable): 1" in out
    assert "- bad.json" in out
    assert "oversize (> 5 GiB, manual): 1" in out
    assert "- huge.bin" in out
    assert "Re-run to retry" in out


def test_print_summary_dry_run_message(capsys):
    summary = _summary(already_ok=1)

    rcc._print_summary(summary, dropped=0, dry_run=True)

    out = capsys.readouterr().out
    assert "already OK:        1" in out
    assert "(dry-run: no metadata rewritten)" in out
