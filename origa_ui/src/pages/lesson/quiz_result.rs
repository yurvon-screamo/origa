#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum QuizResult {
    #[default]
    None,
    Correct,
    Incorrect,
    DontKnow,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OptionDisplay {
    Neutral,
    Correct,
    Wrong,
    Dimmed,
}

impl QuizResult {
    pub fn option_display(&self, is_correct: bool, is_selected: bool) -> OptionDisplay {
        match self {
            QuizResult::None => OptionDisplay::Neutral,
            QuizResult::Correct | QuizResult::Incorrect if is_correct => OptionDisplay::Correct,
            QuizResult::Correct => OptionDisplay::Dimmed,
            QuizResult::Incorrect if is_selected => OptionDisplay::Wrong,
            QuizResult::Incorrect => OptionDisplay::Dimmed,
            QuizResult::DontKnow if is_correct => OptionDisplay::Correct,
            QuizResult::DontKnow => OptionDisplay::Dimmed,
        }
    }

    pub fn option_class(&self, is_correct: bool, is_selected: bool) -> &'static str {
        match self.option_display(is_correct, is_selected) {
            OptionDisplay::Neutral => "quiz-option-neutral",
            OptionDisplay::Correct => "quiz-option-correct",
            OptionDisplay::Wrong => "quiz-option-wrong",
            OptionDisplay::Dimmed => "quiz-option-dimmed",
        }
    }
}
