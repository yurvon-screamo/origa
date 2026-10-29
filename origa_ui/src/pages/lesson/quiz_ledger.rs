use origa::domain::{MultiQuizResult, QuizOption};
use std::collections::HashSet;

use super::quiz_result::QuizResult;

/// Feedback triple of a submitted multi-quiz: verdict, optional «K/N»
/// counter and the ledger rows.
pub(crate) type MultiFeedback = (LedgerVerdict, Option<(usize, usize)>, Vec<LedgerRow>);

/// One ruled row of the quiz answer ledger (task 3SekkMiYYW): the
/// structured replacement for the old «verdict + missed + extra» text
/// flow. Pure data — the [`super::quiz_feedback_strip::QuizFeedbackStrip`]
/// component renders it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LedgerRow {
    pub(crate) text: String,
    /// Reading-type tag (ON/KUN) shown next to the text, if any.
    pub(crate) tag: Option<String>,
    pub(crate) kind: LedgerRowKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LedgerRowKind {
    /// Selected and correct — «Верно», success accent.
    SelectedCorrect,
    /// Correct but not selected — «Пропущено», dashed muted row.
    Missed,
    /// Selected but wrong — «Лишнее», error accent + strikethrough.
    ExtraSelected,
    /// Single-choice: the learner's pick — «Ваш ответ» (the verdict
    /// line carries the success/error color of the outcome).
    YourAnswerWrong,
    YourAnswerCorrect,
    /// The right answer the learner missed — «Правильный ответ».
    CorrectAnswer,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LedgerVerdict {
    Correct,
    /// Lenient pass (#822): counted as a pass, but misses are shown.
    LenientPass,
    Errors,
    Incorrect,
    NotAnswered,
}

/// Verdict of a submitted single-choice quiz (correct / wrong / «не знаю»).
pub(crate) fn single_verdict(result: QuizResult) -> Option<LedgerVerdict> {
    match result {
        QuizResult::Correct => Some(LedgerVerdict::Correct),
        QuizResult::Incorrect => Some(LedgerVerdict::Incorrect),
        QuizResult::DontKnow => Some(LedgerVerdict::NotAnswered),
        _ => None,
    }
}

/// Verdict of a submitted multi-choice quiz from its resolved
/// [`QuizResult`] (already folded through the lenient grading flag).
pub(crate) fn multi_verdict(result: QuizResult) -> Option<LedgerVerdict> {
    match result {
        QuizResult::MultiCorrect => Some(LedgerVerdict::Correct),
        QuizResult::MultiLenientPass => Some(LedgerVerdict::LenientPass),
        QuizResult::MultiPartial => Some(LedgerVerdict::Errors),
        QuizResult::DontKnow => Some(LedgerVerdict::NotAnswered),
        _ => None,
    }
}

fn option_row(option: &QuizOption, kind: LedgerRowKind) -> LedgerRow {
    LedgerRow {
        text: option.text().to_string(),
        tag: option.tag().map(|t| t.to_string()),
        kind,
    }
}

/// Rows of a single-choice quiz result: the learner's pick (verdict
/// carries the mark) plus the right answer on a miss. Order per spec:
/// the learner's actions first, then the correct answer.
pub(crate) fn single_ledger_rows(
    result: QuizResult,
    options: &[QuizOption],
    selected: Option<usize>,
) -> Vec<LedgerRow> {
    let picked = selected.and_then(|i| options.get(i));
    let correct = options.iter().find(|o| o.is_correct());
    match result {
        QuizResult::Correct => picked
            .map(|o| vec![option_row(o, LedgerRowKind::YourAnswerCorrect)])
            .unwrap_or_default(),
        QuizResult::Incorrect => {
            let mut rows = Vec::new();
            if let Some(o) = picked {
                rows.push(option_row(o, LedgerRowKind::YourAnswerWrong));
            }
            if let Some(o) = correct {
                if picked != Some(o) {
                    rows.push(option_row(o, LedgerRowKind::CorrectAnswer));
                }
            }
            rows
        },
        QuizResult::DontKnow => correct
            .map(|o| vec![option_row(o, LedgerRowKind::CorrectAnswer)])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// Rows of a multi-choice quiz result: selected readings, then missed
/// ones (dashed), then extras (struck). Matching the L1 reference:
/// selected → missed → extras.
pub(crate) fn multi_ledger_rows(
    multi: &MultiQuizResult,
    options: &[QuizOption],
    selected: &HashSet<usize>,
) -> Vec<LedgerRow> {
    let mut rows = Vec::new();
    for (index, option) in options.iter().enumerate() {
        if option.is_correct() && selected.contains(&index) {
            rows.push(option_row(option, LedgerRowKind::SelectedCorrect));
        }
    }
    for index in &multi.missed {
        if let Some(option) = options.get(*index) {
            rows.push(option_row(option, LedgerRowKind::Missed));
        }
    }
    for index in &multi.wrong_selections {
        if let Some(option) = options.get(*index) {
            rows.push(option_row(option, LedgerRowKind::ExtraSelected));
        }
    }
    rows
}

/// Rows of a multi-quiz «не знаю» (submitted without picks): every
/// correct option is revealed as the answer. Today this state shows no
/// feedback at all — the ledger closes that hole.
pub(crate) fn multi_dont_know_rows(options: &[QuizOption]) -> Vec<LedgerRow> {
    options
        .iter()
        .filter(|o| o.is_correct())
        .map(|o| option_row(o, LedgerRowKind::CorrectAnswer))
        .collect()
}

/// Rows of a yes/no quiz result. The localized Да/Нет labels come from
/// the caller (`quiz_ledger_labels::yesno_label`).
pub(crate) fn yesno_ledger_rows(
    result: QuizResult,
    picked: Option<bool>,
    correct: bool,
    yes_label: &str,
    no_label: &str,
) -> Vec<LedgerRow> {
    let label = |v: bool| if v { yes_label } else { no_label }.to_string();
    match result {
        QuizResult::Correct => picked
            .map(|v| {
                vec![LedgerRow {
                    text: label(v),
                    tag: None,
                    kind: LedgerRowKind::YourAnswerCorrect,
                }]
            })
            .unwrap_or_default(),
        QuizResult::Incorrect => {
            let mut rows = vec![LedgerRow {
                text: label(picked.unwrap_or(!correct)),
                tag: None,
                kind: LedgerRowKind::YourAnswerWrong,
            }];
            rows.push(LedgerRow {
                text: label(correct),
                tag: None,
                kind: LedgerRowKind::CorrectAnswer,
            });
            rows
        },
        QuizResult::DontKnow => vec![LedgerRow {
            text: label(correct),
            tag: None,
            kind: LedgerRowKind::CorrectAnswer,
        }],
        _ => Vec::new(),
    }
}

/// Selected-correct count over the total correct count — the «K/N»
/// verdict counter of multi quizzes.
pub(crate) fn multi_score(multi: &MultiQuizResult, options: &[QuizOption]) -> (usize, usize) {
    let total = options.iter().filter(|o| o.is_correct()).count();
    (multi.correct_selections.len(), total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> Vec<QuizOption> {
        vec![
            QuizOption::new_simple("シツ".to_string(), true),
            QuizOption::new_simple("むろ".to_string(), true),
            QuizOption::new_simple("ギュウ".to_string(), false),
        ]
    }

    #[test]
    fn single_incorrect_rows_are_pick_then_correct_answer() {
        let rows = single_ledger_rows(QuizResult::Incorrect, &opts(), Some(2));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].kind, LedgerRowKind::YourAnswerWrong);
        assert_eq!(rows[0].text, "ギュウ");
        assert_eq!(rows[1].kind, LedgerRowKind::CorrectAnswer);
        assert_eq!(rows[1].text, "シツ");
    }

    #[test]
    fn single_correct_rows_show_only_the_pick() {
        let rows = single_ledger_rows(QuizResult::Correct, &opts(), Some(0));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, LedgerRowKind::YourAnswerCorrect);
    }

    #[test]
    fn single_dont_know_reveals_the_correct_answer() {
        let rows = single_ledger_rows(QuizResult::DontKnow, &opts(), None);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, LedgerRowKind::CorrectAnswer);
    }

    #[test]
    fn multi_rows_order_selected_then_missed_then_extra() {
        let mut selected = HashSet::new();
        selected.insert(0);
        selected.insert(2);
        let multi = MultiQuizResult {
            correct_selections: vec![0],
            missed: vec![1],
            wrong_selections: vec![2],
            is_perfect: false,
        };
        let rows = multi_ledger_rows(&multi, &opts(), &selected);
        assert_eq!(
            rows.iter().map(|r| r.kind).collect::<Vec<_>>(),
            vec![
                LedgerRowKind::SelectedCorrect,
                LedgerRowKind::Missed,
                LedgerRowKind::ExtraSelected,
            ]
        );
        assert_eq!(rows[2].text, "ギュウ");
    }

    #[test]
    fn multi_dont_know_reveals_every_correct_option() {
        let rows = multi_dont_know_rows(&opts());
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.kind == LedgerRowKind::CorrectAnswer));
    }

    #[test]
    fn yesno_incorrect_shows_pick_then_truth() {
        let rows = yesno_ledger_rows(QuizResult::Incorrect, Some(true), false, "Да", "Нет");
        assert_eq!(rows[0].kind, LedgerRowKind::YourAnswerWrong);
        assert_eq!(rows[0].text, "Да");
        assert_eq!(rows[1].kind, LedgerRowKind::CorrectAnswer);
        assert_eq!(rows[1].text, "Нет");
    }

    #[test]
    fn verdicts_map_quiz_results() {
        assert_eq!(
            single_verdict(QuizResult::Correct),
            Some(LedgerVerdict::Correct)
        );
        assert_eq!(
            single_verdict(QuizResult::Incorrect),
            Some(LedgerVerdict::Incorrect)
        );
        assert_eq!(
            single_verdict(QuizResult::DontKnow),
            Some(LedgerVerdict::NotAnswered)
        );
        assert_eq!(
            multi_verdict(QuizResult::MultiCorrect),
            Some(LedgerVerdict::Correct)
        );
        assert_eq!(
            multi_verdict(QuizResult::MultiLenientPass),
            Some(LedgerVerdict::LenientPass)
        );
        assert_eq!(
            multi_verdict(QuizResult::MultiPartial),
            Some(LedgerVerdict::Errors)
        );
        assert_eq!(single_verdict(QuizResult::None), None);
    }

    #[test]
    fn multi_score_counts_selected_correct_over_total() {
        let multi = MultiQuizResult {
            correct_selections: vec![0],
            missed: vec![1],
            wrong_selections: vec![2],
            is_perfect: false,
        };
        assert_eq!(multi_score(&multi, &opts()), (1, 2));
    }
}
