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

impl QuizResult {
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
}
