"""Unit tests for ``scan_hybrid_words.py``.

The scanner is a regex-based detector, the most regression-prone class of
logic: a tiny change to a character class can silently flip a false
positive/negative. These tests pin the detection invariants so a future
contributor who edits the regexes does so deliberately.
"""

from __future__ import annotations

import json

import pytest

from scan_hybrid_words import (
    GLUED_TOKEN,
    GLUED_TRANSITION,
    HYPHEN_COMPOUND,
    HybridHit,
    main,
    scan_glued,
    scan_hyphenated,
)


# ---------------------------------------------------------------------------
# Test data builders
# ---------------------------------------------------------------------------


def _rule(rule_id: str, **russian_fields: str) -> dict:
    """Build a minimal rule with only the Russian fields given."""
    return {"rule_id": rule_id, "content": {"Russian": russian_fields}}


def _grammar(*rules: dict) -> dict:
    return {"grammar": list(rules)}


# ---------------------------------------------------------------------------
# GLUED_TRANSITION regex
# ---------------------------------------------------------------------------


class TestGluedTransitionRegex:
    @pytest.mark.parametrize(
        "word",
        ["Пotentialная", "пotentialная", "итиidan", "фразa", "Godан", "arrangeет"],
    )
    def test_detects_real_typos(self, word: str):
        assert GLUED_TRANSITION.search(word), f"expected hybrid: {word!r}"

    @pytest.mark.parametrize(
        "word",
        ["Потенциальная", "потенциальная", "итидан", "фраза", "годан", "JLPT", "AI", ""],
    )
    def test_does_not_flag_single_script(self, word: str):
        assert GLUED_TRANSITION.search(word) is None, f"not a hybrid: {word!r}"


# ---------------------------------------------------------------------------
# GLUED_TOKEN excludes hyphen (the key design choice)
# ---------------------------------------------------------------------------


class TestGluedTokenExcludesHyphen:
    def test_hyphenated_compound_is_split_not_glued(self):
        # "te-форма" must tokenize as ["te", "форма"], neither of which is glued.
        tokens = GLUED_TOKEN.findall("te-форма")
        assert tokens == ["te", "форма"]
        assert not any(GLUED_TRANSITION.search(t) for t in tokens)

    def test_glued_token_has_no_hyphen(self):
        for token in GLUED_TOKEN.findall("V-словарная и Adj-основа"):
            assert "-" not in token


# ---------------------------------------------------------------------------
# HYPHEN_COMPOUND regex
# ---------------------------------------------------------------------------


class TestHyphenCompoundRegex:
    @pytest.mark.parametrize(
        "text",
        ["V-словарная", "te-форма", "Adj-основа", "Na-прилагательное", "nai-основе"],
    )
    def test_matches_legitimate_notation(self, text: str):
        assert HYPHEN_COMPOUND.fullmatch(text)

    def test_pure_cyrillic_hyphenated_not_mixed(self):
        # Both sides Cyrillic -> not a *mixed* compound (acceptor filters it).
        m = HYPHEN_COMPOUND.fullmatch("как-то")
        assert m is not None  # regex matches structurally...
        # ...but the scan_hyphenated acceptor rejects it (see TestScanHyphenated).

    def test_no_hyphen_no_compound(self):
        assert HYPHEN_COMPOUND.fullmatch("Потенциальная") is None


# ---------------------------------------------------------------------------
# scan_glued / scan_hyphenated on synthetic grammar
# ---------------------------------------------------------------------------


class TestScanGlued:
    def test_finds_glued_hybrid_with_context(self):
        data = _grammar(
            _rule("R1", explanation="Форма **Пotentialная** глагола."),
        )
        hits = scan_glued(data)
        assert len(hits) == 1
        hit = hits[0]
        assert hit.rule_id == "R1"
        assert hit.field == "explanation"
        assert hit.word == "Пotentialная"
        assert "Пotentialная" in hit.context

    def test_ignores_legitimate_hyphenated_notation(self):
        data = _grammar(
            _rule("R1", how_to_form="Бери V-словарная и te-форму."),
        )
        assert scan_glued(data) == []

    def test_ignores_standalone_latin_terms(self):
        data = _grammar(
            _rule("R1", explanation="Это правило уровня JLPT и N4."),
        )
        assert scan_glued(data) == []

    def test_scans_all_content_fields(self):
        data = _grammar(
            _rule(
                "R1",
                title="фразa",  # hybrid in title
                pro_tip="См. также итиidan",  # hybrid in pro_tip
            ),
        )
        words = {hit.word for hit in scan_glued(data)}
        assert words == {"фразa", "итиidan"}

    def test_skips_empty_fields(self):
        data = _grammar(_rule("R1", explanation="", nuances=None))  # type: ignore[arg-type]
        assert scan_glued(data) == []


class TestScanHyphenated:
    def test_finds_legitimate_mixed_compounds(self):
        data = _grammar(
            _rule("R1", how_to_form="V-словарная и Adj-основа"),
        )
        words = {hit.word for hit in scan_hyphenated(data)}
        assert words == {"V-словарная", "Adj-основа"}

    def test_rejects_pure_cyrillic_hyphenated(self):
        data = _grammar(_rule("R1", explanation="как-то так"))
        assert scan_hyphenated(data) == []

    def test_does_not_affect_glued_exit_logic(self):
        # Hyphenated compounds alone must NOT look like glued hybrids.
        data = _grammar(_rule("R1", how_to_form="te-форма"))
        assert scan_glued(data) == []


# ---------------------------------------------------------------------------
# HybridHit contract
# ---------------------------------------------------------------------------


class TestHybridHit:
    def test_named_tuple_field_order(self):
        hit = HybridHit("R1", "explanation", "Пotentialная", "ctx")
        assert hit.rule_id == "R1"
        assert hit.field == "explanation"
        assert hit.word == "Пotentialная"
        assert hit.context == "ctx"


# ---------------------------------------------------------------------------
# main() exit-code contract (CI guard)
# ---------------------------------------------------------------------------


class TestMainExitCode:
    def test_returns_zero_when_clean(self, tmp_path, monkeypatch):
        store = tmp_path / "clean.json"
        store.write_text(
            json.dumps(_grammar(_rule("R1", explanation="Потенциальная форма")), ensure_ascii=False),
            encoding="utf-8",
        )
        monkeypatch.setattr("sys.argv", ["scan_hybrid_words.py", "--file", str(store)])
        assert main() == 0

    def test_returns_one_when_glued_present(self, tmp_path, monkeypatch):
        store = tmp_path / "dirty.json"
        store.write_text(
            json.dumps(_grammar(_rule("R1", explanation="Пotentialная форма")), ensure_ascii=False),
            encoding="utf-8",
        )
        monkeypatch.setattr("sys.argv", ["scan_hybrid_words.py", "--file", str(store)])
        assert main() == 1

    def test_returns_two_when_file_missing(self, tmp_path, monkeypatch):
        missing = tmp_path / "absent.json"
        monkeypatch.setattr("sys.argv", ["scan_hybrid_words.py", "--file", str(missing)])
        assert main() == 2
