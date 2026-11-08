//! Detection of the そうだ grammar pattern.
//!
//! そう has two readings depending on the predecessor token: 様態
//! (appearance, after a stem) and 伝聞 (hearsay, after a plain/past form).
//! The corpus carried three variant rules until the N4 triple collapsed
//! onto the single combined rule (its format_map generates the sou forms
//! per word class); detection keeps only the false-positive guards.
//! Split out of the former translation.rs god-file (issue #796).

use crate::dictionary::grammar::{GRAMMAR_RULES, GrammarRule};
use crate::domain::JapaneseChar;
use crate::domain::NativeLanguage;
use crate::domain::tokenizer::{PartOfSpeech, TokenInfo};

use super::grammar_match::GrammarMatch;

/// Context-aware detection for the そうだ grammar pattern.
///
/// そう attaches to a predicate: a verb/i-adjective (stem or plain form), a
/// na-adjective/noun stem, or the た/だ auxiliaries (降ったそうだ, 元気だそうだ).
///
/// A standalone そう that is NOT part of a そうだ construction (the demonstrative
/// manner adverb そう in そう思う, そうですね) must be left to `resolve_grammar_match`
/// so the manner rule `こう・そう・ああ・どう` can match — this function returns
/// `None` whenever the preceding token is not a predicate the pattern could
/// attach to (particle, adverb, pronoun, ...) or when a sentence boundary
/// separates そう from any predecessor.
pub(super) fn resolve_sou_da_match(
    token: &TokenInfo,
    index: usize,
    tokens: &[TokenInfo],
    native_language: &NativeLanguage,
) -> Option<GrammarMatch> {
    let surface = token.orthographic_surface_form();

    // Lindera splits the grammar pattern into a standalone そう token followed
    // by a copula (だ/です). Compound words like そういう / そうした are distinct
    // lemmas and must not trigger this path.
    if surface != "そう" {
        return None;
    }

    // Exclude content lemmas whose base form carries kanji (e.g. 添う "to comply",
    // surfaced as そう) — those are dictionary verbs, not the grammar pattern.
    // The grammar-pattern そう surfaces base そう with no kanji.
    let pos = token.part_of_speech();
    if pos.is_vocabulary_word() && token.orthographic_base_form().chars().any(|c| c.is_kanji()) {
        return None;
    }

    if index == 0 {
        return None;
    }

    let prev = previous_predicate(tokens, index)?;
    let prev_pos = prev.part_of_speech();
    let prev_surface = prev.orthographic_surface_form();

    // た (past) and だ (copula) auxiliaries both attach そう as hearsay:
    // 降ったそうだ (past), 元気だそうだ / 学生だそうだ (copula plain form).
    // Other auxiliaries (passive/causative chains, ない) do not.
    let attaches_to_sou_da = matches!(
        prev_pos,
        PartOfSpeech::Verb
            | PartOfSpeech::IAdjective
            | PartOfSpeech::NaAdjective
            | PartOfSpeech::Noun
            | PartOfSpeech::ProperNoun
    ) || (matches!(prev_pos, PartOfSpeech::AuxiliaryVerb)
        && (prev_surface == "た" || prev_surface == "だ"));
    if !attaches_to_sou_da {
        return None;
    }

    let rules = GRAMMAR_RULES.get()?;
    find_sou_da_rule(rules, native_language).map(|r| GrammarMatch::from_rule(r, native_language))
}

/// Locates the nearest content token preceding `index`, stopping at a sentence
/// boundary. Returns `None` when a sentence-ending punctuation (。！？) is hit
/// before any content token, so そう starting a new clause (食べた。そうだ。) is
/// not misread as hearsay about the previous sentence.
fn previous_predicate(tokens: &[TokenInfo], index: usize) -> Option<&TokenInfo> {
    for t in tokens[..index].iter().rev() {
        if is_sentence_boundary(t) {
            return None;
        }
        if matches!(
            t.part_of_speech(),
            PartOfSpeech::Whitespace | PartOfSpeech::Symbol | PartOfSpeech::AuxiliarySymbol
        ) {
            continue;
        }
        return Some(t);
    }
    None
}

fn is_sentence_boundary(token: &TokenInfo) -> bool {
    matches!(
        token.part_of_speech(),
        PartOfSpeech::Symbol | PartOfSpeech::AuxiliarySymbol
    ) && token
        .orthographic_surface_form()
        .chars()
        .any(|c| matches!(c, '。' | '！' | '？'))
}

/// Resolves the ～そうだ rule. Anchored to the rule_id (stable primary key).
/// The corpus carries a single ～そうだ rule since the N4 様態/伝聞 sibling
/// merge; the anchor must not collide with unrelated hearsay rules (～という,
/// ～とか). Falls back to any そうだ-pattern rule for stores predating the
/// merge.
fn find_sou_da_rule(
    rules: &'static [GrammarRule],
    native_language: &NativeLanguage,
) -> Option<&'static GrammarRule> {
    use ulid::Ulid;

    // Corpus anchor (cdn/grammar/grammar_v3.json): ～そうだ（様態・伝聞）.
    const SOU_DA: &str = "01G000000000000000W0000000";

    let anchor_id = Ulid::from_string(SOU_DA).ok();

    rules
        .iter()
        .find(|rule| Some(*rule.rule_id()) == anchor_id)
        // Fallback for stores predating the merge: any そうだ rule.
        .or_else(|| {
            rules
                .iter()
                .find(|rule| rule.content(native_language).pattern().contains("そうだ"))
        })
}

#[cfg(test)]
mod integration_tests {
    use super::super::lookup_tokens_translations;
    use super::super::test_env::ensure_dictionaries;
    use crate::domain::NativeLanguage;
    use crate::domain::tokenizer::tokenize_text;

    // そうだ after an i-adjective stem (美味し, base 美味しい) is 様態
    // (appearance): "looks delicious". Lindera surfaces the stem with
    // surface != base, which resolve_sou_da_label detects.
    #[test]
    fn should_label_sou_as_appearance_after_adjective_stem() {
        ensure_dictionaries();
        let text = "この料理は美味しそうだ";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ"),
            "そう after adjective stem should be 様態 (appearance), got: {:?}",
            sou
        );
    }
    // そうだ after a verb dictionary form (来る, surface == base) is 伝聞
    // (hearsay): "I heard he will come".
    #[test]
    fn should_label_sou_as_hearsay_after_dictionary_form() {
        ensure_dictionaries();
        let text = "田中さんは来るそうだ";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ"),
            "そう after dictionary form should be 伝聞 (hearsay), got: {:?}",
            sou
        );
    }
    // そうだ after a past auxiliary (降った → 降っ + た) is 伝聞 (hearsay):
    // "I heard it rained". The token directly before そう is the た auxiliary.
    #[test]
    fn should_label_sou_as_hearsay_after_past_form() {
        ensure_dictionaries();
        let text = "雨が降ったそうだ";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ"),
            "そう after past auxiliary should be 伝聞 (hearsay), got: {:?}",
            sou
        );
    }
    // Compound lemma そういう ("such") shares the そう prefix but is a distinct
    // word. The context-aware path must not attach a そうだ grammar_label to it.
    #[test]
    fn should_not_label_souiu_compound_as_sou_da_grammar() {
        ensure_dictionaries();
        let text = "そういうこと";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let souiu = results
            .iter()
            .find(|t| t.surface_form == "そういう")
            .expect("「そういう」token should exist");
        let mislabeled = souiu
            .grammar_label
            .as_deref()
            .is_some_and(|label| label.contains("そうだ"));
        assert!(
            !mislabeled,
            "「そういう」must not carry a そうだ grammar_label, got: {:?}",
            souiu.grammar_label
        );
    }
    // そうだ after a copula だ following a noun/na-adj (元気だそうだ) is 伝聞:
    // the token directly before そう is the だ copula. Locks in the central
    // adaptation — noun/na-adj hearsay is detected via the intervening copula.
    #[test]
    fn should_label_sou_as_hearsay_after_copula_da() {
        ensure_dictionaries();
        let text = "元気だそうだ";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ"),
            "そう after copula だ should be 伝聞 (hearsay), got: {:?}",
            sou
        );
    }
    // そうだ after an i-adjective past (美味しかった → 美味しかっ + た) is 伝聞.
    #[test]
    fn should_label_sou_as_hearsay_after_iadjective_past() {
        ensure_dictionaries();
        let text = "美味しかったそうだ";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ"),
            "そう after i-adjective past should be 伝聞 (hearsay), got: {:?}",
            sou
        );
    }
    // A noun directly before そう (元気そうだ) is token-ambiguous (the 様態 stem
    // looks identical to the dictionary form), so it resolves to the single
    // ～そうだ rule.
    #[test]
    fn should_label_sou_as_combined_after_bare_noun() {
        ensure_dictionaries();
        let text = "元気そうだ";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert_eq!(
            sou.grammar_label.as_deref(),
            Some("～そうだ"),
            "そう after a bare noun should fall back to the combined rule, got: {:?}",
            sou
        );
    }
    // Demonstrative/manner そう after a particle (私はそう思う) is NOT a そうだ
    // construction — it must defer to resolve_grammar_label so the manner rule
    // こう・そう・ああ・どう matches. Guards against the Combined-fallback false
    // positive where そう follows a non-predicate token.
    #[test]
    fn should_label_demonstrative_sou_as_manner_not_sou_da() {
        ensure_dictionaries();
        let text = "私はそう思う";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert!(
            sou.grammar_label
                .as_deref()
                .is_some_and(|label| label.contains("そう") && !label.contains("そうだ")),
            "demonstrative そう should match the manner rule, not a そうだ rule, got: {:?}",
            sou.grammar_label
        );
    }
    // A new sentence after 。 breaks the predecessor chain — そう in the second
    // clause (食べた。そうだ。) is agreement, not hearsay about the first clause.
    #[test]
    fn should_not_attach_sou_da_label_across_sentence_boundary() {
        ensure_dictionaries();
        let text = "食べた。そうだ。";
        let tokens = tokenize_text(text).unwrap();
        let results = lookup_tokens_translations(&tokens, &NativeLanguage::Russian, text);

        let sou = results
            .iter()
            .find(|t| t.surface_form == "そう")
            .expect("「そう」token should exist");
        assert!(
            sou.grammar_label
                .as_deref()
                .is_none_or(|label| !label.contains("そうだ")),
            "そう after a sentence boundary must not be labeled ～そうだ, got: {:?}",
            sou.grammar_label
        );
    }
}
