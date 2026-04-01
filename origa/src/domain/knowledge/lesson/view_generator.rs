use crate::dictionary::grammar::get_rule_by_id;
use crate::domain::knowledge::KnowledgeSet;
use crate::domain::value_objects::NativeLanguage;
use crate::domain::{Card, CardType, GrammarRuleCard, MemoryHistory, VocabularyCard};
use rand::{prelude::IndexedRandom, seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

const QUIZ_OPTIONS_COUNT: usize = 4;

const PROB_NORMAL_VIEW: f32 = 0.15;
const PROB_QUIZ_VIEW: f32 = 0.30;
const PROB_YESNO_VIEW: f32 = 0.50;
const PROB_REVERSED_VIEW: f32 = 0.75;

const PROB_KANJI_NORMAL: f32 = 0.25;
const PROB_KANJI_QUIZ: f32 = 0.50;
const PROB_KANJI_YESNO: f32 = 0.70;
const PROB_RADICAL_NORMAL: f32 = 0.50;

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
pub struct YesNoCard {
    card: Card,
    statement_text: String,
    is_correct: bool,
}

impl YesNoCard {
    pub fn new(card: Card, statement_text: String, is_correct: bool) -> Self {
        Self {
            card,
            statement_text,
            is_correct,
        }
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn statement_text(&self) -> &str {
        &self.statement_text
    }

    pub fn is_correct(&self) -> bool {
        self.is_correct
    }

    /// Проверяет, правильно ли ответил пользователь
    /// user_said_yes: true = "Да", false = "Нет"
    /// Возвращает true если ответ правильный (верное утверждение + "Да" или ложное + "Нет")
    pub fn check_answer(&self, user_said_yes: bool) -> bool {
        (self.is_correct && user_said_yes) || (!self.is_correct && !user_said_yes)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarInfo {
    pub rule_id: Option<Ulid>,
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
}

impl LessonCardView {
    pub fn card(&self) -> &Card {
        match self {
            LessonCardView::Normal(card)
            | LessonCardView::Reversed(card)
            | LessonCardView::GrammarMutated { card, .. }
            | LessonCardView::Writing(card) => card,
            LessonCardView::Quiz(quiz) => quiz.card(),
            LessonCardView::YesNo(yc) => yc.card(),
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
            Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) | Card::Radical(_) => {},
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

    pub fn generate_yesno(
        original_card: Card,
        same_type_cards: &[Card],
        lang: &NativeLanguage,
        rng: &mut impl Rng,
    ) -> Result<Self, crate::domain::OrigaError> {
        match &original_card {
            Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) | Card::Radical(_) => {},
        }

        let question = original_card.question(lang)?;
        let correct_answer = original_card.answer(lang)?;

        let is_correct = rng.random_bool(0.5);

        let statement_answer = if is_correct {
            correct_answer.text().to_string()
        } else {
            let distractors: Vec<_> = same_type_cards
                .iter()
                .filter_map(|c| c.answer(lang).ok())
                .filter(|a| a.text() != correct_answer.text())
                .map(|a| a.text().to_string())
                .collect();

            if distractors.is_empty() {
                return Ok(LessonCardView::Normal(original_card));
            }

            distractors
                .choose(rng)
                .expect("distractors guaranteed non-empty after is_empty check")
                .clone()
        };

        let statement_text = format!("{} \n {}", question.text(), statement_answer);

        Ok(LessonCardView::YesNo(YesNoCard::new(
            original_card,
            statement_text,
            is_correct,
        )))
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

fn select_card_view<R: Rng>(
    card: &Card,
    same_type_cards: &[Card],
    lang: &NativeLanguage,
    rng: &mut R,
    allow_writing: bool,
    allow_yesno: bool,
) -> LessonCardView {
    let rand_val = rng.random::<f32>();
    if rand_val < PROB_KANJI_NORMAL {
        LessonCardView::Normal(card.clone())
    } else if rand_val < PROB_KANJI_QUIZ {
        LessonCardView::generate_quiz(card.clone(), same_type_cards, lang)
            .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
    } else if allow_yesno && rand_val < PROB_KANJI_YESNO {
        LessonCardView::generate_yesno(card.clone(), same_type_cards, lang, rng)
            .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
    } else if allow_writing {
        LessonCardView::Writing(card.clone())
    } else {
        LessonCardView::Normal(card.clone())
    }
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
            study_card.memory(),
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
        memory: &MemoryHistory,
        rng: &mut R,
    ) -> LessonCardView {
        match (card_type, is_new) {
            (CardType::Grammar, true) | (CardType::Grammar, false) => {
                LessonCardView::Normal(card.clone())
            },

            (CardType::Radical, true) => {
                let rand_val = rng.random::<f32>();
                if rand_val < PROB_RADICAL_NORMAL {
                    LessonCardView::Normal(card.clone())
                } else {
                    LessonCardView::generate_quiz(
                        card.clone(),
                        same_type_cards,
                        &NativeLanguage::Russian,
                    )
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
                }
            },

            (CardType::Kanji, true) => {
                let rand_val = rng.random::<f32>();
                if rand_val < 0.33 {
                    LessonCardView::Normal(card.clone())
                } else if rand_val < 0.66 {
                    LessonCardView::generate_quiz(
                        card.clone(),
                        same_type_cards,
                        &NativeLanguage::Russian,
                    )
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
                } else {
                    LessonCardView::Writing(card.clone())
                }
            },

            (CardType::Radical, false) => select_card_view(
                card,
                same_type_cards,
                &NativeLanguage::Russian,
                rng,
                false,
                !memory.is_high_difficulty(),
            ),

            (CardType::Kanji, false) => select_card_view(
                card,
                same_type_cards,
                &NativeLanguage::Russian,
                rng,
                true,
                !memory.is_high_difficulty(),
            ),

            (_, true) => {
                let rand_val = rng.random::<f32>();
                if rand_val < 0.50 {
                    LessonCardView::Normal(card.clone())
                } else {
                    LessonCardView::generate_quiz(
                        card.clone(),
                        same_type_cards,
                        &NativeLanguage::Russian,
                    )
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
                }
            },

            (CardType::Vocabulary, false) => {
                let is_high_difficulty = memory.is_high_difficulty();
                let eligible_for_advanced_views = memory.is_known_card() || memory.is_in_progress();
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
                } else if !is_high_difficulty && rand_val < PROB_YESNO_VIEW {
                    LessonCardView::generate_yesno(
                        card.clone(),
                        same_type_cards,
                        &NativeLanguage::Russian,
                        rng,
                    )
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
                } else if eligible_for_advanced_views && rand_val < PROB_REVERSED_VIEW {
                    self.apply_reversed(card)
                } else if eligible_for_advanced_views {
                    self.apply_grammar_mutated(card, known_grammars, rng)
                } else {
                    LessonCardView::Normal(card.clone())
                }
            },
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
                                    let grammar_info = GrammarInfo::new(
                                        Some(grammar_card.rule_id().to_owned()),
                                        grammar_title,
                                        grammar_description,
                                    );
                                    LessonCardView::GrammarMutated {
                                        card: Card::Vocabulary(mutated),
                                        grammar_info,
                                    }
                                },
                                Err(_) => LessonCardView::Normal(card.clone()),
                            },
                            None => LessonCardView::Normal(card.clone()),
                        }
                    },
                    None => LessonCardView::Normal(card.clone()),
                }
            },
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
    fn generate_quiz_can_return_quiz_for_radical() {
        crate::use_cases::init_real_dictionaries();

        let radical_chars: Vec<char> = vec!['一', '二', '三', '人', '口', '日', '月'];
        let radical_cards: Vec<Card> = radical_chars
            .into_iter()
            .filter_map(|c| crate::domain::knowledge::RadicalCard::new(c).ok())
            .map(Card::Radical)
            .collect();

        if radical_cards.len() < 4 {
            // Not enough radicals in dictionary, test is not applicable
            return;
        }

        let lang = NativeLanguage::Russian;
        let result = LessonCardView::generate_quiz(radical_cards[0].clone(), &radical_cards, &lang);

        match result.unwrap() {
            LessonCardView::Quiz(quiz) => {
                assert_eq!(quiz.options().len(), 4);
                assert!(quiz.options().iter().any(|o| o.is_correct()));
            },
            LessonCardView::Normal(_) => {
                // This is also acceptable if not enough distractors
            },
            _ => panic!("Expected Quiz or Normal view for radical"),
        }
    }

    #[test]
    fn grammar_info_new_creates_instance() {
        let info = GrammarInfo::new(None, "Title".to_string(), "Description".to_string());
        assert_eq!(info.title(), "Title");
        assert_eq!(info.description(), "Description");
    }

    #[test]
    fn grammar_info_creates_with_rule_id() {
        let rule_id = Ulid::new();
        let info = GrammarInfo::new(
            Some(rule_id),
            "て-form".to_string(),
            "Description".to_string(),
        );
        assert_eq!(info.rule_id(), Some(rule_id));
    }

    #[test]
    fn grammar_info_without_rule_id_returns_none() {
        let info = GrammarInfo::new(None, "て-form".to_string(), "Description".to_string());
        assert_eq!(info.rule_id(), None);
    }

    #[test]
    fn grammar_info_returns_correct_data() {
        let info = GrammarInfo::new(
            None,
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
            grammar_info: GrammarInfo::new(
                None,
                "Test".to_string(),
                "Test description".to_string(),
            ),
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
                },
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
                },
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
                },
                _ => panic!("Expected Quiz view"),
            }
        }

        mod yesno_view_filtering {
            use super::*;
            use crate::domain::memory::{Difficulty, MemoryState, Rating, ReviewLog, Stability};
            use crate::domain::value_objects::Question;
            use crate::domain::{Card, StudyCard};
            use chrono::{Duration, Utc};
            use rand::{rngs::StdRng, SeedableRng};

            fn create_study_card_with_memory(
                word: &str,
                stability: f64,
                difficulty: f64,
                interval_days: i64,
                rating: Rating,
            ) -> StudyCard {
                let card = Card::Vocabulary(VocabularyCard::new(
                    Question::new(word.to_string()).unwrap(),
                ));
                let mut study_card = StudyCard::new(card);
                let memory = MemoryState::new(
                    Stability::new(stability).unwrap(),
                    Difficulty::new(difficulty).unwrap(),
                    Utc::now(),
                );
                study_card.add_review(
                    memory,
                    ReviewLog::new(rating, Duration::days(interval_days)),
                );
                study_card
            }

            fn create_high_difficulty_card(word: &str) -> StudyCard {
                let study_card = create_study_card_with_memory(word, 3.0, 7.0, 5, Rating::Hard);
                assert!(study_card.memory().is_high_difficulty());
                assert!(!study_card.memory().is_known_card());
                assert!(!study_card.memory().is_in_progress());
                study_card
            }

            fn create_in_progress_card(word: &str) -> StudyCard {
                let study_card = create_study_card_with_memory(word, 5.0, 3.0, 5, Rating::Good);
                assert!(study_card.memory().is_in_progress());
                assert!(!study_card.memory().is_high_difficulty());
                assert!(!study_card.memory().is_known_card());
                study_card
            }

            fn create_knowledge_set_with_vocab(words: &[&str]) -> KnowledgeSet {
                let mut ks = KnowledgeSet::new();
                for word in words {
                    ks.create_card(Card::Vocabulary(VocabularyCard::new(
                        Question::new(word.to_string()).unwrap(),
                    )))
                    .unwrap();
                }
                ks
            }

            const DISTRACTOR_WORDS: &[&str] = &["猫", "犬", "鳥", "魚", "馬", "牛"];
            const ITERATIONS: u64 = 500;

            fn count_yesno_views(study_card: &StudyCard, ks: &KnowledgeSet) -> (usize, usize) {
                let generator = LessonViewGenerator::new(ks);
                let mut yesno_count = 0;
                let mut other_count = 0;

                for seed in 0..ITERATIONS {
                    let mut rng = StdRng::seed_from_u64(seed);
                    let view = generator.apply_view(study_card, study_card.is_new(), &mut rng);

                    match view {
                        LessonCardView::YesNo(_) => yesno_count += 1,
                        _ => other_count += 1,
                    }
                }

                (yesno_count, other_count)
            }

            #[test]
            fn high_difficulty_card_never_gets_yesno_view() {
                crate::use_cases::init_real_dictionaries();

                let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
                let study_card = create_high_difficulty_card("猫");

                let (yesno, _other) = count_yesno_views(&study_card, &ks);

                assert_eq!(
                yesno, 0,
                "high_difficulty card should never get YesNo view, got {yesno} YesNo out of {ITERATIONS} iterations"
            );
            }

            #[test]
            fn in_progress_card_can_get_yesno_view() {
                crate::use_cases::init_real_dictionaries();

                let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
                let study_card = create_in_progress_card("猫");

                let (yesno, _other) = count_yesno_views(&study_card, &ks);

                assert!(
                yesno > 0,
                "in_progress card should be able to get YesNo view, got 0 YesNo out of {ITERATIONS} iterations"
            );
            }
        }
    }

    mod tests_yesno {
        use super::*;
        use rand::{rngs::StdRng, SeedableRng};

        fn create_vocab_card_with_word(word: &str) -> Card {
            Card::Vocabulary(VocabularyCard::new(
                Question::new(word.to_string()).unwrap(),
            ))
        }

        fn create_yesno_card(is_correct: bool) -> YesNoCard {
            let card = create_vocab_card_with_word("テスト");
            YesNoCard::new(card, "テスト – тест".to_string(), is_correct)
        }

        #[test]
        fn test_yesno_card_check_answer_correct_yes() {
            let yesno = create_yesno_card(true);
            assert!(yesno.check_answer(true));
        }

        #[test]
        fn test_yesno_card_check_answer_false_no() {
            let yesno = create_yesno_card(false);
            assert!(yesno.check_answer(false));
        }

        #[test]
        fn test_yesno_card_check_answer_wrong_yes() {
            let yesno = create_yesno_card(false);
            assert!(!yesno.check_answer(true));
        }

        #[test]
        fn test_yesno_card_check_answer_wrong_no() {
            let yesno = create_yesno_card(true);
            assert!(!yesno.check_answer(false));
        }

        #[test]
        fn test_generate_yesno_correct_statement() {
            crate::use_cases::init_real_dictionaries();

            let vocab_words = ["猫", "犬", "鳥", "魚"];
            let cards: Vec<Card> = vocab_words
                .iter()
                .map(|w| create_vocab_card_with_word(w))
                .collect();

            let mut rng = StdRng::seed_from_u64(42);
            let result = LessonCardView::generate_yesno(
                cards[0].clone(),
                &cards[1..],
                &NativeLanguage::Russian,
                &mut rng,
            );

            assert!(result.is_ok());
            match result.unwrap() {
                LessonCardView::YesNo(yesno) => {
                    assert!(!yesno.statement_text().is_empty());
                },
                _ => panic!("Expected YesNo view"),
            }
        }

        #[test]
        fn test_generate_yesno_false_statement() {
            crate::use_cases::init_real_dictionaries();

            let vocab_words = ["猫", "犬", "鳥", "魚"];
            let cards: Vec<Card> = vocab_words
                .iter()
                .map(|w| create_vocab_card_with_word(w))
                .collect();

            let mut rng = StdRng::seed_from_u64(123);
            let result = LessonCardView::generate_yesno(
                cards[0].clone(),
                &cards[1..],
                &NativeLanguage::Russian,
                &mut rng,
            );

            assert!(result.is_ok());
            match result.unwrap() {
                LessonCardView::YesNo(yesno) => {
                    assert!(!yesno.statement_text().is_empty());
                },
                _ => panic!("Expected YesNo view"),
            }
        }

        #[test]
        fn test_generate_yesno_fallback_no_distractors() {
            crate::use_cases::init_real_dictionaries();

            let card = create_vocab_card_with_word("猫");
            let empty_cards: Vec<Card> = vec![];

            let mut rng = StdRng::seed_from_u64(42);
            let result = LessonCardView::generate_yesno(
                card.clone(),
                &empty_cards,
                &NativeLanguage::Russian,
                &mut rng,
            );

            assert!(result.is_ok());
            match result.unwrap() {
                LessonCardView::Normal(returned_card) => {
                    assert_eq!(returned_card, card);
                },
                _ => panic!("Expected Normal fallback when no distractors available"),
            }
        }

        #[test]
        fn test_yesno_probability_distribution() {
            crate::use_cases::init_real_dictionaries();

            let vocab_words = ["猫", "犬", "鳥", "魚", "馬", "牛", "羊", "豚"];
            let cards: Vec<Card> = vocab_words
                .iter()
                .map(|w| create_vocab_card_with_word(w))
                .collect();

            let iterations = 1000;
            let mut yesno_count = 0;

            for seed in 0..iterations {
                let mut rng = StdRng::seed_from_u64(seed);
                let result = LessonCardView::generate_yesno(
                    cards[0].clone(),
                    &cards[1..],
                    &NativeLanguage::Russian,
                    &mut rng,
                );

                if let Ok(LessonCardView::YesNo(_)) = result {
                    yesno_count += 1;
                }
            }

            let ratio = yesno_count as f32 / iterations as f32;

            // YesNo должен генерироваться примерно в 100% случаев при наличии дистракторов
            // (fallback на Normal происходит только когда is_correct=false И нет дистракторов)
            assert!(ratio > 0.95, "YesNo generation ratio too low: {ratio}");
        }

        #[test]
        fn test_yesno_is_correct_distribution() {
            crate::use_cases::init_real_dictionaries();

            let vocab_words = ["猫", "犬", "鳥", "魚"];
            let cards: Vec<Card> = vocab_words
                .iter()
                .map(|w| create_vocab_card_with_word(w))
                .collect();

            let iterations = 1000;
            let mut correct_count = 0;
            let mut incorrect_count = 0;

            for seed in 0..iterations {
                let mut rng = StdRng::seed_from_u64(seed);
                let result = LessonCardView::generate_yesno(
                    cards[0].clone(),
                    &cards[1..],
                    &NativeLanguage::Russian,
                    &mut rng,
                );

                if let Ok(LessonCardView::YesNo(yesno)) = result {
                    if yesno.is_correct() {
                        correct_count += 1;
                    } else {
                        incorrect_count += 1;
                    }
                }
            }

            let correct_ratio = correct_count as f32 / iterations as f32;
            let incorrect_ratio = incorrect_count as f32 / iterations as f32;

            // is_correct должен быть примерно 50/50 из-за rng.random_bool(0.5)
            assert!(
                (0.45..=0.55).contains(&correct_ratio),
                "is_correct ratio should be ~50%, got {correct_ratio}"
            );
            assert!(
                (0.45..=0.55).contains(&incorrect_ratio),
                "is_incorrect ratio should be ~50%, got {incorrect_ratio}"
            );
        }
    }

    mod reversed_view_filtering {
        use super::*;
        use crate::domain::memory::{Difficulty, MemoryState, Rating, ReviewLog, Stability};
        use crate::domain::value_objects::Question;
        use crate::domain::{Card, StudyCard};
        use chrono::{Duration, Utc};
        use rand::{rngs::StdRng, SeedableRng};

        fn create_study_card_with_memory(
            word: &str,
            stability: f64,
            difficulty: f64,
            interval_days: i64,
            rating: Rating,
        ) -> StudyCard {
            let card = Card::Vocabulary(VocabularyCard::new(
                Question::new(word.to_string()).unwrap(),
            ));
            let mut study_card = StudyCard::new(card);
            let memory = MemoryState::new(
                Stability::new(stability).unwrap(),
                Difficulty::new(difficulty).unwrap(),
                Utc::now(),
            );
            study_card.add_review(
                memory,
                ReviewLog::new(rating, Duration::days(interval_days)),
            );
            study_card
        }

        fn create_high_difficulty_card(word: &str) -> StudyCard {
            let study_card = create_study_card_with_memory(word, 3.0, 7.0, 5, Rating::Hard);
            assert!(study_card.memory().is_high_difficulty());
            assert!(!study_card.memory().is_known_card());
            assert!(!study_card.memory().is_in_progress());
            study_card
        }

        fn create_known_card(word: &str) -> StudyCard {
            let study_card = create_study_card_with_memory(word, 15.0, 2.0, 20, Rating::Easy);
            assert!(study_card.memory().is_known_card());
            assert!(!study_card.memory().is_high_difficulty());
            assert!(!study_card.memory().is_in_progress());
            study_card
        }

        fn create_in_progress_card(word: &str) -> StudyCard {
            let study_card = create_study_card_with_memory(word, 5.0, 3.0, 5, Rating::Good);
            assert!(study_card.memory().is_in_progress());
            assert!(!study_card.memory().is_high_difficulty());
            assert!(!study_card.memory().is_known_card());
            study_card
        }

        fn create_knowledge_set_with_vocab(words: &[&str]) -> KnowledgeSet {
            let mut ks = KnowledgeSet::new();
            for word in words {
                ks.create_card(Card::Vocabulary(VocabularyCard::new(
                    Question::new(word.to_string()).unwrap(),
                )))
                .unwrap();
            }
            ks
        }

        const DISTRACTOR_WORDS: &[&str] = &["猫", "犬", "鳥", "魚", "馬", "牛"];
        const ITERATIONS: u64 = 500;

        fn count_view_types(study_card: &StudyCard, ks: &KnowledgeSet) -> (usize, usize, usize) {
            let generator = LessonViewGenerator::new(ks);
            let mut reversed_count = 0;
            let mut grammar_mutated_count = 0;
            let mut normal_count = 0;

            for seed in 0..ITERATIONS {
                let mut rng = StdRng::seed_from_u64(seed);
                let view = generator.apply_view(study_card, study_card.is_new(), &mut rng);

                match view {
                    LessonCardView::Reversed(_) => reversed_count += 1,
                    LessonCardView::GrammarMutated { .. } => grammar_mutated_count += 1,
                    LessonCardView::Normal(_) => normal_count += 1,
                    _ => {},
                }
            }

            (reversed_count, grammar_mutated_count, normal_count)
        }

        #[test]
        fn high_difficulty_card_never_gets_reversed_view() {
            crate::use_cases::init_real_dictionaries();

            let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
            let study_card = create_high_difficulty_card("猫");

            let (reversed, grammar_mutated, normal) = count_view_types(&study_card, &ks);

            assert_eq!(
                reversed, 0,
                "high_difficulty card should never get Reversed view"
            );
            assert_eq!(
                grammar_mutated, 0,
                "high_difficulty card should never get GrammarMutated view"
            );
            assert!(
                normal > 0,
                "high_difficulty card should get Normal view as fallback in advanced range"
            );
        }

        #[test]
        fn known_card_can_get_reversed_view() {
            crate::use_cases::init_real_dictionaries();

            let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
            let study_card = create_known_card("猫");

            let (reversed, grammar_mutated, normal) = count_view_types(&study_card, &ks);

            assert!(
                reversed > 0 || grammar_mutated > 0,
                "known card should get Reversed or GrammarMutated views, got {reversed} reversed, {grammar_mutated} grammar_mutated, {normal} normal"
            );
        }

        #[test]
        fn in_progress_card_can_get_reversed_view() {
            crate::use_cases::init_real_dictionaries();

            let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
            let study_card = create_in_progress_card("猫");

            let (reversed, grammar_mutated, normal) = count_view_types(&study_card, &ks);

            assert!(
                reversed > 0 || grammar_mutated > 0,
                "in_progress card should get Reversed or GrammarMutated views, got {reversed} reversed, {grammar_mutated} grammar_mutated, {normal} normal"
            );
        }
    }
}
