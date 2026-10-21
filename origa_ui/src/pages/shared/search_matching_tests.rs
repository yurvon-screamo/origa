//! Native tests for [`super::search_matching`] (#837): the extracted
//! default matching, the kana-normalized reading matching and the words/
//! kanji page matchers.
//!
//! The kanji matcher's card arm is covered by e2e (KanjiCard construction
//! requires the loaded kanji dictionary); the "furigana dictionary not
//! loaded" degradation branch cannot be tested directly — the fixture
//! below installs the dictionary into a process-global OnceLock before
//! any reading test runs, and OnceLock cannot be unloaded.

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use rstest::rstest;

    use crate::pages::shared::search_matching::{
        answer_matches, default_card_matches, readings_match, vocabulary_readings,
        vocabulary_search_matcher,
    };
    use leptos::prelude::{RwSignal, Set};
    use origa::dictionary::furigana_dict::{init_furigana_dict, is_furigana_dict_loaded};
    use origa::domain::{Card, CardAnswer, NativeLanguage, Question, StudyCard, VocabularyCard};

    const LANG: NativeLanguage = NativeLanguage::Russian;

    fn owner_with<T>(f: impl FnOnce() -> T) -> T {
        leptos::prelude::Owner::new().with(f)
    }

    /// A card whose Japanese lives in `reverse_side` — `answer()` works
    /// without the translation dictionary.
    fn reversed_card(word: &str, reverse_side: &str) -> StudyCard {
        let vocab = VocabularyCard::new_with_pos(
            Question::new(word.to_string()).unwrap(),
            None,
            Some(Question::new(reverse_side.to_string()).unwrap()),
        );
        StudyCard::new(Card::Vocabulary(vocab))
    }

    fn card_with_word(word: &str) -> StudyCard {
        let vocab =
            VocabularyCard::new_with_pos(Question::new(word.to_string()).unwrap(), None, None);
        StudyCard::new(Card::Vocabulary(vocab))
    }

    // ── default_card_matches ─────────────────────────────────────────

    #[test]
    fn default_card_matches_question_surface_case_insensitively() {
        let card = reversed_card("Eating", "たべる");

        assert!(default_card_matches(&card, "eat", LANG));
    }

    #[test]
    fn default_card_matches_answer_text() {
        let card = reversed_card("eating", "たべる");

        assert!(default_card_matches(&card, "たべる", LANG));
    }

    #[test]
    fn default_card_misses_when_neither_side_matches() {
        let card = reversed_card("eating", "たべる");

        assert!(!default_card_matches(&card, "いぬ", LANG));
    }

    #[test]
    fn answer_matches_vocabulary_translation() {
        let answer = CardAnswer::Vocabulary {
            translations: vec!["есть".to_string(), "кушать".to_string()],
            description: None,
        };

        assert!(answer_matches(Some(&answer), "есть"));
        assert!(!answer_matches(Some(&answer), "пить"));
    }

    #[test]
    fn answer_matches_vocabulary_description() {
        let answer = CardAnswer::Vocabulary {
            translations: vec!["есть".to_string()],
            description: Some("поглощать пищу".to_string()),
        };

        assert!(answer_matches(Some(&answer), "пищу"));
    }

    #[test]
    fn answer_matches_without_flat_text_returns_false() {
        // CommonMistake has no public constructor outside origa's own
        // tests — serde is the public way in.
        let mistake: origa::dictionary::grammar::CommonMistake =
            serde_json::from_str(r#"{"wrong":"а","correct":"б"}"#).unwrap();
        let answer = CardAnswer::grammar_nuances(vec![mistake], Vec::new()).unwrap();

        assert!(!answer_matches(Some(&answer), "а"));
        assert!(!answer_matches(None, "а"));
    }

    // ── readings_match ───────────────────────────────────────────────

    #[rstest]
    #[case::on_reading_hiragana_query(&["ショク"], "しょく", true)]
    #[case::on_reading_katakana_query(&["ショク"], "ショク", true)]
    #[case::kun_reading_okurigana_dot(&["た.べる"], "たべる", true)]
    #[case::kun_reading_partial(&["きび.しい"], "きび", true)]
    #[case::kun_reading_compound_dash(&["ひと-り"], "ひとり", true)]
    #[case::unrelated_reading(&["ショク"], "ねこ", false)]
    fn readings_match_normalizes_script_and_separators(
        #[case] readings: &[&str],
        #[case] query: &str,
        #[case] expected: bool,
    ) {
        let readings: Vec<String> = readings.iter().map(|s| s.to_string()).collect();

        assert_eq!(readings_match(&readings, query), expected);
    }

    // ── vocabulary readings (furigana dictionary fixture) ────────────

    /// One shared fixture for every reading test: the dictionary slot is
    /// a process-global OnceLock, so the first install wins — all test
    /// words must live in this single fixture (a foreign install makes
    /// the reading tests fail loudly, which is the intended loud failure).
    const FIXTURE: &str =
        "食べる|たべる|0:た\n大人|おとな|0-1:おとな\n大人|だいにん|0-1:だいにん\n";

    fn ensure_furigana_fixture() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            if !is_furigana_dict_loaded() {
                init_furigana_dict(FIXTURE).expect("furigana fixture installs");
            }
        });
    }

    #[rstest]
    #[case::hiragana_query("たべる", "食べる", true)]
    #[case::katakana_query("タベル", "食べる", true)]
    #[case::partial_query("たべ", "食べる", true)]
    #[case::second_reading("だいにん", "大人", true)]
    #[case::unrelated_query("ねこ", "食べる", false)]
    fn vocabulary_matcher_finds_word_by_reading(
        #[case] query: &str,
        #[case] word: &str,
        #[case] expected: bool,
    ) {
        ensure_furigana_fixture();

        owner_with(|| {
            let card = card_with_word(word);
            let matcher = vocabulary_search_matcher(RwSignal::new(true));

            assert_eq!(matcher(&card, query, LANG), expected);
        })
    }

    #[test]
    fn vocabulary_matcher_misses_readings_until_the_dictionary_is_ready() {
        ensure_furigana_fixture();

        owner_with(|| {
            let card = card_with_word("食べる");
            let ready = RwSignal::new(false);
            let matcher = vocabulary_search_matcher(ready);

            assert!(!matcher(&card, "たべる", LANG), "not ready → no match");

            ready.set(true);
            assert!(matcher(&card, "たべる", LANG), "ready → reading matches");
        })
    }

    #[test]
    fn vocabulary_matcher_finds_katakana_word_by_hiragana_query() {
        // Dict-independent: the surface arm normalizes the kana script.
        owner_with(|| {
            let card = card_with_word("コンピューター");
            let matcher = vocabulary_search_matcher(RwSignal::new(false));

            assert!(matcher(&card, "こんぴゅーたー", LANG));
        })
    }

    #[test]
    fn vocabulary_readings_return_every_dictionary_reading() {
        ensure_furigana_fixture();

        let mut readings = vocabulary_readings("大人");
        readings.sort();

        assert_eq!(readings, vec!["おとな".to_string(), "だいにん".to_string()]);
    }

    #[test]
    fn vocabulary_readings_second_call_returns_the_cached_result() {
        ensure_furigana_fixture();

        let first = vocabulary_readings("食べる");
        let second = vocabulary_readings("食べる");

        assert_eq!(first, second);
        assert!(first.contains(&"たべる".to_string()));
    }
}
