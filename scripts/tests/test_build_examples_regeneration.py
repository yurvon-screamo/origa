"""Acceptance coverage for the examples corpus regeneration (task #410).

The sibling ``test_build_examples.py`` pins the carry-over helpers in
isolation. This module drives the real CLI/pipeline against a tiny local
corpus fixture: stable ids, carried translations, range-based chunk names,
idempotence, and fail-loud on a damaged previous corpus. No prod state is
touched and no network is used — ``ensure_examples_file`` is fed a local gz.
"""

from __future__ import annotations

import gzip
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

import build_examples as gen

REPO = Path(__file__).resolve().parents[2]
SCRIPT = REPO / "scripts" / "build_examples.py"


def _make_work(work: Path, pairs: list[tuple[str, str, str]]) -> Path:
    """Write a local examples.utf.gz the CLI unpacks instead of downloading."""
    work.mkdir(parents=True, exist_ok=True)
    text = "".join(f"A: {ja}\t{en}\nB: {bline}\n" for ja, en, bline in pairs)
    with gzip.open(work / "examples.utf.gz", "wt", encoding="utf-8") as fh:
        fh.write(text)
    return work / "examples.utf"


def _make_cdn(cdn: Path, *, rows=None, refs=None) -> Path:
    """Build a minimal CDN root: word sets, furigana seed, old corpus."""
    sets = cdn / "well_known_set"
    sets.mkdir(parents=True, exist_ok=True)
    (sets / "n5.json").write_text(
        json.dumps({"level": "N5", "words": ["あさ"]}), encoding="utf-8"
    )
    dic = cdn / "dictionaries"
    dic.mkdir(parents=True, exist_ok=True)
    (dic / "JmdictFurigana.txt").write_text("朝|あさ\n", encoding="utf-8")

    out = cdn / "examples"
    out.mkdir(parents=True, exist_ok=True)
    if rows is not None:
        (out / "data").mkdir(parents=True, exist_ok=True)
        by_chunk: dict[int, list[dict]] = {}
        for row in rows:
            by_chunk.setdefault(row["i"] // gen.CHUNK_SIZE, []).append(row)
        for cn, part in by_chunk.items():
            (out / "data" / f"s{cn:04d}.json").write_text(
                json.dumps(sorted(part, key=lambda r: r["i"]), ensure_ascii=False),
                encoding="utf-8",
            )
        words = {}
        for sid, word in (refs or {}).items():
            words.setdefault(word, []).append([sid, 0, 2])
        (out / "index.json").write_text(
            json.dumps(
                {
                    "v": 1,
                    "h": "old",
                    "words": {w: {"refs": r} for w, r in words.items()},
                    "s": len(rows),
                },
                ensure_ascii=False,
            ),
            encoding="utf-8",
        )
    return out


PAIRS = [
    ("あさ、おはよう。", "good morning", "あさ"),
    ("あさ、また明日。", "see you tomorrow", "あさ"),
    # Tanaka numeric artifact in the ruby seed: the #929 filter must drop it.
    ("あさ、今日はいい天気。", "nice weather today", "あさ(#2028980)"),
]


def _snapshot(directory: Path) -> dict[str, bytes]:
    return {
        str(p.relative_to(directory)): p.read_bytes()
        for p in sorted(directory.rglob("*"))
        if p.is_file()
    }


def _run_pipeline(monkeypatch, tmp_path: Path, cdn: Path, work: Path) -> int:
    monkeypatch.setattr(gen, "CDN", cdn)
    monkeypatch.setattr(sys, "argv", ["build_examples.py", "--work", str(work)])
    return gen.main()


def test_main_regeneration_carries_ids_translations_and_ranges(monkeypatch, tmp_path):
    cdn = _make_cdn(
        tmp_path / "cdn",
        rows=[
            {
                "i": 1001,
                "x": "あさ、おはよう。",
                "en": "good morning",
                "f": [],
                "ru": "утро1",
                "vi": "sáng1",
                "ko": "아침1",
            },
            {
                "i": 22412,
                "x": "あさ、また明日。",
                "en": "see you tomorrow",
                "f": [],
                "ru": "утро2",
                "vi": "sáng2",
                "ko": "아침2",
            },
            {
                "i": 23000,
                "x": "filler sentence",
                "en": "",
                "f": [],
                "w": "あさ",
                "ru": "филлер",
            },
        ],
        refs={1001: "あさ", 22412: "あさ"},
    )
    work = tmp_path / "work"
    _make_work(work, PAIRS)

    assert _run_pipeline(monkeypatch, tmp_path, tmp_path / "cdn", work) == 0

    index = json.loads((cdn / "index.json").read_text(encoding="utf-8"))
    # hash covers words/s/count, not the h field itself
    body = dict(index, h="")
    expected = hashlib.sha256(
        json.dumps(
            body, ensure_ascii=False, sort_keys=True, separators=(",", ":")
        ).encode()
    ).hexdigest()
    assert index["h"] == expected
    assert {ref[0] for ref in index["words"]["あさ"]["refs"]} == {1001, 22412, 22413}

    # chunks are named by id range: carried ids sit in s0001/s0022, the new
    # id appended to s0022. Positional packing would put everything in s0000.
    assert sorted(p.name for p in (cdn / "data").glob("*.json")) == [
        "s0001.json",
        "s0022.json",
    ]
    chunk_1 = json.loads((cdn / "data" / "s0001.json").read_text(encoding="utf-8"))
    chunk_22 = json.loads((cdn / "data" / "s0022.json").read_text(encoding="utf-8"))
    assert [r["i"] for r in chunk_1] == [1001]
    assert [r["i"] for r in chunk_22] == [22412, 22413]
    assert {k: chunk_1[0][k] for k in ("ru", "vi", "ko")} == {
        "ru": "утро1",
        "vi": "sáng1",
        "ko": "아침1",
    }
    assert {k: chunk_22[0][k] for k in ("ru", "vi", "ko")} == {
        "ru": "утро2",
        "vi": "sáng2",
        "ko": "아침2",
    }
    # a genuinely new sentence ships without translations
    assert not {"ru", "vi", "ko"} & chunk_22[1].keys()
    # filler and Tanaka numeric artifacts never reach the corpus
    assert "filler sentence" not in (cdn / "data" / "s0022.json").read_text(
        encoding="utf-8"
    )
    assert not any("#2028980" in pair for r in chunk_22 for pair in r["f"])

    cache = json.loads((work / "examples_matches.json").read_text(encoding="utf-8"))
    assert {c["sid"] for c in cache["あさ"]} == {1001, 22412, 22413}


def test_main_regeneration_is_byte_idempotent(monkeypatch, tmp_path):
    cdn = _make_cdn(
        tmp_path / "cdn",
        rows=[{"i": 1001, "x": "あさ、おはよう。", "en": "e", "f": [], "ru": "р"}],
        refs={1001: "あさ"},
    )
    work = tmp_path / "work"
    _make_work(work, PAIRS)

    assert _run_pipeline(monkeypatch, tmp_path, tmp_path / "cdn", work) == 0
    first = _snapshot(cdn)
    assert _run_pipeline(monkeypatch, tmp_path, tmp_path / "cdn", work) == 0
    assert _snapshot(cdn) == first


def _run_cli(cdn: Path, work: Path) -> subprocess.CompletedProcess[str]:
    env = {**os.environ, "ORIGA_CDN_DIR": str(cdn)}
    return subprocess.run(
        [sys.executable, str(SCRIPT), "--work", str(work)],
        capture_output=True,
        text=True,
        env=env,
        timeout=120,
        check=False,
    )


def test_cli_fresh_machine_uses_compact_ids_and_warns(tmp_path):
    cdn = tmp_path / "cdn"
    _make_cdn(cdn)
    work = tmp_path / "work"
    _make_work(work, PAIRS)

    proc = _run_cli(cdn, work)

    assert proc.returncode == 0, proc.stderr
    assert "no previous corpus present" in proc.stderr
    assert "will NOT be carried" in proc.stderr
    index = json.loads((cdn / "examples" / "index.json").read_text(encoding="utf-8"))
    assert {r[0] for r in index["words"]["あさ"]["refs"]} == {0, 1, 2}
    assert (cdn / "examples" / "data" / "s0000.json").exists()


@pytest.mark.parametrize(
    ("broken", "needle"),
    [
        ("corrupt", "present but unreadable"),
        ("half", "no matching chunk rows were read"),
    ],
)
def test_cli_damaged_previous_corpus_fails_loud_without_wiping(
    tmp_path, broken, needle
):
    out = _make_cdn(
        tmp_path / "cdn",
        rows=[{"i": 1001, "x": "あさ、おはよう。", "en": "e", "f": [], "ru": "р"}],
        refs={1001: "あさ"},
    )
    original_index = (out / "index.json").read_bytes()
    original_chunk = (out / "data" / "s0001.json").read_bytes()
    if broken == "corrupt":
        (out / "index.json").write_text("{ broken", encoding="utf-8")
    else:
        for p in (out / "data").glob("*.json"):
            p.unlink()
    work = tmp_path / "work"
    _make_work(work, PAIRS)

    proc = _run_cli(out.parent, work)

    assert proc.returncode == 1
    assert needle in proc.stderr
    assert "fix or remove" in proc.stderr or "re-materialize" in proc.stderr
    # a failed regeneration must not destroy the only copy of the old corpus
    if broken == "corrupt":
        assert (out / "index.json").read_bytes() == b"{ broken"
        assert (out / "data" / "s0001.json").read_bytes() == original_chunk
    else:
        assert (out / "index.json").read_bytes() == original_index
