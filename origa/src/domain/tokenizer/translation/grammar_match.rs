//! Keyword- and format-map-based resolution of grammar rules for tokens.
//!
//! Owns `GrammarMatch` — the rule match carried through the translation
//! pipeline — plus the two strategies that produce it: keyword matching on
//! the token surface (with katakana→hiragana normalization) and format-map
//! matching against the conjugated base form. Split out of the former
//! translation.rs god-file (issue #796).

use crate::dictionary::grammar::{GRAMMAR_RULES, GrammarRule};
use crate::domain::JapaneseChar;
use crate::domain::NativeLanguage;
use crate::domain::grammar::find_format_map_matches;
use crate::domain::katakana_to_hiragana;
use crate::domain::tokenizer::TokenInfo;

/// A grammar rule match carried through the resolution pipeline: the rule
/// title (surfaced as `grammar_label`) plus the rule's `short_description`
/// (surfaced as `grammar_description`). Keeping both together avoids a second
/// rule lookup and ensures the description always corresponds to the matched
/// rule rather than a re-resolved one.
pub(super) struct GrammarMatch {
    pattern: String,
    short_description: String,
}

impl GrammarMatch {
    pub(super) fn from_rule(rule: &GrammarRule, native_language: &NativeLanguage) -> Self {
        let content = rule.content(native_language);
        Self {
            pattern: content.pattern().to_string(),
            short_description: content.short_description().to_string(),
        }
    }
}

/// Splits a `GrammarMatch` into the two `TokenTranslation` fields. The
/// description is `None` when the matched rule has an empty `short_description`
/// so the popup does not render an empty line.
pub(super) fn split_grammar_fields(
    grammar: Option<GrammarMatch>,
) -> (Option<String>, Option<String>) {
    match grammar {
        Some(m) => {
            let description = if m.short_description.is_empty() {
                None
            } else {
                Some(m.short_description)
            };
            (Some(m.pattern), description)
        },
        None => (None, None),
    }
}

fn match_grammar_keyword(
    rules: &'static [GrammarRule],
    surface: &str,
    surface_hiragana: &str,
    original_text: &str,
    original_hiragana: &str,
) -> Option<&'static GrammarRule> {
    for rule in rules.iter() {
        let keyword_groups = rule.keywords();
        if keyword_groups.is_empty() {
            continue;
        }

        // Match the token surface against a keyword, accepting either the
        // raw surface (hiragana input) or its katakana→hiragana equivalent
        // (katakana input like ベキ → べき). This is a monotonic OR extension:
        // it can only add matches for katakana forms that normalize to an
        // existing keyword, never drop a hiragana match.
        let token_matches_some_group = keyword_groups.iter().any(|group| {
            group
                .iter()
                .any(|kw| surface == kw || surface_hiragana == kw)
        });
        if !token_matches_some_group {
            continue;
        }

        // Every group must contribute at least one keyword present in the original
        // text. This converges with detect_keyword_rules AND-semantics and prevents
        // bare-single-keyword false positives (e.g. topic は falsely matching the
        // ～は～ほど～ない rule when ほど is absent from the sentence). The katakana
        // equivalent of the text is also accepted so a katakana-written phrase
        // (ベキ) still covers its hiragana keyword (べき).
        let text_covers_all_groups = keyword_groups.iter().all(|group| {
            !group.is_empty()
                && group
                    .iter()
                    .any(|kw| original_text.contains(kw) || original_hiragana.contains(kw))
        });
        if text_covers_all_groups {
            return Some(rule);
        }
    }
    None
}

pub(super) fn resolve_grammar_match(
    token: &TokenInfo,
    native_language: &NativeLanguage,
    original_text: &str,
    original_hiragana: &str,
) -> Option<GrammarMatch> {
    let rules = GRAMMAR_RULES.get()?;
    let surface = token.orthographic_surface_form();
    let surface_hiragana = katakana_to_hiragana(surface);
    let base = token.orthographic_base_form();
    let pos = token.part_of_speech();
    let is_vocab = pos.is_vocabulary_word();

    // Keyword matching for non-vocabulary tokens (particles, auxiliaries, etc.)
    // — and for vocabulary tokens whose surface form equals the dictionary
    // base form AND matches a grammar keyword. The dictionary-form gate
    // prevents conjugated content words (e.g. 食べた, 食べて) from getting a
    // grammar_label via keyword lookup while still recognizing grammar nouns
    // like べき, はず, ところ that Lindera classifies as Noun.
    // See issue #178 P-6 for context.
    let eligible_for_keyword_match = !is_vocab || surface == base;
    if eligible_for_keyword_match {
        if let Some(rule) = match_grammar_keyword(
            rules,
            surface,
            &surface_hiragana,
            original_text,
            original_hiragana,
        ) {
            return Some(GrammarMatch::from_rule(rule, native_language));
        }
    }

    if is_vocab && surface != base {
        // Conjugated auxiliaries attached to a content stem (e.g. すぎ inside 強すぎて,
        // base すぎる, classified by Lindera as Verb) must still resolve to their
        // grammar keyword before format_map matching — otherwise a more generic
        // rule (～て) wins on the te-form surface.
        if let Some(rule) = match_grammar_keyword(
            rules,
            surface,
            &surface_hiragana,
            original_text,
            original_hiragana,
        ) {
            return Some(GrammarMatch::from_rule(rule, native_language));
        }

        let mut matches = find_format_map_matches(base, pos, original_text, rules);

        // Lindera prefers the kanji lemma as ``base`` even when the user's
        // phrase is written entirely in hiragana. ``find_format_map_matches``
        // formats the base into its conjugated surface (e.g. ``分かる`` →
        // ``分かって``) and checks the original text — but the kanji-formatted
        // string never appears in a hiragana phrase, so the match silently
        // fails. Retry the same chain with the hiragana-equivalent base
        // derived from the katakana reading so ``分かる`` becomes ``わかる`` and
        // the formatted ``わかって`` does occur inside ``わかってるなら``.
        let hiragana_base = if base.chars().any(|c| c.is_kanji()) {
            Some(katakana_to_hiragana(token.phonological_base_form()))
        } else {
            None
        };

        if let Some(ref hira) = hiragana_base {
            matches.extend(find_format_map_matches(hira, pos, original_text, rules));
        }

        // Score each candidate by the longest format it produces across both
        // bases (kanji + hiragana-equivalent) so rules that only match through
        // the hiragana retry are not penalised when ``rule.format(base, pos)``
        // happens to error on the kanji lemma.
        if let Some(best) = matches.into_iter().max_by_key(|rule| {
            let kanji_len = rule.format(base, pos).map_or(0, |f| f.len());
            let hira_len = hiragana_base
                .as_ref()
                .map(|h| rule.format(h, pos).map_or(0, |f| f.len()))
                .unwrap_or(0);
            kanji_len.max(hira_len)
        }) {
            return Some(GrammarMatch::from_rule(best, native_language));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_grammar_fields_returns_both_when_description_present() {
        let m = GrammarMatch {
            pattern: "～test".to_string(),
            short_description: "desc".to_string(),
        };
        let (label, description) = split_grammar_fields(Some(m));
        assert_eq!(label.as_deref(), Some("～test"));
        assert_eq!(description.as_deref(), Some("desc"));
    }
    #[test]
    fn split_grammar_fields_drops_description_when_empty() {
        let m = GrammarMatch {
            pattern: "～test".to_string(),
            short_description: String::new(),
        };
        let (label, description) = split_grammar_fields(Some(m));
        assert_eq!(label.as_deref(), Some("～test"));
        assert!(
            description.is_none(),
            "empty short_description must surface as None so the popup renders no line"
        );
    }
    #[test]
    fn split_grammar_fields_returns_none_none_when_no_match() {
        let (label, description) = split_grammar_fields(None);
        assert!(label.is_none());
        assert!(description.is_none());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::super::lookup_tokens_translations;
    use super::super::test_env::ensure_dictionaries;
    use crate::domain::NativeLanguage;
    use crate::domain::tokenizer::PartOfSpeech;
    use crate::domain::tokenizer::tokenize_text;

    // Standalone grammar markers (particle / noun / auxiliary) that previously
    // carried no grammar_label because their rule had only a format_map and no
    // keywords. Keyword matching fires on the token surface, so once a keyword
    // is added the marker token itself is labeled instead of staying bare.
    fn assert_marker_has_grammar_label(text: &str, marker_surface: &str, expected_label: &str) {
        ensure_dictionaries();
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let marker = results
            .iter()
            .find(|t| t.surface_form == marker_surface)
            .unwrap_or_else(|| panic!(
                "「{marker_surface}」 token should exist in '{text}', got surfaces: {surfaces:?}",
                surfaces = results.iter().map(|t| t.surface_form.as_str()).collect::<Vec<_>>()
            ));
        assert_eq!(
            marker.grammar_label.as_deref(),
            Some(expected_label),
            "「{marker_surface}」 in '{text}' grammar_label mismatch, token: {marker:?}",
        );
    }
    #[test]
    fn should_resolve_grammar_label_for_particle() {
        ensure_dictionaries();
        let text = "東京から大阪まで";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let kara_particle = results.iter().find(|t| t.surface_form == "から");
        assert!(
            kara_particle.is_some(),
            "「から」particle should exist in tokens"
        );
        let kara = kara_particle.unwrap();
        assert!(
            kara.grammar_label.is_some(),
            "「から」should have a grammar label, got: {:?}",
            kara
        );
    }
    #[test]
    fn should_detect_te_form_grammar_label() {
        ensure_dictionaries();
        let text = "食べて";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let verb_token = results.iter().find(|t| t.base_form == "食べる");
        assert!(verb_token.is_some(), "Should find 食べる token");
        let verb = verb_token.unwrap();
        assert!(
            verb.grammar_label.is_some(),
            "「食べて」verb should have grammar_label for te-form, got: {:?}",
            verb
        );
    }
    #[test]
    fn should_detect_tai_form_grammar_label() {
        ensure_dictionaries();
        let text = "食べたい";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let verb_token = results.iter().find(|t| t.base_form == "食べる");
        assert!(verb_token.is_some(), "Should find 食べる token");
        let verb = verb_token.unwrap();
        assert!(
            verb.grammar_label.is_some(),
            "「食べたい」verb should have grammar_label for tai-form, got: {:?}",
            verb
        );
    }
    #[test]
    fn should_detect_ta_form_grammar_label() {
        ensure_dictionaries();
        let text = "食べた";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let verb_token = results.iter().find(|t| t.base_form == "食べる");
        assert!(verb_token.is_some(), "Should find 食べる token");
        let verb = verb_token.unwrap();
        assert!(
            verb.grammar_label.is_some(),
            "「食べた」verb should have grammar_label for ta-form, got: {:?}",
            verb
        );
    }
    #[test]
    fn should_detect_nai_form_grammar_label() {
        ensure_dictionaries();
        let text = "食べない";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let verb_token = results.iter().find(|t| t.base_form == "食べる");
        assert!(verb_token.is_some(), "Should find 食べる token");
        let verb = verb_token.unwrap();
        assert!(
            verb.grammar_label.is_some(),
            "「食べない」verb should have grammar_label for nai-form, got: {:?}",
            verb
        );
    }
    #[test]
    fn should_not_set_grammar_label_for_base_form() {
        ensure_dictionaries();
        let text = "食べる";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);
        let verb_token = results.iter().find(|t| t.base_form == "食べる");
        assert!(verb_token.is_some(), "Should find 食べる token");
        assert!(
            verb_token.unwrap().grammar_label.is_none(),
            "Base form should NOT have grammar_label"
        );
    }
    #[test]
    fn should_detect_beki_grammar_label() {
        ensure_dictionaries();
        let text = "うん、あるべきものをあるべき形カタチに";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

        let beki = results.iter().find(|t| t.surface_form == "べき");
        assert!(
            beki.is_some(),
            "「べき」token should exist, tokens: {:?}",
            results.iter().map(|t| &t.surface_form).collect::<Vec<_>>()
        );
        let beki = beki.unwrap();
        assert!(
            beki.grammar_label
                .as_ref()
                .is_some_and(|label| label.contains("べき")),
            "「べき」should carry the べきだ grammar_label, got: {:?}",
            beki
        );
    }
    // Regression for issue #178 P-6: a standalone べき token is classified
    // by Lindera as Noun (助動詞語幹). The old `!is_vocab` gate skipped keyword
    // matching entirely for Noun tokens, so べき never received a grammar_label.
    // The fix allows keyword lookup when surface == base so dictionary-form
    // grammar nouns (べき, はず, ところ, etc.) still resolve to a grammar rule.
    #[test]
    fn tokenize_standalone_beki_returns_grammar_label() {
        ensure_dictionaries();
        let text = "べき";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

        let beki = results
            .iter()
            .find(|t| t.surface_form == "べき")
            .expect("「べき」standalone token should exist");
        assert!(
            beki.grammar_label
                .as_ref()
                .is_some_and(|label| label.contains("べき")),
            "standalone「べき」should carry the べきだ grammar_label, got: {:?}",
            beki
        );
    }
    // べき carries the べきだ grammar rule, whose short_description is non-empty
    // (RU «Следует…; должен…», EN «Should…»). The popup must surface that
    // description as grammar_description so べき is no longer shown title-only.
    // Asserting the description content (not just non-empty) guards against a
    // wiring error that attached the wrong field.
    #[test]
    fn tokenize_standalone_beki_returns_grammar_description() {
        ensure_dictionaries();
        let text = "べき";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

        let beki = results
            .iter()
            .find(|t| t.surface_form == "べき")
            .expect("「べき」standalone token should exist");
        assert!(
            beki.grammar_label
                .as_ref()
                .is_some_and(|label| label.contains("べき")),
            "standalone「べき」should carry the べきだ grammar_label, got: {:?}",
            beki
        );
        assert!(
            beki.grammar_description
                .as_ref()
                .is_some_and(|d| d.contains("Should")),
            "standalone「べき」should carry the べきだ grammar_description, got: {:?}",
            beki.grammar_description
        );
    }
    // Katakana ベキ (e.g. emphasis or stylised writing) must still match the
    // べきだ rule: match_grammar_keyword normalises the katakana surface to
    // hiragana (ベキ → べき) and the katakana input text to cover the keyword.
    // Robust to whether Lindera keeps ベキ as katakana or normalises to べき.
    #[test]
    fn should_match_katakana_beki_via_normalization() {
        ensure_dictionaries();
        let text = "ベキ";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

        let beki = results
            .iter()
            .find(|t| t.surface_form == "ベキ" || t.surface_form == "べき")
            .expect("「ベキ/べき」token should exist");
        assert!(
            beki.grammar_label
                .as_ref()
                .is_some_and(|label| label.contains("べき")),
            "katakana「ベキ」should match the べき grammar rule via hiragana normalization, got: {:?}",
            beki
        );
    }
    // The katakana→hiragana normalization is a monotonic OR extension: it can
    // only add matches for katakana forms that normalise to an existing keyword.
    // A katakana content word (テスト, "test") whose hiragana equivalent (てすと)
    // is not a grammar keyword must NOT receive a spurious grammar_label.
    #[test]
    fn should_not_overmatch_katakana_content_word() {
        ensure_dictionaries();
        let text = "テスト";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

        let tesuto = results
            .iter()
            .find(|t| t.surface_form == "テスト")
            .expect("「テスト」token should exist");
        assert!(
            tesuto.grammar_label.is_none(),
            "katakana content word「テスト」must not receive a spurious grammar_label via normalization, got: {:?}",
            tesuto.grammar_label
        );
    }
    // Negative test for the relaxed grammar_label gate: a common content
    // noun in dictionary form (猫, "cat") must NOT pick up a spurious
    // grammar_label just because surface == base. Guards against the
    // relaxation over-matching content vocabulary that happens to collide
    // with a grammar keyword.
    #[test]
    fn tokenize_common_content_noun_has_no_grammar_label() {
        ensure_dictionaries();
        let text = "猫";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::English, text);

        let neko = results
            .iter()
            .find(|t| t.surface_form == "猫")
            .expect("「猫」token should exist");
        assert!(
            neko.grammar_label.is_none(),
            "common noun「猫」must not receive a spurious grammar_label, got: {:?}",
            neko
        );
    }
    // --- Phrase 3: 大丈夫でやがる ---
    // Before the fix, やがる (derogatory auxiliary "to do, damn it") had a
    // vocabulary entry but the 〜てやがる / 〜でやがる construction was never flagged
    // with a grammar_label. This test locks in the grammar rule that labels it.
    #[test]
    fn should_label_yagaru_derogatory_auxiliary_grammar() {
        ensure_dictionaries();
        let text = "大丈夫でやがる";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let yagaru = results
            .iter()
            .find(|t| t.surface_form == "やがる")
            .expect("「やがる」auxiliary token should exist");
        assert!(
            yagaru.grammar_label.is_some(),
            "〜やがる derogatory construction should carry a grammar_label, got: {:?}",
            yagaru
        );
    }
    // --- Phrase 4: 二度も言わんでええの！ ---
    // Before the fix, the colloquial negative 「ん」 (contraction of ぬ) was
    // tokenized with base_form 'ぬ'. Translation lookup went through base_form,
    // and 'ぬ' was absent from the vocabulary dictionary, so translation was None
    // even though 「ん」 itself was present. No grammar rule labeled the ん negative
    // either. The same ん/ぬ root cause recurred in phrase 7 (くれませんか). This
    // test locks in the ぬ vocabulary entry that resolves it.
    #[test]
    fn should_label_or_translate_n_negative_auxiliary() {
        ensure_dictionaries();
        let text = "二度も言わんでええの！";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let n = results
            .iter()
            .find(|t| t.surface_form == "ん" && t.pos == PartOfSpeech::AuxiliaryVerb)
            .expect("「ん」auxiliary token should exist");
        assert!(
            n.translation.is_some() || n.grammar_label.is_some(),
            "negative 「ん」auxiliary should carry a translation or grammar_label, got base={:?} translation={:?} grammar={:?}",
            n.base_form,
            n.translation,
            n.grammar_label
        );
    }
    // --- Phrase 5: 雨の音強すぎて、テレビの音聞こえない ---
    // Before the fix, the 〜すぎる ("too much") grammar rule only matched via
    // format_map (強い → 強すぎる), but the text contained the te-form 強すぎて,
    // which format_map never produces — so the "too much" meaning was never
    // detected on either the 強 stem or the すぎ token. This test locks in the
    // keyword-based すぎる detection that resolves conjugated auxiliaries.
    #[test]
    fn should_detect_sugiru_too_much_for_tsuyosugite() {
        ensure_dictionaries();
        let text = "雨の音強すぎて、テレビの音聞こえない";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let has_sugiru_label = results.iter().any(|t| {
            t.grammar_label
                .as_deref()
                .is_some_and(|label| label.contains("すぎる"))
        });
        assert!(
            has_sugiru_label,
            "〜すぎる (too much) should be detected for 強すぎて, labels: {:?}",
            results
                .iter()
                .filter(|t| t.grammar_label.is_some())
                .map(|t| (&t.surface_form, &t.grammar_label))
                .collect::<Vec<_>>()
        );
    }
    // --- Phrase 10: ついてますし (the し reason particle) ---
    // Before the fix, 〜し (reason/listing particle "besides / and what's more")
    // existed as a grammar rule, but that rule had neither keywords nor a
    // format_map, so it could never be matched by resolve_grammar_label.
    // Additionally 「し」 base_form resolved to the する-stem homonym, yielding the
    // verb meaning "to do" instead of the particle. This test locks in the
    // keyword activation + homonym blacklist that resolve it.
    #[test]
    fn should_label_shi_reason_particle_grammar() {
        ensure_dictionaries();
        let text = "ストーブついてますし、大丈夫ですよ";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let shi = results
            .iter()
            .find(|t| t.surface_form == "し" && t.pos == PartOfSpeech::Particle)
            .expect("「し」particle token should exist");
        assert!(
            shi.grammar_label.is_some(),
            "〜し reason particle should carry a grammar_label, got translation={:?} grammar={:?}",
            shi.translation,
            shi.grammar_label
        );
    }
    // --- Cross-cutting false positive: plain topic は (phrases 1 & 2) ---
    // Before the fix, the grammar rule ～は～ほど～ない listed 「は」 itself as a
    // keyword, so EVERY topic particle は was mislabeled with that pattern even in
    // a plain ～は～です sentence. This test locks in the AND-semantics fix that
    // prevents the bare-keyword false positive.
    #[test]
    fn should_not_mislabel_plain_ha_particle_as_hahodo_nai() {
        ensure_dictionaries();
        let text = "私は学生です";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let ha = results
            .iter()
            .find(|t| t.surface_form == "は" && t.pos == PartOfSpeech::Particle)
            .expect("「は」particle token should exist");
        let mislabeled = ha
            .grammar_label
            .as_deref()
            .is_some_and(|label| label.contains("ほど"));
        assert!(
            !mislabeled,
            "plain topic 「は」must not get the ～は～ほど～ない label, got: {:?}",
            ha.grammar_label
        );
    }
    // ～すぎる keyword rule must also resolve for adjective-stem compounds:
    // 複雑すぎる is tokenized by Lindera as 複雑 (NaAdjective) + すぎる, and the
    // auxiliary must surface the ～すぎる grammar_label even though format_map
    // would otherwise match the longer phrase.
    #[test]
    fn should_detect_sugiru_label_for_compound_word() {
        ensure_dictionaries();
        let text = "この問題は複雑すぎる";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let has_sugiru_label = results.iter().any(|t| {
            t.grammar_label
                .as_deref()
                .is_some_and(|label| label.contains("すぎる"))
        });
        assert!(
            has_sugiru_label,
            "～すぎる should be detected for 複雑すぎる, labels: {:?}",
            results
                .iter()
                .filter(|t| t.grammar_label.is_some())
                .map(|t| (&t.surface_form, &t.grammar_label))
                .collect::<Vec<_>>()
        );
    }
    // ～すぎる detection for noun-stem compounds in te-form: 複雑すぎて is split
    // by Lindera into 複雑 (Noun) + すぎ (Verb, base 過ぎる) + て. The sugiru
    // rule's format_map covers only Verb and IAdjective (no Noun entry), so
    // format("複雑", Noun) errors before any matching can happen. The keyword
    // whitelist on the rule catches the auxiliary token すぎ. This test locks
    // in that noun-stem compounds in te-form are still labeled correctly via
    // the keyword fallback.
    #[test]
    fn should_detect_sugiru_te_form_for_noun_stem_compound() {
        ensure_dictionaries();
        let text = "この問題は複雑すぎて困っている";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let has_sugiru = results.iter().any(|t| {
            t.grammar_label
                .as_deref()
                .is_some_and(|label| label.contains("すぎる"))
        });
        assert!(
            has_sugiru,
            "すぎる should be detected for 複雑すぎて, labels: {:?}",
            results
                .iter()
                .filter(|t| t.grammar_label.is_some())
                .map(|t| (&t.surface_form, &t.grammar_label))
                .collect::<Vec<_>>()
        );
    }
    #[test]
    fn should_label_te_particle_via_keyword() {
        assert_marker_has_grammar_label("食べて飲んで", "て", "～て");
    }
    #[test]
    fn should_label_tara_conditional_via_keyword() {
        assert_marker_has_grammar_label("食べたら", "たら", "～たら");
    }
    #[test]
    fn should_label_okage_thanks_to_via_keyword() {
        assert_marker_has_grammar_label("彼のおかげで成功した", "おかげ", "～おかげで");
    }
    #[test]
    fn should_label_sei_because_of_via_keyword() {
        assert_marker_has_grammar_label("失敗は彼のせいだ", "せい", "～せいで");
    }
    #[test]
    fn should_label_totan_the_moment_via_keyword() {
        assert_marker_has_grammar_label("ドアを開けたとたん猫が逃げた", "とたん", "～たとたん");
    }
    #[test]
    fn should_label_san_honorific_suffix_via_keyword() {
        ensure_dictionaries();
        let text = "田中さん";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let san = results
            .iter()
            .find(|t| t.surface_form == "さん")
            .expect("「さん」suffix token should exist");
        assert!(
            san.grammar_label
                .as_deref()
                .is_some_and(|l| l.contains("さん")),
            "「さん」should carry the ～さん honorific grammar_label via keyword, got: {:?}",
            san
        );
    }
    #[test]
    fn should_label_itsu_question_word_via_keyword() {
        ensure_dictionaries();
        let text = "いつ行きますか";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let itsu = results
            .iter()
            .find(|t| t.surface_form == "いつ")
            .expect("「いつ」token should exist");
        assert!(
            itsu.grammar_label
                .as_deref()
                .is_some_and(|l| l.contains("いつ")),
            "「いつ」should carry the いつ question-word grammar_label via keyword, got: {:?}",
            itsu
        );
    }
    // i-adjective ku-form (adverbial): 早い → 早く. The biggest single gap in
    // the corpus (~10k hits). The ku-form turns an i-adjective into an adverb.
    #[test]
    fn should_label_i_adjective_ku_form_adverbial() {
        ensure_dictionaries();
        let text = "もっと早く歩いて";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let hayaku = results
            .iter()
            .find(|t| t.surface_form == "早く")
            .expect("「早く」ku-form token should exist");
        assert!(
            hayaku.grammar_label.as_deref().is_some_and(|l| l == "～く"),
            "i-adjective ku-form「早く」should carry the bare ～く pattern, got: {:?}",
            hayaku
        );
    }
    // i-adjective past (katta-form): 早い → 早かった / 早かっ. AdjectiveToKatta
    // is implemented but exposed by no rule (~5k hits in the corpus).
    #[test]
    fn should_label_i_adjective_katta_past_form() {
        ensure_dictionaries();
        let text = "昨日の天気が本当に良かった";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let yokatta = results
            .iter()
            .find(|t| t.base_form == "良い" && t.surface_form != "良い")
            .expect("conjugated「良い」token should exist");
        assert!(
            yokatta
                .grammar_label
                .as_deref()
                .is_some_and(|l| l == "～かった")
                && yokatta
                    .grammar_description
                    .as_deref()
                    .is_some_and(|d| d.contains("Прошедшее")),
            "i-adjective past form of「良い」should carry the past grammar_label, got: {:?}",
            yokatta
        );
    }
    // i-adjective te-form (kute-form): 早い → 早くて. AdjectiveToKute is
    // implemented but the ～て／～くて／～で rule carries no format_map, so the
    // i-adjective branch never matches.
    #[test]
    fn should_label_i_adjective_kute_te_form() {
        ensure_dictionaries();
        let text = "このケーキは美味しくて止まらない";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let oishikute = results
            .iter()
            .find(|t| t.base_form == "美味しい" && t.surface_form != "美味しい")
            .expect("conjugated「美味しい」token should exist");
        assert!(
            oishikute
                .grammar_label
                .as_deref()
                .is_some_and(|l| l.contains("て")),
            "i-adjective te-form「美味しくて」should carry the te-form grammar_label, got: {:?}",
            oishikute
        );
    }
    // AdjectiveToGaru — i-adj stem + がる ("to feel/seem that way").
    // 怖がってる (怖い→怖がる→怖がって). The te-variant is matched via
    // formatted_conjugation_variants.
    #[test]
    fn should_label_i_adjective_garu_form() {
        ensure_dictionaries();
        let text = "周りが怖がってるぞ";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let kowa = results
            .iter()
            .find(|t| t.surface_form == "怖")
            .expect("「怖」token should exist");
        assert!(
            kowa.grammar_label.is_some(),
            "i-adjective がる-form「怖」should carry a grammar_label, got: {:?}",
            kowa
        );
    }
    // AdjectiveToKunai — i-adj negative form (高い→高くない).
    #[test]
    fn should_label_i_adjective_kunai_negative_form() {
        ensure_dictionaries();
        let text = "この問題は難しくない";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let muzukashi = results
            .iter()
            .find(|t| t.base_form == "難しい" && t.surface_form != "難しい")
            .expect("conjugated「難しい」token should exist");
        assert!(
            muzukashi.grammar_label.is_some(),
            "i-adjective negative form「難しくない」should carry a grammar_label, got: {:?}",
            muzukashi
        );
    }
    // AdjectiveToKunakatta — i-adj negative past (高い→高くなかった).
    #[test]
    fn should_label_i_adjective_kunakatta_negative_past_form() {
        ensure_dictionaries();
        let text = "あの映画は面白くなかった";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let omoshiro = results
            .iter()
            .find(|t| t.base_form == "面白い" && t.surface_form != "面白い")
            .expect("conjugated「面白い」token should exist");
        assert!(
            omoshiro.grammar_label.is_some(),
            "i-adjective negative past「面白くなかった」should carry a grammar_label, got: {:?}",
            omoshiro
        );
    }
    // AdjectiveToKereba — i-adj conditional (高い→高ければ).
    #[test]
    fn should_label_i_adjective_kereba_conditional_form() {
        ensure_dictionaries();
        let text = "安ければ買います";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);
        let yasuku = results
            .iter()
            .find(|t| t.base_form == "安い" && t.surface_form != "安い")
            .expect("conjugated「安い」token should exist");
        assert!(
            yasuku.grammar_label.is_some(),
            "i-adjective conditional「安ければ」should carry a grammar_label, got: {:?}",
            yasuku
        );
    }
}
