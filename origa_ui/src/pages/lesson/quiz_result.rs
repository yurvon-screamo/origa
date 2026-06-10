use origa::domain::MultiQuizResult;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum QuizResult {
    #[default]
    None,
    Correct,
    Incorrect,
    DontKnow,
    MultiCorrect,
    MultiPartial,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OptionDisplay {
    Neutral,
    Correct,
    Wrong,
    Missed,
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
            QuizResult::MultiCorrect | QuizResult::MultiPartial => OptionDisplay::Neutral,
        }
    }

    pub fn multi_option_display(
        is_correct: bool,
        is_selected: bool,
        multi_result: &MultiQuizResult,
        index: usize,
    ) -> OptionDisplay {
        if !is_correct && !is_selected {
            return OptionDisplay::Dimmed;
        }
        if is_correct && is_selected {
            return OptionDisplay::Correct;
        }
        if is_correct && !is_selected {
            let is_missed = multi_result.missed.contains(&index);
            return if is_missed {
                OptionDisplay::Missed
            } else {
                OptionDisplay::Dimmed
            };
        }
        OptionDisplay::Wrong
    }

    pub fn option_class(&self, is_correct: bool, is_selected: bool) -> &'static str {
        match self.option_display(is_correct, is_selected) {
            OptionDisplay::Neutral => "quiz-option-neutral",
            OptionDisplay::Correct => "quiz-option-correct",
            OptionDisplay::Wrong => "quiz-option-wrong",
            OptionDisplay::Dimmed => "quiz-option-dimmed",
            OptionDisplay::Missed => "quiz-option-missed",
        }
    }

    pub fn from_multi_result(result: &MultiQuizResult) -> Self {
        if result.is_perfect {
            QuizResult::MultiCorrect
        } else {
            QuizResult::MultiPartial
        }
    }

    pub fn from_multi_result_lenient(result: &MultiQuizResult) -> Self {
        if result.is_lenient_pass() {
            QuizResult::MultiCorrect
        } else {
            QuizResult::MultiPartial
        }
    }
}
