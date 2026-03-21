use crate::dictionary::grammar::get_rule_by_id;
use crate::domain::knowledge::KnowledgeSet;
use crate::domain::value_objects::NativeLanguage;
use crate::domain::{Card, CardType, GrammarRuleCard, VocabularyCard};
use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

const QUIZ_OPTIONS_COUNT: usize = 4;

const PROB_NORMAL_VIEW: f32 = 0.25;
const PROB_QUIZ_VIEW: f32 = 0.5;
const PROB_REVERSED_VIEW: f32 = 0.75;

const PROB_KANJI_NORMAL: f32 = 0.33;
const PROB_KANJI_QUIZ: f32 = 0.66;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuizOption {
    text: String,
    is_correct: bool,
}

impl QuizOption {
    pub fn new(text: String, is_correct: bool) -> Self {
        Self { text, is_correct }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuizCard {
    card: Card,
    options: Vec<QuizOption>,
}

impl QuizCard {
    pub fn new(card: Card, options: Vec<QuizOption>) -> Self {
        Self { card, options }
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn options(&self) -> &[QuizOption] {
        &self.options
    }

    pub fn check_answer(&self, index: usize) -> bool {
        self.options
            .get(index)
            .map(|o| o.is_correct())
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarInfo {
    title: String,
    description: String,
}

impl GrammarInfo {
    pub fn new(title: String, description: String) -> Self {
        Self { title, description }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LessonCardView {
    Normal(Card),
    Quiz(QuizCard),
    Reversed(Card),
    GrammarMutated {
        card: Card,
        grammar_info: GrammarInfo,
    },
    Writing(Card),
}

impl LessonCardView {
    pub fn card(&self) -> &Card {
        match self {
            LessonCardView::Normal(card)
            | LessonCardView::Reversed(card)
            | LessonCardView::GrammarMutated { card, .. }
            | LessonCardView::Writing(card) => card,
            LessonCardView::Quiz(quiz) => quiz.card(),
        }
    }

    pub fn grammar_info(&self) -> Option<&GrammarInfo> {
        match self {
            LessonCardView::GrammarMutated { grammar_info, .. } => Some(grammar_info),
            _ => None,
        }
    }

    pub fn generate_quiz(
        original_card: Card,
        same_type_cards: &[Card],
        lang: &NativeLanguage,
    ) -> Result<Self, crate::domain::OrigaError> {
        match &original_card {
            Card::Radical(_) => {
                return Ok(LessonCardView::Normal(original_card));
            }
            Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) => {}
        }

        let correct_answer = original_card.answer(lang)?;

        let mut distractors: Vec<String> = same_type_cards
            .iter()
            .filter_map(|c| {
                c.answer(lang)
                    .ok()
                    .filter(|a| a.text() != correct_answer.text())
            })
            .map(|a| a.text().to_string())
            .collect();

        distractors.shuffle(&mut rand::rng());
        let needed_distractors = QUIZ_OPTIONS_COUNT.saturating_sub(1);
        let selected_distractors: Vec<String> =
            distractors.into_iter().take(needed_distractors).collect();

        if selected_distractors.len() < needed_distractors {
            return Ok(LessonCardView::Normal(original_card));
        }

        let mut options: Vec<QuizOption> = selected_distractors
            .into_iter()
            .map(|text| QuizOption::new(text, false))
            .collect();

        options.push(QuizOption::new(correct_answer.text().to_string(), true));
        options.shuffle(&mut rand::rng());

        let quiz = QuizCard::new(original_card, options);
        Ok(LessonCardView::Quiz(quiz))
    }
}

fn select_applicable_grammar<R: Rng>(
    vocab: &VocabularyCard,
    known_grammars: &[GrammarRuleCard],
    rng: &mut R,
) -> Option<GrammarRuleCard> {
    let word_part = vocab.part_of_speech().ok()?;

    let mut rules: Vec<_> = known_grammars
        .iter()
        .filter(|g| g.apply_to().contains(&word_part))
        .cloned()
        .collect();

    rules.shuffle(rng);
    rules.into_iter().next()
}

pub struct LessonViewGenerator<'a> {
    knowledge_set: &'a KnowledgeSet,
}

impl<'a> LessonViewGenerator<'a> {
    pub fn new(knowledge_set: &'a KnowledgeSet) -> Self {
        Self { knowledge_set }
    }

    pub fn apply_view(
        &self,
        study_card: &crate::domain::StudyCard,
        is_new: bool,
        rng: &mut impl Rng,
    ) -> LessonCardView {
        let card = study_card.card();
        let card_type = CardType::from(card);

        let cards_by_type = self.knowledge_set.build_cards_by_type();
        let same_type_cards = cards_by_type
            .get(&card_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let known_grammars: Vec<_> = self
            .knowledge_set
            .study_cards()
            .values()
            .filter_map(|x| match x.card() {
                Card::Grammar(grammar_rule_card) => Some(grammar_rule_card.clone()),
                _ => None,
            })
            .collect();

        self.apply_view_inner(
            card,
            card_type,
            is_new,
            same_type_cards,
            &known_grammars,
            rng,
        )
    }

    fn apply_view_inner<R: Rng>(
        &self,
        card: &Card,
        card_type: CardType,
        is_new: bool,
        same_type_cards: &[Card],
        known_grammars: &[GrammarRuleCard],
        rng: &mut R,
    ) -> LessonCardView {
        match (card_type, is_new) {
            (CardType::Grammar, true)
            | (CardType::Grammar, false)
            | (CardType::Radical, true)
            | (CardType::Radical, false) => LessonCardView::Normal(card.clone()),

            (CardType::Kanji, true) | (CardType::Kanji, false) => {
                let rand_val = rng.random::<f32>();
                if rand_val < PROB_KANJI_NORMAL {
                    LessonCardView::Normal(card.clone())
                } else if rand_val < PROB_KANJI_QUIZ {
                    LessonCardView::generate_quiz(
                        card.clone(),
                        same_type_cards,
                        &NativeLanguage::Russian,
                    )
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
                } else {
                    LessonCardView::Writing(card.clone())
                }
            }

            (_, true) => {
                if rng.random_bool(0.5) {
                    LessonCardView::generate_quiz(
                        card.clone(),
                        same_type_cards,
                        &NativeLanguage::Russian,
                    )
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
                } else {
                    LessonCardView::Normal(card.clone())
                }
            }

            (CardType::Vocabulary, false) => {
                let rand_val = rng.random::<f32>();
                if rand_val < PROB_NORMAL_VIEW {
                    LessonCardView::Normal(card.clone())
                } else if rand_val < PROB_QUIZ_VIEW {
                    LessonCardView::generate_quiz(
                        card.clone(),
                        same_type_cards,
                        &NativeLanguage::Russian,
                    )
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
                } else if rand_val < PROB_REVERSED_VIEW {
                    self.apply_reversed(card)
                } else {
                    self.apply_grammar_mutated(card, known_grammars, rng)
                }
            }
        }
    }

    fn apply_reversed(&self, card: &Card) -> LessonCardView {
        match card {
            Card::Vocabulary(vocab) => match vocab.revert(&NativeLanguage::Russian) {
                Ok(reverted) => LessonCardView::Reversed(Card::Vocabulary(reverted)),
                Err(_) => LessonCardView::Normal(card.clone()),
            },
            _ => LessonCardView::Normal(card.clone()),
        }
    }

    fn apply_grammar_mutated<R: Rng>(
        &self,
        card: &Card,
        known_grammars: &[GrammarRuleCard],
        rng: &mut R,
    ) -> LessonCardView {
        match card {
            Card::Vocabulary(vocab) => {
                match select_applicable_grammar(vocab, known_grammars, rng) {
                    Some(grammar_card) => {
                        let rule = get_rule_by_id(grammar_card.rule_id());
                        match rule {
                            Some(r) => match vocab.with_grammar_rule(r, &NativeLanguage::Russian) {
                                Ok((mutated, grammar_description)) => {
                                    let grammar_title = grammar_card
                                        .title(&NativeLanguage::Russian)
                                        .map(|q| q.text().to_string())
                                        .unwrap_or_else(|_| grammar_card.rule_id().to_string());
                                    let grammar_info =
                                        GrammarInfo::new(grammar_title, grammar_description);
                                    LessonCardView::GrammarMutated {
                                        card: Card::Vocabulary(mutated),
                                        grammar_info,
                                    }
                                }
                                Err(_) => LessonCardView::Normal(card.clone()),
                            },
                            None => LessonCardView::Normal(card.clone()),
                        }
                    }
                    None => LessonCardView::Normal(card.clone()),
                }
            }
            _ => LessonCardView::Normal(card.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::knowledge::VocabularyCard;
    use crate::domain::value_objects::Question;
    use ulid::Ulid;

    fn create_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    fn create_grammar_card(rule_id: Ulid) -> Card {
        Card::Grammar(GrammarRuleCard::new(rule_id).unwrap())
    }

    #[test]
    fn generate_quiz_returns_normal_for_radical() {
        crate::use_cases::init_real_dictionaries();

        let vocab1 = create_vocab_card("単語1");
        let vocab2 = create_vocab_card("単語2");
        let vocab3 = create_vocab_card("単語3");

        let other_cards: Vec<Card> = vec![vocab1, vocab2, vocab3];
        let lang = NativeLanguage::Russian;

        // Radical карточки всегда возвращают Normal вид
        let radical_card = crate::domain::knowledge::RadicalCard::new('一').unwrap();
        let result =
            LessonCardView::generate_quiz(Card::Radical(radical_card), &other_cards, &lang);

        assert!(matches!(result, Ok(LessonCardView::Normal(_))));
    }

    #[test]
    fn grammar_info_new_creates_instance() {
        let info = GrammarInfo::new("Title".to_string(), "Description".to_string());
        assert_eq!(info.title(), "Title");
        assert_eq!(info.description(), "Description");
    }

    #[test]
    fn grammar_info_returns_correct_data() {
        let info = GrammarInfo::new(
            "て-form".to_string(),
            "Форма для соединения глаголов".to_string(),
        );
        assert_eq!(info.title(), "て-form");
        assert_eq!(info.description(), "Форма для соединения глаголов");
    }

    #[test]
    fn lesson_card_view_card_returns_inner_card() {
        let vocab = create_vocab_card("猫");

        let normal = LessonCardView::Normal(vocab.clone());
        assert_eq!(normal.card(), &vocab);

        let reversed = LessonCardView::Reversed(vocab.clone());
        assert_eq!(reversed.card(), &vocab);

        let mutated = LessonCardView::GrammarMutated {
            card: vocab.clone(),
            grammar_info: GrammarInfo::new("Test".to_string(), "Test description".to_string()),
        };
        assert_eq!(mutated.card(), &vocab);

        let quiz = LessonCardView::Quiz(QuizCard::new(vocab.clone(), vec![]));
        assert_eq!(quiz.card(), &vocab);
    }

    mod grammar_quiz {
        use super::*;
        use crate::use_cases::init_real_dictionaries;

        fn get_first_grammar_rule_id() -> Ulid {
            init_real_dictionaries();
            Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").expect("Invalid ULID")
        }

        #[test]
        fn grammar_card_generates_quiz_with_sufficient_distinct_cards() {
            init_real_dictionaries();

            let grammar_rule_id = get_first_grammar_rule_id();
            let grammar_card = create_grammar_card(grammar_rule_id);

            let rule_ids = vec![
                "01KJ9AVWBG78GHSKKD8W1YHJB3",
                "01KJ9AVWBG1AAJZXRGA499R44W",
                "01KJ9AVWBG865E0F72RYM7F34B",
            ];
            let other_cards: Vec<Card> = rule_ids
                .into_iter()
                .map(|id| create_grammar_card(Ulid::from_string(id).expect("Invalid ULID")))
                .collect();

            let lang = NativeLanguage::Russian;

            let result = LessonCardView::generate_quiz(grammar_card, &other_cards, &lang);

            assert!(result.is_ok());

            match result.unwrap() {
                LessonCardView::Quiz(quiz) => {
                    assert_eq!(quiz.options().len(), 4);
                    assert!(quiz.options().iter().any(|o| o.is_correct()));
                }
                _ => panic!("Expected Quiz view for grammar card with sufficient distractors"),
            }
        }

        #[test]
        fn grammar_card_returns_normal_with_insufficient_distinct_cards() {
            init_real_dictionaries();

            let grammar_rule_id = get_first_grammar_rule_id();
            let grammar_card = create_grammar_card(grammar_rule_id);

            let other_cards: Vec<Card> = vec![];

            let lang = NativeLanguage::Russian;

            let result = LessonCardView::generate_quiz(grammar_card.clone(), &other_cards, &lang);

            assert!(result.is_ok());

            match result.unwrap() {
                LessonCardView::Normal(card) => {
                    assert_eq!(card, grammar_card);
                }
                _ => panic!("Expected Normal view for grammar card with insufficient distractors"),
            }
        }

        #[test]
        fn grammar_quiz_options_contain_correct_answer() {
            init_real_dictionaries();

            let grammar_rule_id = get_first_grammar_rule_id();
            let grammar_card = create_grammar_card(grammar_rule_id);

            let rule_ids = vec![
                "01KJ9AVWBG78GHSKKD8W1YHJB3",
                "01KJ9AVWBG1AAJZXRGA499R44W",
                "01KJ9AVWBG865E0F72RYM7F34B",
            ];
            let other_cards: Vec<Card> = rule_ids
                .into_iter()
                .map(|id| create_grammar_card(Ulid::from_string(id).expect("Invalid ULID")))
                .collect();

            let lang = NativeLanguage::Russian;

            let result = LessonCardView::generate_quiz(grammar_card.clone(), &other_cards, &lang);

            assert!(result.is_ok());

            match result.unwrap() {
                LessonCardView::Quiz(quiz) => {
                    let correct_answer = grammar_card.answer(&lang).unwrap();
                    assert!(
                        quiz.options()
                            .iter()
                            .any(|o| o.text() == correct_answer.text()),
                        "Quiz options should contain the correct answer"
                    );
                }
                _ => panic!("Expected Quiz view"),
            }
        }
    }
}
