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
