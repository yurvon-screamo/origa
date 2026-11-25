"""Unit tests for ``build_examples.py`` example-selection fixes.

Covers the two content-side corrections of the wrong-furigana spike
(20261109082311-9i1um6f): junk Tanaka id readings never become ruby pairs,
and kanji words are only backed by sentences where the word actually
surfaces (an offset-less kana-variant match — the auxiliary てしまう usage
of 仕舞う — does not demonstrate the card word).
"""

from __future__ import annotations

import json

import pytest

import build_examples as gen


def test_bline_ruby_drops_numeric_id_readings():
    line = "彼(かれ) で(2028980) 法(ほう) て(2654270)"
    assert gen.bline_ruby(line) == [["彼", "かれ"], ["法", "ほう"]]


def test_bline_ruby_keeps_all_kana_readings():
    line = "頭(あたま) を(を) 冷やせ(ひやせ)"
    assert gen.bline_ruby(line) == [
        ["頭", "あたま"],
        ["を", "を"],
        ["冷やせ", "ひやせ"],
    ]


def _sentences(n):
    return [{"id": i, "x": f"sentence {i}", "en": "", "f": []} for i in range(n)]


def test_select_for_words_kanji_word_needs_surfaced_offset():
    # Arrange: both matches are offset-less (kana-variant auxiliary usage).
    sentences = _sentences(2)
    matches = {"仕舞う": {0: [-1, -1], 1: [-1, -1]}}

    # Act
    sel = gen.select_for_words(sentences, matches, {"仕舞う": "N4"})

    # Assert: no example is better than an example without the word.
    assert sel == {}


def test_select_for_words_kanji_word_drops_only_offsetless_refs():
    # Arrange: one surfacing match, one kana-variant match.
    sentences = _sentences(2)
    matches = {"頭": {0: [-1, -1], 1: [0, 1]}}

    # Act
    sel = gen.select_for_words(sentences, matches, {"頭": "N4"})

    # Assert
    assert sel == {"頭": [{"sid": 1, "start": 0, "end": 1}]}


def test_select_for_words_kana_word_keeps_offsetless_matches():
    # A kana word matched through its kanji spelling (朝 for あさ) is a
    # legitimate example: offsets stay [-1,-1] by design there.
    sentences = _sentences(1)
    matches = {"あさ": {0: [-1, -1]}}

    sel = gen.select_for_words(sentences, matches, {"あさ": "N5"})

    assert sel == {"あさ": [{"sid": 0, "start": -1, "end": -1}]}


# ---------------------------------------------------------------------------
# Carry-over of stable sentence ids (post-#929 regeneration)
# ---------------------------------------------------------------------------


def _make_old_corpus(tmp_path, rows, refs):
    """Build a minimal previous corpus in tmp_path/cdn-style layout."""
    data_dir = tmp_path / "data"
    data_dir.mkdir(parents=True, exist_ok=True)
    # chunk by id range (1000 per chunk) like the real writer
    chunks: dict[int, list[dict]] = {}
    for r in rows:
        chunks.setdefault(r["i"] // 1000, []).append(r)
    for cn, part in chunks.items():
        part.sort(key=lambda r: r["i"])
        (data_dir / f"s{cn:04d}.json").write_text(
            json.dumps(part, ensure_ascii=False, separators=(",", ":")),
            encoding="utf-8",
        )
    index = {"v": 1, "h": "h", "words": {}, "s": len(rows)}
    # build words -> refs
    words_map: dict[str, list] = {}
    for sid, word in refs.items():
        words_map.setdefault(word, []).append([sid, 0, 1])
    for w, rs in words_map.items():
        index["words"][w] = {"refs": rs}
    (tmp_path / "index.json").write_text(
        json.dumps(index, ensure_ascii=False, separators=(",", ":")),
        encoding="utf-8",
    )
    return tmp_path


def test_read_old_corpus_empty_when_no_previous(tmp_path):
    out_dir = tmp_path / "examples"
    out_dir.mkdir()
    sid_to_x, x_to_sid, sid_to_tr, max_sid, present = gen.read_old_corpus(out_dir)
    assert sid_to_x == {} and x_to_sid == {} and sid_to_tr == {} and max_sid == 0


def test_read_old_corpus_drops_filler_rows(tmp_path):
    out_dir = tmp_path / "examples"
    out_dir.mkdir()
    rows = [
        {"i": 0, "x": "alpha", "en": "a", "f": [], "ru": "ра", "vi": "va", "ko": "ka"},
        {
            "i": 23000,
            "x": "filler text",
            "en": "",
            "f": [],
            "w": "word",
            "ru": "рф",
            "vi": "вф",
            "ko": "кф",
        },
    ]
    # only sid 0 is referenced
    _make_old_corpus(out_dir, rows, {0: "w"})
    sid_to_x, x_to_sid, sid_to_tr, max_sid, present = gen.read_old_corpus(out_dir)
    assert 0 in sid_to_x
    assert 23000 not in sid_to_x
    assert "filler text" not in x_to_sid
    assert max_sid == 0


def test_read_old_corpus_carries_translations(tmp_path):
    out_dir = tmp_path / "examples"
    out_dir.mkdir()
    rows = [
        {
            "i": 5,
            "x": "hello",
            "en": "hi",
            "f": [],
            "ru": "привет",
            "vi": "xin",
            "ko": "안",
        },
    ]
    _make_old_corpus(out_dir, rows, {5: "w"})
    sid_to_x, x_to_sid, sid_to_tr, max_sid, present = gen.read_old_corpus(out_dir)
    assert sid_to_x[5] == "hello"
    assert x_to_sid["hello"] == 5
    assert sid_to_tr[5] == {"ru": "привет", "vi": "xin", "ko": "안"}
    assert max_sid == 5


def test_read_old_corpus_collision_min_sid_wins(tmp_path):
    out_dir = tmp_path / "examples"
    out_dir.mkdir()
    rows = [
        {"i": 2, "x": "dup", "en": "", "f": [], "ru": "r2"},
        {"i": 7, "x": "dup", "en": "", "f": [], "ru": "r7"},
    ]
    _make_old_corpus(out_dir, rows, {2: "w", 7: "w2"})
    sid_to_x, x_to_sid, sid_to_tr, max_sid, present = gen.read_old_corpus(out_dir)
    assert x_to_sid["dup"] == 2  # min sid wins
    assert sid_to_x[2] == "dup" and sid_to_x[7] == "dup"
    assert max_sid == 7


def test_build_sid_map_carries_matching_and_new_above_max():
    sentences = [
        {"id": 0, "x": "old text", "en": "", "f": []},
        {"id": 1, "x": "brand new", "en": "", "f": []},
    ]
    used_sids = [0, 1]
    old_x_to_sid = {"old text": 42}
    sid_map, stats = gen.build_sid_map(sentences, used_sids, old_x_to_sid, 42)
    assert sid_map[0] == 42  # carried
    assert sid_map[1] > 42  # new above max
    assert stats == {"carried": 1, "new": 1}


def test_build_sid_map_duplicate_x_loser_gets_new_id():
    sentences = [
        {"id": 0, "x": "dup", "en": "", "f": []},
        {"id": 1, "x": "dup", "en": "", "f": []},
    ]
    used_sids = [0, 1]
    # only one old sid for "dup" -> second becomes new
    old_x_to_sid = {"dup": 10}
    sid_map, stats = gen.build_sid_map(sentences, used_sids, old_x_to_sid, 10)
    assert sid_map[0] == 10  # carried (first)
    assert sid_map[1] > 10  # loser -> new id
    assert stats["carried"] == 1 and stats["new"] == 1


def test_build_sid_map_no_old_corpus_uses_compact_ids():
    sentences = [
        {"id": 0, "x": "a", "en": "", "f": []},
        {"id": 1, "x": "b", "en": "", "f": []},
    ]
    used_sids = [0, 1]
    sid_map, stats = gen.build_sid_map(sentences, used_sids, {}, 0)
    assert sid_map == {0: 0, 1: 1}
    assert stats == {"carried": 0, "new": 2}


def test_read_old_corpus_empty_when_dir_absent(tmp_path):
    # fresh machine: no previous corpus at all -> legit compact fallback
    sid_to_x, x_to_sid, sid_to_tr, max_sid, present = gen.read_old_corpus(tmp_path)
    assert sid_to_x == {} and x_to_sid == {} and sid_to_tr == {} and max_sid == 0


def test_read_old_corpus_raises_on_broken_index(tmp_path):
    out_dir = _make_old_corpus(
        tmp_path / "examples",
        [{"i": 5, "x": "hello", "en": "hi", "f": []}],
        {5: "w"},
    )
    (out_dir / "index.json").write_text("{ not json", encoding="utf-8")
    with pytest.raises(gen.OldCorpusError):
        gen.read_old_corpus(out_dir)


def test_read_old_corpus_raises_on_broken_chunk(tmp_path):
    out_dir = _make_old_corpus(
        tmp_path / "examples",
        [{"i": 5, "x": "hello", "en": "hi", "f": []}],
        {5: "w"},
    )
    (out_dir / "data" / "s0001.json").write_text("]{broken", encoding="utf-8")
    with pytest.raises(gen.OldCorpusError):
        gen.read_old_corpus(out_dir)


def test_read_old_corpus_raises_on_incomplete_corpus(tmp_path):
    # index references sids that no chunk row provides: unusable for carry
    out_dir = _make_old_corpus(
        tmp_path / "examples",
        [{"i": 5, "x": "hello", "en": "hi", "f": []}],
        {5: "w"},
    )
    (out_dir / "index.json").write_text(
        json.dumps({"v": 1, "h": "", "words": {"w": {"refs": [[77, 0, 1]]}}, "s": 1}),
        encoding="utf-8",
    )
    with pytest.raises(gen.OldCorpusError):
        gen.read_old_corpus(out_dir)


def test_parse_old_chunk_row_drops_filler_and_malformed():
    referenced = {5}
    assert gen.parse_old_chunk_row({"i": 5, "x": "ok"}, referenced) == (5, "ok", {})
    assert (
        gen.parse_old_chunk_row({"i": 23000, "x": "filler", "w": "w"}, referenced)
        is None
    )
    assert gen.parse_old_chunk_row({"i": "NaN", "x": "bad"}, referenced) is None
    assert gen.parse_old_chunk_row({"x": "no id"}, referenced) is None
    assert gen.parse_old_chunk_row("not a dict", referenced) is None
    assert gen.parse_old_chunk_row({"i": 6, "x": ""}, referenced) is None


def test_build_out_rows_carries_translations():
    sentences = [{"id": 0, "x": "old text", "en": "e0", "f": [["旧", "きゅう"]]}]
    sid_map = {0: 42}
    old_x_to_sid = {"old text": 42}
    old_sid_to_tr = {42: {"ru": "ра", "vi": "va", "ko": "ko"}}
    rows = gen.build_out_rows(sentences, [0], sid_map, old_x_to_sid, old_sid_to_tr)
    assert rows == [
        {
            "i": 42,
            "x": "old text",
            "en": "e0",
            "f": [["旧", "きゅう"]],
            "ru": "ра",
            "vi": "va",
            "ko": "ko",
        }
    ]


def test_build_out_rows_new_row_ships_without_translations():
    sentences = [{"id": 0, "x": "brand new", "en": "e0", "f": []}]
    sid_map = {0: 22413}
    rows = gen.build_out_rows(sentences, [0], sid_map, {}, {})
    assert rows == [{"i": 22413, "x": "brand new", "en": "e0", "f": []}]


def test_read_old_corpus_raises_when_index_present_but_chunks_missing(tmp_path):
    # half-corpus: index references sentences, no chunk files at all
    out_dir = tmp_path / "examples"
    out_dir.mkdir()
    (out_dir / "index.json").write_text(
        json.dumps({"v": 1, "h": "", "words": {"w": {"refs": [[5, 0, 1]]}}, "s": 1}),
        encoding="utf-8",
    )
    with pytest.raises(gen.OldCorpusError):
        gen.read_old_corpus(out_dir)


def test_read_old_corpus_raises_when_chunks_present_but_index_missing(tmp_path):
    # half-corpus: chunk files exist, index.json gone
    out_dir = tmp_path / "examples"
    (out_dir / "data").mkdir(parents=True)
    (out_dir / "data" / "s0000.json").write_text(
        json.dumps([{"i": 5, "x": "hello", "en": "hi", "f": []}]),
        encoding="utf-8",
    )
    with pytest.raises(gen.OldCorpusError):
        gen.read_old_corpus(out_dir)


def test_read_old_corpus_raises_on_semantically_broken_index(tmp_path):
    # words: null -> OldCorpusError with instructions, not a raw AttributeError
    out_dir = _make_old_corpus(
        tmp_path / "examples",
        [{"i": 5, "x": "hello", "en": "hi", "f": []}],
        {5: "w"},
    )
    (out_dir / "index.json").write_text(
        json.dumps({"v": 1, "h": "", "words": None, "s": 1}),
        encoding="utf-8",
    )
    with pytest.raises(gen.OldCorpusError):
        gen.read_old_corpus(out_dir)


def test_read_old_corpus_present_but_empty_corpus_is_quiet_degenerate(tmp_path):
    # a present corpus with zero refs carries nothing: honest empty result
    out_dir = tmp_path / "examples"
    out_dir.mkdir()
    (out_dir / "index.json").write_text(
        json.dumps({"v": 1, "h": "", "words": {}, "s": 0}),
        encoding="utf-8",
    )
    sid_to_x, x_to_sid, sid_to_tr, max_sid, present = gen.read_old_corpus(out_dir)
    assert (sid_to_x, x_to_sid, sid_to_tr, max_sid) == ({}, {}, {}, 0)
    assert present is True


def test_read_old_corpus_raises_on_malformed_ref_element(tmp_path):
    # a non-numeric sid inside an otherwise-valid index is corruption, not leniency
    out_dir = _make_old_corpus(
        tmp_path / "examples",
        [{"i": 5, "x": "hello", "en": "hi", "f": []}],
        {5: "w"},
    )
    (out_dir / "index.json").write_text(
        json.dumps({"v": 1, "h": "", "words": {"w": {"refs": [["x", 0, 1]]}}, "s": 1}),
        encoding="utf-8",
    )
    with pytest.raises(gen.OldCorpusError):
        gen.read_old_corpus(out_dir)


def test_read_old_corpus_tolerates_empty_ref_lists(tmp_path):
    # an empty refs array = word without examples: legitimate, not corruption
    out_dir = _make_old_corpus(
        tmp_path / "examples",
        [{"i": 5, "x": "hello", "en": "hi", "f": []}],
        {5: "w"},
    )
    index = json.loads((out_dir / "index.json").read_text(encoding="utf-8"))
    index["words"]["no-examples-word"] = {"refs": []}
    (out_dir / "index.json").write_text(json.dumps(index), encoding="utf-8")
    sid_to_x, x_to_sid, sid_to_tr, max_sid, present = gen.read_old_corpus(out_dir)
    assert present is True and x_to_sid == {"hello": 5} and max_sid == 5
