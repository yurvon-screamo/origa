use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::Card;

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

    pub fn check_answer(&self, user_said_yes: bool) -> bool {
        (self.is_correct && user_said_yes) || (!self.is_correct && !user_said_yes)
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
        }
    }

    pub fn grammar_info(&self) -> Option<&GrammarInfo> {
        match self {
            LessonCardView::GrammarMutated { grammar_info, .. } => Some(grammar_info),
            LessonCardView::Normal(_)
            | LessonCardView::Quiz(_)
            | LessonCardView::YesNo(_)
            | LessonCardView::Reversed(_)
            | LessonCardView::Writing(_)
            | LessonCardView::PhraseListen { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LessonCard {
    view: LessonCardView,
    is_short_term: bool,
}

impl LessonCard {
    pub fn new(view: LessonCardView, is_short_term: bool) -> Self {
        Self {
            view,
            is_short_term,
        }
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
}
