"""Unit tests for ``build_examples.py`` example-selection fixes.

Covers the two content-side corrections of the wrong-furigana spike
(20261109082311-9i1um6f): junk Tanaka id readings never become ruby pairs,
and kanji words are only backed by sentences where the word actually
surfaces (an offset-less kana-variant match — the auxiliary てしまう usage
of 仕舞う — does not demonstrate the card word).
"""

from __future__ import annotations

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
            __import__("json").dumps(part, ensure_ascii=False, separators=(",", ":")),
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
        __import__("json").dumps(index, ensure_ascii=False, separators=(",", ":")),
        encoding="utf-8",
    )
    return tmp_path


def test_read_old_corpus_empty_when_no_previous(tmp_path):
    out_dir = tmp_path / "examples"
    out_dir.mkdir()
    sid_to_x, x_to_sid, sid_to_tr, max_sid = gen.read_old_corpus(out_dir)
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
    sid_to_x, x_to_sid, sid_to_tr, max_sid = gen.read_old_corpus(out_dir)
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
    sid_to_x, x_to_sid, sid_to_tr, max_sid = gen.read_old_corpus(out_dir)
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
    sid_to_x, x_to_sid, sid_to_tr, max_sid = gen.read_old_corpus(out_dir)
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
