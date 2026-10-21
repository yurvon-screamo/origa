use origa::domain::MultiQuizResult;

use super::yesno_card_view::YesNoResult;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum QuizResult {
    #[default]
    None,
    Correct,
    Incorrect,
    DontKnow,
    MultiCorrect,
    /// Lenient pass (#822): ≥50% of the correct readings selected, none
    /// wrong — counts as a pass (the rating pipeline is untouched), but
    /// some readings were missed, so the message must NOT claim "all
    /// readings selected". Shows the missed list in success color.
    MultiLenientPass,
    MultiPartial,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OptionDisplay {
    Neutral,
    Correct,
    Wrong,
    Missed,
    Dimmed,
    /// Completely hidden — used to remove incorrect/unselected options
    /// after the user answers, collapsing the answer grid to only the
    /// selected answer and the correct one.
    Hidden,
}

impl QuizResult {
    pub fn option_display(&self, is_correct: bool, is_selected: bool) -> OptionDisplay {
        match self {
            QuizResult::None => OptionDisplay::Neutral,
            QuizResult::Correct | QuizResult::Incorrect if is_correct => OptionDisplay::Correct,
            QuizResult::Correct => OptionDisplay::Hidden,
            QuizResult::Incorrect if is_selected => OptionDisplay::Wrong,
            QuizResult::Incorrect => OptionDisplay::Hidden,
            QuizResult::DontKnow if is_correct => OptionDisplay::Correct,
            QuizResult::DontKnow => OptionDisplay::Hidden,
            QuizResult::MultiCorrect | QuizResult::MultiLenientPass | QuizResult::MultiPartial => {
                OptionDisplay::Neutral
            },
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
            OptionDisplay::Hidden => "quiz-option-hidden",
        }
    }

    /// Strict multi grading: anything less than every correct option is a
    /// partial fail.
    pub fn from_multi_result(result: &MultiQuizResult) -> Self {
        if result.is_perfect {
            QuizResult::MultiCorrect
        } else {
            QuizResult::MultiPartial
        }
    }

    /// Lenient multi grading (kanji reading quiz): the RATING side counts
    /// ≥50% correct selections without wrong ones as a pass — this mapping
    /// mirrors that for the MESSAGE only. Perfect stays MultiCorrect («все
    /// чтения выбраны»); a pass with missed readings becomes
    /// MultiLenientPass so the UI can show WHAT was missed while keeping
    /// the success color (#822).
    pub fn from_multi_result_lenient(result: &MultiQuizResult) -> Self {
        if result.is_perfect {
            QuizResult::MultiCorrect
        } else if result.is_lenient_pass() {
            QuizResult::MultiLenientPass
        } else {
            QuizResult::MultiPartial
        }
    }
}

impl From<YesNoResult> for QuizResult {
    fn from(value: YesNoResult) -> Self {
        match value {
            YesNoResult::None => QuizResult::None,
            YesNoResult::Correct => QuizResult::Correct,
            YesNoResult::Incorrect => QuizResult::Incorrect,
            YesNoResult::DontKnow => QuizResult::DontKnow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn multi_result(missed: Vec<usize>, wrong: Vec<usize>) -> MultiQuizResult {
        let correct_selections = vec![0usize];
        let is_perfect = missed.is_empty() && wrong.is_empty();
        MultiQuizResult {
            correct_selections,
            missed,
            wrong_selections: wrong,
            is_perfect,
        }
    }

    #[rstest]
    #[case::perfect_maps_to_multi_correct(multi_result(vec![], vec![]), QuizResult::MultiCorrect)]
    #[case::lenient_pass_with_missed_maps_to_lenient(
        multi_result(vec![2], vec![]),
        QuizResult::MultiLenientPass
    )]
    #[case::below_lenient_threshold_maps_to_partial(
        multi_result(vec![1, 2, 3], vec![]),
        QuizResult::MultiPartial
    )]
    #[case::wrong_selection_never_passes_lenient(
        multi_result(vec![], vec![1]),
        QuizResult::MultiPartial
    )]
    fn lenient_mapping_keeps_perfect_splits_pass_and_fail(
        #[case] result: MultiQuizResult,
        #[case] expected: QuizResult,
    ) {
        assert_eq!(QuizResult::from_multi_result_lenient(&result), expected);
    }

    #[test]
    fn single_quiz_hides_incorrect_unselected_options_after_answer() {
        // User answered incorrectly. Options that are neither correct nor
        // selected by the user should be Hidden — not Dimmed — so the grid
        // collapses to just the user's answer + the correct one.
        let result = QuizResult::Incorrect;
        assert_eq!(
            result.option_display(false, false),
            OptionDisplay::Hidden,
            "incorrect unselected options should be Hidden after answering"
        );
    }

    #[test]
    fn single_quiz_keeps_selected_wrong_visible() {
        let result = QuizResult::Incorrect;
        assert_eq!(
            result.option_display(false, true),
            OptionDisplay::Wrong,
            "the user's selected wrong answer must stay visible"
        );
    }

    #[test]
    fn single_quiz_correct_answer_keeps_selected_visible() {
        let result = QuizResult::Correct;
        assert_eq!(result.option_display(true, true), OptionDisplay::Correct);
    }

    #[test]
    fn single_quiz_correct_answer_hides_unselected() {
        let result = QuizResult::Correct;
        assert_eq!(
            result.option_display(false, false),
            OptionDisplay::Hidden,
            "when answered correctly, unselected options should collapse"
        );
    }

    #[test]
    fn dont_know_hides_all_wrong_keeps_correct() {
        let result = QuizResult::DontKnow;
        assert_eq!(result.option_display(true, false), OptionDisplay::Correct);
        assert_eq!(result.option_display(false, false), OptionDisplay::Hidden);
    }
}
