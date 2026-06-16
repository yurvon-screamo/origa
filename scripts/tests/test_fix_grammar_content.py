"""Unit tests for ``fix_grammar_content.py``.

Covers the two mechanical, idempotent fixes the script applies to per-rule
JSON: bracket normalisation (mixed 「 … 」 → matching Japanese pair) and the
simplified-Chinese → Japanese-kanji word map.

The single regression guarantee the script's docstring promises is
**idempotency** — running twice yields no further changes. The script is
intentionally a context-blind replacer (see ``TestReplaceIsContextBlind``):
it does NOT understand ``❌ … ✅`` educational pairs and will rewrite a
simplified glyph on both sides of such a pair. That contract is pinned by
tests so a future contributor who adds context-awareness does so
deliberately.
"""

from __future__ import annotations

import json
from typing import Any

import pytest

from fix_grammar_content import (
    BRACKET_RE,
    STRING_FIELDS,
    WORD_MAP,
    clean_rule,
    fix_text,
    iter_rule_files,
)


# ---------------------------------------------------------------------------
# Bracket normalisation regex
# ---------------------------------------------------------------------------


class TestBracketRegex:
    def test_mixed_pair_is_fixed(self):
        assert BRACKET_RE.sub(r"「\1」", "「テスト»") == "「テスト」"

    def test_already_correct_pair_unchanged(self):
        # Matching Japanese pair — the regex must not match it (otherwise the
        # greedy capture would corrupt nested-correct text).
        assert BRACKET_RE.sub(r"「\1」", "「テスト」") == "「テスト」"

    def test_french_pair_unchanged(self):
        # Opening French « paired with closing » is a different (valid)
        # convention; the regex only fires on 「 + ».
        assert BRACKET_RE.sub(r"「\1」", "«テスト»") == "«テスト»"

    def test_multiple_mixed_pairs_all_fixed(self):
        src = "「a» and 「b»"
        assert BRACKET_RE.sub(r"「\1」", src) == "「a」 and 「b」"

    def test_empty_bracket_pair_handled(self):
        assert BRACKET_RE.sub(r"「\1」", "「»") == "「」"


# ---------------------------------------------------------------------------
# WORD_MAP application
# ---------------------------------------------------------------------------


class TestWordMap:
    @pytest.mark.parametrize(
        "simplified, japanese",
        [
            ("银行", "銀行"),
            ("书店", "本屋"),
            ("汉字", "漢字"),
            ("会议", "会議"),
            ("动词", "動詞"),
            ("形容词", "形容詞"),
            ("辞书", "辞書"),
            ("难しい", "難しい"),
            ("日记", "日記"),
        ],
    )
    def test_known_pairs_replaced(self, simplified, japanese):
        assert fix_text(simplified) == japanese

    def test_verb_fragment_书かせる_replaced(self):
        # 鈩 — the simplified 书 only ever attaches to 書く-family verbs; the
        # word map targets the full fragment, not the lone glyph.
        assert fix_text("书かせる") == "書かせる"
        assert fix_text("书かない") == "書かない"
        assert fix_text("见てもらう") == "見てもらう"

    def test_unmapped_simplified_glyph_left_alone(self):
        # The script deliberately does NOT do blind single-char swaps:
        # 书 and 本 are different words, so a lone 书 outside a known fragment
        # must be left for human review, not silently rewritten.
        assert fix_text("私は书") == "私は书"

    def test_replacement_inside_a_sentence(self):
        src = "この银行は大きい。"
        assert fix_text(src) == "この銀行は大きい。"


# ---------------------------------------------------------------------------
# fix_text: bracket + word map composition
# ---------------------------------------------------------------------------


class TestFixTextComposition:
    def test_bracket_and_wordmap_applied_together(self):
        src = "「银行»"
        assert fix_text(src) == "「銀行」"

    def test_chinese_restaurant_reinterpretation(self):
        # 餐厅 → レストラン is a semantic reinterpretation, not a glyph swap;
        # the docstring calls this out specifically.
        assert fix_text("餐厅") == "レストラン"

    def test_exchange_rate_compound(self):
        assert fix_text("汇率") == "為替レート"


# ---------------------------------------------------------------------------
# Idempotency — the script's core CI guarantee
# ---------------------------------------------------------------------------


class TestIdempotency:
    @pytest.mark.parametrize(
        "text",
        [
            "「テスト»",
            "この银行は大きい。",
            "「书店»で書かせる。",
            "形容词と難しい日记",
        ],
    )
    def test_running_twice_yields_no_further_changes(self, text):
        once = fix_text(text)
        twice = fix_text(once)
        assert once == twice, (
            f"fix_text is not idempotent on {text!r}: "
            f"once={once!r}, twice={twice!r}"
        )

    def test_clean_text_is_stable(self):
        # Already-clean Japanese text must round-trip untouched.
        clean = "「図書館」で本を書く。"
        assert fix_text(clean) == clean

    @pytest.mark.parametrize("value", sorted(WORD_MAP.values()))
    def test_no_word_map_value_contains_any_key(self, value):
        # Idempotency invariant from fix_grammar_content.py's docstring:
        # "Idempotency holds as long as no WORD_MAP value contains a
        # substring that matches any key." A future contributor adding an
        # entry whose value contains another key would break fix_text's
        # idempotency — this guard catches that at test time.
        for key in WORD_MAP:
            assert key not in value, (
                f"WORD_MAP value {value!r} contains key {key!r} — fix_text "
                f"would not be idempotent"
            )


# ---------------------------------------------------------------------------
# clean_rule: structural behavior + context preservation
# ---------------------------------------------------------------------------


def _rule_with(explanation: str, **extra: Any) -> dict[str, Any]:
    """Build a rule dict whose English block carries the given explanation."""
    en_block: dict[str, Any] = {
        "title": "T",
        "short_description": "sd",
        "explanation": explanation,
        "how_to_form": "htf",
        "examples": "ex",
        "nuances": "nu",
        "pro_tip": "pt",
    }
    ru_block = {**en_block, "title": "Т"}
    content = {"English": en_block, "Russian": ru_block}
    rule: dict[str, Any] = {"rule_id": "R", "level": "N3", "content": content}
    rule.update(extra)
    return rule


class TestCleanRule:
    def test_returns_true_when_field_changed(self):
        rule = _rule_with("この银行は")
        assert clean_rule(rule) is True
        assert rule["content"]["English"]["explanation"] == "この銀行は"

    def test_returns_false_when_nothing_changed(self):
        rule = _rule_with("この銀行は")
        assert clean_rule(rule) is False

    def test_only_string_fields_touched(self):
        # Non-string fields (lists, dicts) are left alone even if they contain
        # simplified glyphs — the script's scope is prose, not structured data.
        rule = _rule_with(
            "ok",
            examples_list=[{"银行"}],  # type: ignore[arg-type]
        )
        original_examples = rule["examples_list"]
        clean_rule(rule)
        assert rule["examples_list"] == original_examples

    def test_non_dict_rule_returns_false(self):
        assert clean_rule("not a rule") is False  # type: ignore[arg-type]
        assert clean_rule({"content": "not-an-object"}) is False

    def test_all_canonical_prose_fields_covered(self):
        # STRING_FIELDS is the contract for "fields eligible for cleanup".
        # A future author adding a prose field must remember to extend it.
        assert set(STRING_FIELDS) == {
            "short_description",
            "title",
            "explanation",
            "how_to_form",
            "examples",
            "nuances",
            "pro_tip",
        }

    def test_russian_block_also_cleaned(self):
        # Russian-language blocks can still carry simplified-Chinese
        # contamination (cross-language copy-paste), so they must be cleaned.
        rule = _rule_with("ok")
        rule["content"]["Russian"]["explanation"] = "银行银行"
        assert clean_rule(rule) is True
        assert rule["content"]["Russian"]["explanation"] == "銀行銀行"


# ---------------------------------------------------------------------------
# Educational ❌-context preservation
# ---------------------------------------------------------------------------


class TestReplaceIsContextBlind:
    """``fix_text`` is intentionally a mechanical, context-blind replacer.

    The script's docstring is explicit: it applies ``WORD_MAP`` unconditionally.
    It does NOT understand ``❌ … ✅`` educational pairs, so a contamination
    that lands inside a "wrong vs right" teaching example WILL be rewritten on
    both sides. These tests pin that contract so a future contributor who adds
    context-awareness does so deliberately rather than by accident — and so
    nobody reads the existing tests as promising a guarantee the code does not
    provide.
    """

    def test_replacement_applies_inside_educational_pair(self):
        # If a simplified glyph lands inside a ❌/✅ pair, fix_text rewrites it
        # on BOTH sides — the script has no notion of "preserve the wrong form
        # to keep the contrast". Author content that relies on this contrast
        # must avoid simplified glyphs entirely.
        src = "❌ 银行 → ✅ 銀行"
        assert fix_text(src) == "❌ 銀行 → ✅ 銀行"

    def test_chinese_meta_glyph_outside_map_left_alone(self):
        # 呵呵 is not in WORD_MAP, so it survives — NOT because the script
        # recognizes it as meta-content, but simply because no rule matches.
        # The distinction matters: a future WORD_MAP entry could change this.
        assert fix_text("呵呵という音") == "呵呵という音"

    def test_pair_with_no_mapped_glyphs_unchanged(self):
        # Both sides use correct Japanese with no WORD_MAP keys present, so the
        # string round-trips. This is the common case in real rule files.
        src = "❌ 食べれる → ✅ 食べられる"
        assert fix_text(src) == src


# ---------------------------------------------------------------------------
# iter_rule_files: directory traversal contract
# ---------------------------------------------------------------------------


class TestIterRuleFiles:
    def test_underscore_subdirs_excluded(self, tmp_path):
        (tmp_path / "_example").mkdir()
        (tmp_path / "_example" / "rule_01.json").write_text("{}", encoding="utf-8")
        (tmp_path / "n3_01").mkdir()
        (tmp_path / "n3_01" / "rule_01.json").write_text("{}", encoding="utf-8")

        files = list(iter_rule_files(tmp_path))
        names = [p.parent.name for p in files]
        assert names == ["n3_01"]

    def test_only_rule_prefixed_json_yielded(self, tmp_path):
        (tmp_path / "n3_01").mkdir()
        (tmp_path / "n3_01" / "rule_01.json").write_text("{}", encoding="utf-8")
        (tmp_path / "n3_01" / "manifest.json").write_text("{}", encoding="utf-8")
        (tmp_path / "n3_01" / "notes.txt").write_text("ignore me", encoding="utf-8")

        files = list(iter_rule_files(tmp_path))
        assert [p.name for p in files] == ["rule_01.json"]

    def test_empty_dir_yields_nothing(self, tmp_path):
        assert list(iter_rule_files(tmp_path)) == []


# ---------------------------------------------------------------------------
# End-to-end: write a temp rule file and run clean_rule against its contents
# (validates the JSON I/O contract the script relies on).
# ---------------------------------------------------------------------------


def test_clean_rule_roundtrip_through_json(tmp_path):
    rule = _rule_with("「银行»で书かせる")
    path = tmp_path / "n3_01" / "rule_01.json"
    path.parent.mkdir(parents=True)
    path.write_text(json.dumps(rule, ensure_ascii=False), encoding="utf-8")

    loaded = json.loads(path.read_text(encoding="utf-8"))
    assert clean_rule(loaded) is True
    assert loaded["content"]["English"]["explanation"] == "「銀行」で書かせる"
