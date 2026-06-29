//! Pure derivation of the data-load overlay message from granular `AuthStore`
//! loading flags. Split into two pure functions so both the phase classification
//! and the localized-string interpolation are unit-testable without DOM or i18n.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadingFlags {
    pub vocabulary: bool,
    pub kanji: bool,
    pub grammar: bool,
    pub radicals: bool,
    pub phrases: bool,
    pub pitch_audio: bool,
    pub dictionary: bool,
    pub furigana: bool,
    pub jlpt_content: bool,
}

// 8 parallel base resources + 1 JLPT finalization signal.
pub const LOADING_RESOURCES_TOTAL: u8 = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingMessageState {
    /// Parallel fetch of dictionaries and study materials in progress.
    Fetching { completed: u8, total: u8 },
    /// All base resources loaded; JLPT content and post-load migrations in progress.
    Finalizing { completed: u8, total: u8 },
    /// Every granular flag is set. The overlay is not rendered in this state
    /// (its guard is `!is_all_data_loaded`); kept for match-exhaustiveness.
    Complete,
}

/// Maps the current flag snapshot to a loading phase.
///
/// `load_jlpt_content` runs structurally after the 8 parallel loaders, so
/// "all 8 base flags set with JLPT still pending" honestly denotes the
/// finalization phase rather than a mid-fetch state.
pub fn loading_message_state(flags: &LoadingFlags) -> LoadingMessageState {
    let completed = [
        flags.vocabulary,
        flags.kanji,
        flags.grammar,
        flags.radicals,
        flags.phrases,
        flags.pitch_audio,
        flags.dictionary,
        flags.furigana,
        flags.jlpt_content,
    ]
    .iter()
    .filter(|loaded| **loaded)
    .count() as u8;

    let all_base_done = flags.vocabulary
        && flags.kanji
        && flags.grammar
        && flags.radicals
        && flags.phrases
        && flags.pitch_audio
        && flags.dictionary
        && flags.furigana;

    match (completed, all_base_done, flags.jlpt_content) {
        (LOADING_RESOURCES_TOTAL, _, _) => LoadingMessageState::Complete,
        (_, true, false) => LoadingMessageState::Finalizing {
            completed,
            total: LOADING_RESOURCES_TOTAL,
        },
        _ => LoadingMessageState::Fetching {
            completed,
            total: LOADING_RESOURCES_TOTAL,
        },
    }
}

/// Fills a localized template (`"… {} … {}"`) using the project convention
/// (first `{}` = completed, second `{}` = total). Selects the template by phase.
pub fn format_loading_message(
    state: LoadingMessageState,
    fetching_template: &str,
    finalizing_template: &str,
) -> String {
    match state {
        LoadingMessageState::Fetching { completed, total } => {
            fill_template(fetching_template, completed, total)
        },
        LoadingMessageState::Finalizing { completed, total } => {
            fill_template(finalizing_template, completed, total)
        },
        LoadingMessageState::Complete => fill_template(
            finalizing_template,
            LOADING_RESOURCES_TOTAL,
            LOADING_RESOURCES_TOTAL,
        ),
    }
}

fn fill_template(template: &str, completed: u8, total: u8) -> String {
    template
        .replacen("{}", &completed.to_string(), 1)
        .replacen("{}", &total.to_string(), 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn base() -> LoadingFlags {
        LoadingFlags {
            vocabulary: false,
            kanji: false,
            grammar: false,
            radicals: false,
            phrases: false,
            pitch_audio: false,
            dictionary: false,
            furigana: false,
            jlpt_content: false,
        }
    }

    fn all_base_loaded() -> LoadingFlags {
        LoadingFlags {
            vocabulary: true,
            kanji: true,
            grammar: true,
            radicals: true,
            phrases: true,
            pitch_audio: true,
            dictionary: true,
            furigana: true,
            jlpt_content: false,
        }
    }

    #[rstest]
    #[case::start(base(), LoadingMessageState::Fetching { completed: 0, total: 9 })]
    #[case::one_base(
        LoadingFlags { kanji: true, ..base() },
        LoadingMessageState::Fetching { completed: 1, total: 9 }
    )]
    #[case::seven_base(
        LoadingFlags { kanji: false, ..all_base_loaded() },
        LoadingMessageState::Fetching { completed: 7, total: 9 }
    )]
    #[case::all_base_finalizing(
        all_base_loaded(),
        LoadingMessageState::Finalizing { completed: 8, total: 9 }
    )]
    #[case::all_complete(
        LoadingFlags { jlpt_content: true, ..all_base_loaded() },
        LoadingMessageState::Complete
    )]
    #[case::robustness_jlpt_before_base(
        LoadingFlags { kanji: false, jlpt_content: true, ..all_base_loaded() },
        LoadingMessageState::Fetching { completed: 8, total: 9 }
    )]
    fn classifies_loading_state(
        #[case] flags: LoadingFlags,
        #[case] expected: LoadingMessageState,
    ) {
        assert_eq!(loading_message_state(&flags), expected);
    }

    #[rstest]
    #[case::fetching_progress(
        LoadingMessageState::Fetching { completed: 3, total: 9 },
        "F 3-9"
    )]
    #[case::fetching_zero(
        LoadingMessageState::Fetching { completed: 0, total: 9 },
        "F 0-9"
    )]
    #[case::finalizing(
        LoadingMessageState::Finalizing { completed: 8, total: 9 },
        "FIN 8 of 9"
    )]
    #[case::complete_falls_back_to_finalizing_deterministic(
        LoadingMessageState::Complete,
        "FIN 9 of 9"
    )]
    fn formats_message(#[case] state: LoadingMessageState, #[case] expected: &str) {
        assert_eq!(
            format_loading_message(state, "F {}-{}", "FIN {} of {}"),
            expected
        );
    }

    #[test]
    fn missing_placeholders_do_not_panic() {
        let fetching = format_loading_message(
            LoadingMessageState::Fetching {
                completed: 3,
                total: 9,
            },
            "Loading {}",
            "Finalizing {} of {}",
        );
        assert_eq!(fetching, "Loading 3");

        let finalizing = format_loading_message(
            LoadingMessageState::Finalizing {
                completed: 8,
                total: 9,
            },
            "Loading {}",
            "Done",
        );
        assert_eq!(finalizing, "Done");
    }
}
