//! Postfix-tail distractors (#503): another full form of the same word
//! carries the rule's identical visible tail (書いたつもりです,
//! 書きますつもりです for the ～つもりです quiz). Often the result is a valid
//! construction of a *different* rule — which tests the meaning contrast
//! the rule body teaches.

use crate::dictionary::grammar::FormatAction;
use crate::domain::PartOfSpeech;
use crate::domain::apply_format_actions;

/// Full forms of the source word used to break postfix-tail chains,
/// ordered by learner-error plausibility. The dictionary form comes
/// first via the bare tail chain in [`composed_tail_candidates`].
const VERB_FULL_FORM_POOL: &[FormatAction] = &[
    FormatAction::VerbToTeForm {},
    FormatAction::VerbToTa {},
    FormatAction::VerbToMasu {},
    FormatAction::VerbToNai {},
    FormatAction::VerbToStem {},
    FormatAction::VerbToMizenkei {},
];

const I_ADJECTIVE_FULL_FORM_POOL: &[FormatAction] = &[
    FormatAction::AdjectiveToKunai {},
    FormatAction::AdjectiveToKatta {},
    FormatAction::AdjectiveToKute {},
];

const NA_ADJECTIVE_FULL_FORM_POOL: &[FormatAction] = &[
    FormatAction::AdjectiveToNa {},
    FormatAction::AdjectiveToDe {},
    FormatAction::AdjectiveToNara {},
];

/// `[wrong full form] + tail` composed through the existing format
/// machinery, so only applicable Japanese is produced.
pub(super) fn composed_tail_candidates(
    base: &[FormatAction],
    tail: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
) -> Vec<String> {
    let pool: &[FormatAction] = match pos {
        PartOfSpeech::Verb => VERB_FULL_FORM_POOL,
        PartOfSpeech::IAdjective => I_ADJECTIVE_FULL_FORM_POOL,
        PartOfSpeech::NaAdjective => NA_ADJECTIVE_FULL_FORM_POOL,
        _ => return Vec::new(),
    };

    // Dictionary form + tail: the classic "attached to the plain form"
    // mistake (書くつもりです is the correct answer of a different rule;
    // 書くください is simply wrong). Duplicates of the correct answer are
    // removed by the caller's dedup.
    let mut chains: Vec<Vec<FormatAction>> = vec![tail.to_vec()];
    for action in pool {
        if base.len() == 1 && std::mem::discriminant(&base[0]) == std::mem::discriminant(action) {
            continue; // the rule's own base form would repeat the answer
        }
        let mut chain = Vec::with_capacity(tail.len() + 1);
        chain.push(action.clone());
        chain.extend(tail.iter().cloned());
        chains.push(chain);
    }

    chains
        .iter()
        .filter_map(|chain| apply_format_actions(source_word, chain, pos).ok())
        .collect()
}
