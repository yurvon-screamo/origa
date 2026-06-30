use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::Card;
use crate::domain::knowledge::card::CardType;
use crate::domain::memory::Rating;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuizOption {
    text: String,
    is_correct: bool,
    #[serde(default)]
    tag: Option<String>,
}

impl QuizOption {
    pub fn new(text: String, is_correct: bool, tag: Option<String>) -> Self {
        Self {
            text,
            is_correct,
            tag,
        }
    }

    pub fn new_simple(text: String, is_correct: bool) -> Self {
        Self {
            text,
            is_correct,
            tag: None,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuizMode {
    #[default]
    Single,
    Multi,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuizCard {
    card: Card,
    options: Vec<QuizOption>,
    #[serde(default)]
    mode: QuizMode,
}

impl QuizCard {
    pub fn new(card: Card, options: Vec<QuizOption>, mode: QuizMode) -> Self {
        Self {
            card,
            options,
            mode,
        }
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn options(&self) -> &[QuizOption] {
        &self.options
    }

    pub fn mode(&self) -> QuizMode {
        self.mode
    }

    pub fn check_answer(&self, index: usize) -> bool {
        self.options
            .get(index)
            .map(|o| o.is_correct())
            .unwrap_or(false)
    }

    pub fn check_multi_answers(&self, selected: &[usize]) -> MultiQuizResult {
        let correct_indices: Vec<usize> = self
            .options
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_correct())
            .map(|(i, _)| i)
            .collect();

        let selected_set: std::collections::HashSet<usize> = selected.iter().copied().collect();
        let correct_set: std::collections::HashSet<usize> =
            correct_indices.iter().copied().collect();

        let correct_selections: Vec<usize> = selected
            .iter()
            .filter(|&&i| correct_set.contains(&i))
            .copied()
            .collect();

        let missed: Vec<usize> = correct_indices
            .iter()
            .filter(|&&i| !selected_set.contains(&i))
            .copied()
            .collect();

        let wrong_selections: Vec<usize> = selected
            .iter()
            .filter(|&&i| !correct_set.contains(&i))
            .copied()
            .collect();

        let is_perfect = missed.is_empty() && wrong_selections.is_empty();

        MultiQuizResult {
            correct_selections,
            missed,
            wrong_selections,
            is_perfect,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultiQuizResult {
    pub correct_selections: Vec<usize>,
    pub missed: Vec<usize>,
    pub wrong_selections: Vec<usize>,
    pub is_perfect: bool,
}

impl MultiQuizResult {
    pub fn rating(&self) -> Rating {
        if self.is_perfect {
            Rating::Good
        } else {
            Rating::Again
        }
    }

    pub fn is_lenient_pass(&self) -> bool {
        self.wrong_selections.is_empty()
            && self.correct_selections.len() * 2
                >= (self.correct_selections.len() + self.missed.len())
    }

    pub fn rating_lenient(&self) -> Rating {
        if self.is_lenient_pass() {
            Rating::Good
        } else {
            Rating::Again
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct YesNoCard {
    card: Card,
    word: String,
    statement: String,
    is_correct: bool,
}

impl YesNoCard {
    pub fn new(card: Card, word: String, statement: String, is_correct: bool) -> Self {
        Self {
            card,
            word,
            statement,
            is_correct,
        }
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn statement(&self) -> &str {
        &self.statement
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }

    pub fn check_answer(&self, user_said_yes: bool) -> bool {
        (self.is_correct && user_said_yes) || (!self.is_correct && !user_said_yes)
    }
}

impl<'de> Deserialize<'de> for YesNoCard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Manual impl (rather than derive) to backfill the legacy
        // single-field `statement_text` shape; see `split_legacy_statement_text`.
        #[derive(Deserialize)]
        struct Wire {
            card: Card,
            is_correct: bool,
            #[serde(default)]
            word: Option<String>,
            #[serde(default)]
            statement: Option<String>,
            #[serde(default)]
            statement_text: Option<String>,
        }

        let wire = Wire::deserialize(deserializer)?;

        let (word, statement) = match (wire.word, wire.statement) {
            (Some(word), Some(statement)) => (word, statement),
            (Some(word), None) => (word, String::new()),
            (None, Some(statement)) => (String::new(), statement),
            (None, None) => wire
                .statement_text
                .as_deref()
                .map(split_legacy_statement_text)
                .unwrap_or_default(),
        };

        Ok(YesNoCard {
            card: wire.card,
            word,
            statement,
            is_correct: wire.is_correct,
        })
    }
}

/// Recovers `word` and `statement` from a legacy `statement_text` joined
/// by `" \n "` (the format emitted by `generate_yesno`). When the
/// separator is absent the whole string becomes `statement`.
fn split_legacy_statement_text(joined: &str) -> (String, String) {
    const SEP: &str = " \n ";
    match joined.split_once(SEP) {
        Some((word, statement)) => (word.trim().to_string(), statement.trim().to_string()),
        None => (String::new(), joined.to_string()),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarInfo {
    rule_id: Option<Ulid>,
    title: String,
    description: String,
}

impl GrammarInfo {
    pub fn new(rule_id: Option<Ulid>, title: String, description: String) -> Self {
        Self {
            rule_id,
            title,
            description,
        }
    }

    pub fn rule_id(&self) -> Option<Ulid> {
        self.rule_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarQuizCard {
    card: Card,
    grammar_info: GrammarInfo,
    word_text: String,
    quiz: QuizCard,
}

impl GrammarQuizCard {
    pub fn new(card: Card, grammar_info: GrammarInfo, word_text: String, quiz: QuizCard) -> Self {
        Self {
            card,
            grammar_info,
            word_text,
            quiz,
        }
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn grammar_info(&self) -> &GrammarInfo {
        &self.grammar_info
    }

    pub fn word_text(&self) -> &str {
        &self.word_text
    }

    pub fn quiz(&self) -> &QuizCard {
        &self.quiz
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LessonCardView {
    Normal(Card),
    Quiz(QuizCard),
    YesNo(YesNoCard),
    Reversed(Card),
    GrammarMutated {
        card: Card,
        grammar_info: GrammarInfo,
    },
    Writing(Card),
    PhraseListen {
        card: Card,
        audio_file: String,
        options: Vec<QuizOption>,
    },
    KanjiReadingQuiz(QuizCard),
    GrammarQuiz(GrammarQuizCard),
}

impl LessonCardView {
    pub fn card(&self) -> &Card {
        match self {
            LessonCardView::Normal(card)
            | LessonCardView::Reversed(card)
            | LessonCardView::GrammarMutated { card, .. }
            | LessonCardView::Writing(card)
            | LessonCardView::PhraseListen { card, .. } => card,
            LessonCardView::Quiz(quiz) => quiz.card(),
            LessonCardView::YesNo(yc) => yc.card(),
            LessonCardView::KanjiReadingQuiz(quiz) => quiz.card(),
            LessonCardView::GrammarQuiz(gq) => gq.card(),
        }
    }

    pub fn grammar_info(&self) -> Option<&GrammarInfo> {
        match self {
            LessonCardView::GrammarMutated { grammar_info, .. } => Some(grammar_info),
            LessonCardView::GrammarQuiz(gq) => Some(gq.grammar_info()),
            LessonCardView::Normal(_)
            | LessonCardView::Quiz(_)
            | LessonCardView::YesNo(_)
            | LessonCardView::Reversed(_)
            | LessonCardView::Writing(_)
            | LessonCardView::PhraseListen { .. }
            | LessonCardView::KanjiReadingQuiz(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LessonCard {
    #[serde(default = "Ulid::nil")]
    card_id: Ulid,
    view: LessonCardView,
    #[serde(default)]
    is_short_term: bool,
}

impl LessonCard {
    pub fn new(card_id: Ulid, view: LessonCardView, is_short_term: bool) -> Self {
        Self {
            card_id,
            view,
            is_short_term,
        }
    }

    pub fn card_id(&self) -> Ulid {
        self.card_id
    }

    pub fn view(&self) -> &LessonCardView {
        &self.view
    }

    pub fn into_view(self) -> LessonCardView {
        self.view
    }

    pub fn is_short_term(&self) -> bool {
        self.is_short_term
    }

    pub fn card(&self) -> &Card {
        self.view.card()
    }

    pub fn grammar_info(&self) -> Option<&GrammarInfo> {
        self.view.grammar_info()
    }

    fn card_type(&self) -> CardType {
        CardType::from(self.view.card())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LessonData {
    pub cards: Vec<(Ulid, LessonCard)>,
    pub core_count: usize,
}

impl LessonData {
    pub fn card_ids(&self) -> Vec<Ulid> {
        self.cards.iter().map(|(id, _)| *id).collect()
    }

    pub fn cards_map(&self) -> HashMap<Ulid, LessonCard> {
        self.cards
            .iter()
            .map(|(id, card)| (*id, card.clone()))
            .collect()
    }

    pub fn total_count(&self) -> usize {
        self.cards.len()
    }

    pub fn phrase_count(&self) -> usize {
        self.cards
            .iter()
            .filter(|(_, lc)| lc.card_type() == CardType::Phrase)
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn contains_key(&self, id: &Ulid) -> bool {
        self.cards.iter().any(|(card_id, _)| card_id == id)
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn get(&self, id: &Ulid) -> Option<&LessonCard> {
        self.cards
            .iter()
            .find(|(card_id, _)| card_id == id)
            .map(|(_, card)| card)
    }

    pub fn keys(&self) -> impl Iterator<Item = &Ulid> {
        self.cards.iter().map(|(id, _)| id)
    }

    pub fn values(&self) -> impl Iterator<Item = &LessonCard> {
        self.cards.iter().map(|(_, card)| card)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Ulid, &LessonCard)> {
        self.cards.iter().map(|(id, card)| (id, card))
    }

    pub fn into_cards(self) -> Vec<(Ulid, LessonCard)> {
        self.cards
    }
}

impl<'de> Deserialize<'de> for LessonData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Wire mirror of `LessonData` used for the `card_id` backfill
        // (legacy lessons serialised before multi-show expansion stored
        // the card id only in the slot key). When a field is added to
        // `LessonData`, it MUST be mirrored here, otherwise it will be
        // silently dropped on deserialisation. The
        // `lesson_data_roundtrip_preserves_all_fields` test guards this
        // contract automatically — keep it in sync with new fields.
        #[derive(Deserialize)]
        struct Wire {
            #[serde(default)]
            cards: Vec<(Ulid, LessonCard)>,
            #[serde(default)]
            core_count: usize,
        }

        let mut wire = Wire::deserialize(deserializer)?;
        for (slot_id, lc) in &mut wire.cards {
            if lc.card_id.is_nil() {
                lc.card_id = *slot_id;
            }
        }
        Ok(LessonData {
            cards: wire.cards,
            core_count: wire.core_count,
        })
    }
}

impl IntoIterator for LessonData {
    type Item = (Ulid, LessonCard);
    type IntoIter = std::vec::IntoIter<(Ulid, LessonCard)>;

    fn into_iter(self) -> Self::IntoIter {
        self.cards.into_iter()
    }
}

#[cfg(test)]
impl LessonData {
    /// Returns every lesson card whose underlying `card_id` (the
    /// multi-show identity) matches. Only used in tests to count how
    /// many showings one logical card produced. Declared `pub(crate)`
    /// and gated behind `#[cfg(test)]` so it never ships as part of the
    /// public API: production code addresses cards by slot id (`get`)
    /// and never by the multi-show `card_id`.
    pub(crate) fn find_by_card_id(&self, card_id: Ulid) -> Vec<&LessonCard> {
        self.cards
            .iter()
            .filter(|(_, lc)| lc.card_id() == card_id)
            .map(|(_, lc)| lc)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Card;
    use crate::domain::knowledge::{PhraseCard, VocabularyCard};
    use crate::domain::value_objects::Question;

    fn make_vocabulary_lesson_card(id: Ulid) -> (Ulid, LessonCard) {
        let card = Card::Vocabulary(VocabularyCard::new(
            Question::new("test".to_string()).unwrap(),
        ));
        (id, LessonCard::new(id, LessonCardView::Normal(card), false))
    }

    fn make_phrase_lesson_card(id: Ulid) -> (Ulid, LessonCard) {
        let card = Card::Phrase(PhraseCard::new_test_with_id(id));
        (id, LessonCard::new(id, LessonCardView::Normal(card), false))
    }

    #[test]
    fn lesson_data_constructor_preserves_all_cards() {
        let vocab_1 = Ulid::new();
        let vocab_2 = Ulid::new();
        let phrase_1 = Ulid::new();
        let phrase_2 = Ulid::new();

        let cards = vec![
            make_phrase_lesson_card(phrase_1),
            make_vocabulary_lesson_card(vocab_1),
            make_phrase_lesson_card(phrase_2),
            make_vocabulary_lesson_card(vocab_2),
        ];
        let core_count = cards.len();
        let data = LessonData { cards, core_count };

        assert_eq!(data.total_count(), 4);
        assert_eq!(data.core_count, 4);
        assert_eq!(data.phrase_count(), 2);

        assert!(data.contains_key(&vocab_1));
        assert!(data.contains_key(&vocab_2));
        assert!(data.contains_key(&phrase_1));
        assert!(data.contains_key(&phrase_2));
    }

    #[test]
    fn lesson_data_constructor_handles_empty() {
        let data = LessonData {
            cards: vec![],
            core_count: 0,
        };

        assert!(data.is_empty());
        assert_eq!(data.core_count, 0);
        assert_eq!(data.total_count(), 0);
    }

    #[test]
    fn lesson_data_core_count_independent_of_phrase_position() {
        // After interleaving, phrases live inside the core section; core_count
        // must reflect the interleaved layout (phrases counted as core) while
        // phrase_count still identifies them by type.
        let vocab_1 = Ulid::new();
        let phrase_1 = Ulid::new();
        let vocab_2 = Ulid::new();

        let cards = vec![
            make_vocabulary_lesson_card(vocab_1),
            make_phrase_lesson_card(phrase_1),
            make_vocabulary_lesson_card(vocab_2),
        ];
        let data = LessonData {
            cards,
            core_count: 3,
        };

        assert_eq!(data.core_count, 3);
        assert_eq!(data.phrase_count(), 1);
        assert_eq!(
            data.phrase_count(),
            data.cards
                .iter()
                .filter(|(_, lc)| lc.card_type() == CardType::Phrase)
                .count()
        );
    }

    /// Roundtrip guard for the manual `Deserialize` impl: every field of
    /// `LessonData` must survive a serialise → deserialise cycle. The
    /// impl uses a local `Wire` mirror struct (see `impl Deserialize for
    /// LessonData`); forgetting to mirror a new field there would let
    /// the field vanish silently on load. This test fails the moment
    /// such a regression is introduced.
    #[test]
    fn lesson_data_roundtrip_preserves_all_fields() {
        let card_id = Ulid::new();
        let slot_id = Ulid::new();
        let view = LessonCardView::Normal(Card::Vocabulary(VocabularyCard::new(
            Question::new("猫".to_string()).expect("valid question"),
        )));
        let original = LessonData {
            cards: vec![(slot_id, LessonCard::new(card_id, view.clone(), true))],
            core_count: 1,
        };

        let json = serde_json::to_string(&original).expect("serialize LessonData");
        let restored: LessonData = serde_json::from_str(&json).expect("deserialize LessonData");

        assert_eq!(restored.core_count, original.core_count);
        assert_eq!(restored.cards.len(), original.cards.len());
        let (restored_slot, restored_lc) = &restored.cards[0];
        assert_eq!(
            *restored_slot, slot_id,
            "slot id (tuple key) must roundtrip"
        );
        assert_eq!(
            restored_lc.card_id(),
            card_id,
            "explicit card_id must roundtrip (not be overwritten by the slot-id backfill)"
        );
        assert!(
            restored_lc.is_short_term(),
            "is_short_term flag must roundtrip"
        );
        assert_eq!(
            restored_lc.view(),
            &view,
            "LessonCardView payload must roundtrip"
        );
    }

    /// Guards the legacy backfill branch of the manual `Deserialize`
    /// impl: when a `LessonCard` was serialised without an explicit
    /// `card_id` (nil ULID), the deserialiser must repopulate it from
    /// the slot id (the tuple key). This is the only non-trivial
    /// transformation in the impl and the roundtrip test above does
    /// not exercise it (it uses an explicit non-nil card_id).
    #[test]
    fn lesson_data_deserialize_backfills_nil_card_id_from_slot() {
        let slot_id = Ulid::new();
        let view = LessonCardView::Normal(Card::Vocabulary(VocabularyCard::new(
            Question::new("猫".to_string()).expect("valid question"),
        )));
        let view_json = serde_json::to_string(&view).expect("serialize view");
        // Simulates a legacy payload: the inner card object omits
        // `card_id`, so `#[serde(default = "Ulid::nil")]` yields the
        // nil ULID and triggers the backfill branch.
        let legacy_json = format!(
            r#"{{"cards":[["{slot_id}",{{"view":{view_json},"is_short_term":false}}]],"core_count":1}}"#,
        );

        let restored: LessonData =
            serde_json::from_str(&legacy_json).expect("deserialize legacy LessonData");

        let (restored_slot, restored_lc) = &restored.cards[0];
        assert_eq!(*restored_slot, slot_id, "slot id must roundtrip");
        assert_eq!(
            restored_lc.card_id(),
            slot_id,
            "nil card_id must be backfilled from the slot id"
        );
    }

    mod yesno_card_deserialize_tests {
        use super::*;

        fn sample_card() -> Card {
            Card::Vocabulary(VocabularyCard::new(
                Question::new("温度".to_string()).expect("valid question"),
            ))
        }

        fn sample_card_json() -> String {
            serde_json::to_string(&sample_card()).expect("serialize card")
        }

        #[test]
        fn yesno_card_roundtrip_preserves_fields() {
            let original = YesNoCard::new(
                sample_card(),
                "温度".to_string(),
                "спортивное состязание, матч, игра".to_string(),
                true,
            );

            let json = serde_json::to_string(&original).expect("serialize YesNoCard");
            let restored: YesNoCard = serde_json::from_str(&json).expect("deserialize YesNoCard");

            assert_eq!(restored.card(), original.card());
            assert_eq!(restored.word(), "温度");
            assert_eq!(restored.statement(), "спортивное состязание, матч, игра");
            assert!(restored.is_correct());
        }

        #[test]
        fn yesno_card_serialized_new_shape_omits_statement_text() {
            let card = YesNoCard::new(
                sample_card(),
                "温度".to_string(),
                "перевод".to_string(),
                false,
            );

            let json = serde_json::to_string(&card).expect("serialize YesNoCard");
            assert!(
                !json.contains("statement_text"),
                "new shape must not emit legacy `statement_text`: {json}"
            );
            assert!(json.contains("\"word\""));
            assert!(json.contains("\"statement\""));
        }

        #[test]
        fn yesno_card_deserialize_backfills_from_legacy_statement_text() {
            let json = format!(
                r#"{{"card":{},"statement_text":"温度 \n спортивное состязание","is_correct":true}}"#,
                sample_card_json()
            );

            let restored: YesNoCard =
                serde_json::from_str(&json).expect("deserialize legacy shape");

            assert_eq!(restored.word(), "温度");
            assert_eq!(restored.statement(), "спортивное состязание");
            assert!(restored.is_correct());
        }

        #[test]
        fn yesno_card_deserialize_legacy_without_separator() {
            let json = format!(
                r#"{{"card":{},"statement_text":"no separator here","is_correct":false}}"#,
                sample_card_json()
            );

            let restored: YesNoCard =
                serde_json::from_str(&json).expect("deserialize legacy shape");

            assert_eq!(restored.word(), "");
            assert_eq!(restored.statement(), "no separator here");
            assert!(!restored.is_correct());
        }

        #[test]
        fn yesno_card_deserialize_partial_new_shape_word_only() {
            let json = format!(
                r#"{{"card":{},"word":"温度","is_correct":true}}"#,
                sample_card_json()
            );

            let restored: YesNoCard =
                serde_json::from_str(&json).expect("deserialize partial new shape");

            assert_eq!(restored.word(), "温度");
            assert_eq!(restored.statement(), "");
            assert!(restored.is_correct());
        }

        #[test]
        fn yesno_card_deserialize_partial_new_shape_statement_only() {
            let json = format!(
                r#"{{"card":{},"statement":"only statement","is_correct":true}}"#,
                sample_card_json()
            );

            let restored: YesNoCard =
                serde_json::from_str(&json).expect("deserialize partial new shape");

            assert_eq!(restored.word(), "");
            assert_eq!(restored.statement(), "only statement");
            assert!(restored.is_correct());
        }

        #[test]
        fn yesno_card_deserialize_missing_card_errors() {
            let json = r#"{"word":"温度","statement":"x","is_correct":true}"#;

            let result: Result<YesNoCard, _> = serde_json::from_str(json);
            assert!(
                result.is_err(),
                "missing `card` must be a hard error, not a silent default"
            );
        }

        #[test]
        fn yesno_card_deserialize_missing_is_correct_errors() {
            let json = format!(
                r#"{{"card":{},"word":"温度","statement":"x"}}"#,
                sample_card_json()
            );

            let result: Result<YesNoCard, _> = serde_json::from_str(&json);
            assert!(
                result.is_err(),
                "missing `is_correct` must be a hard error: the answer-key must not silently default"
            );
        }

        #[test]
        fn yesno_card_deserialize_prefers_new_fields_over_legacy_statement_text() {
            let json = format!(
                r#"{{"card":{},"word":"新","statement":"new-stmt","statement_text":"legacy \n old","is_correct":true}}"#,
                sample_card_json()
            );

            let restored: YesNoCard = serde_json::from_str(&json).expect("deserialize mixed shape");

            assert_eq!(restored.word(), "新");
            assert_eq!(restored.statement(), "new-stmt");
        }

        /// Legacy statement_text without the `" \n "` separator (na-adjective
        /// edge): the whole string lands in `statement`, `word` stays empty.
        /// Pins the data-shape contract for the degraded legacy case.
        #[test]
        fn yesno_card_deserialize_legacy_na_adjective_edge() {
            let json = format!(
                r#"{{"card":{},"statement_text":"静か","is_correct":true}}"#,
                sample_card_json()
            );

            let restored: YesNoCard =
                serde_json::from_str(&json).expect("deserialize legacy shape");

            assert_eq!(restored.word(), "");
            assert_eq!(restored.statement(), "静か");
        }

        /// Minimal shape with only `card` + `is_correct` (no word/statement/
        /// statement_text): both text fields default to empty rather than
        /// erroring, since they are recoverable display-only data.
        #[test]
        fn yesno_card_deserialize_only_card_and_is_correct_yields_empty_strings() {
            let json = format!(r#"{{"card":{},"is_correct":true}}"#, sample_card_json());

            let restored: YesNoCard =
                serde_json::from_str(&json).expect("deserialize minimal shape");

            assert_eq!(restored.word(), "");
            assert_eq!(restored.statement(), "");
            assert!(restored.is_correct());
        }
    }
}
