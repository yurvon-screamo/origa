use super::*;

fn candidates(word: &str, correct: &str, pos: PartOfSpeech) -> Vec<String> {
    near_miss_candidates(word, correct, &pos)
}

#[test]
fn godan_masu_breaks_stem_keeps_ending() {
    let result = candidates("書く", "書きます", PartOfSpeech::Verb);
    assert_eq!(
        result,
        vec![
            "書くます".to_string(),
            "書かます".to_string(),
            "書っます".to_string(),
            "書けます".to_string(),
            "書こます".to_string(),
        ]
    );
}

#[test]
fn godan_te_form_breaks_stem_keeps_ending() {
    let result = candidates("書く", "書いて", PartOfSpeech::Verb);
    assert_eq!(
        result,
        vec![
            "書くて".to_string(),
            "書かて".to_string(),
            "書って".to_string(),
            "書いで".to_string(),
            "書けて".to_string(),
            "書こて".to_string(),
            "書きて".to_string(),
        ]
    );
}

#[test]
fn godan_nai_leads_with_forgot_to_conjugate() {
    let result = candidates("書く", "書かない", PartOfSpeech::Verb);
    // The a-row candidate collides with the correct form (ない builds on
    // mizenkei) and is deduplicated away.
    assert_eq!(
        result,
        vec![
            "書くない".to_string(),
            "書っない".to_string(),
            "書けない".to_string(),
            "書こない".to_string(),
            "書きない".to_string(),
        ]
    );
}

#[test]
fn ichidan_masu_includes_godanization_error() {
    let result = candidates("食べる", "食べます", PartOfSpeech::Verb);
    assert_eq!(
        result,
        vec![
            "食べるます".to_string(),
            "食べらます".to_string(),
            "食っます".to_string(),
            "食べれます".to_string(),
            "食べろます".to_string(),
            "食べります".to_string(),
        ]
    );
}

#[test]
fn imperative_empty_ending_yields_wrong_row_stems() {
    let result = candidates("書く", "書け", PartOfSpeech::Verb);
    assert_eq!(
        result,
        vec![
            "書く".to_string(),
            "書か".to_string(),
            "書っ".to_string(),
            "書こ".to_string(),
            "書き".to_string(),
        ]
    );
}

#[test]
fn honorific_wrap_keeps_prefix_on_every_candidate() {
    let result = candidates("書く", "お書きになります", PartOfSpeech::Verb);
    assert_eq!(
        result,
        vec![
            "お書くになります".to_string(),
            "お書かになります".to_string(),
            "お書っになります".to_string(),
            "お書けになります".to_string(),
            "お書こになります".to_string(),
        ]
    );
    assert!(!result.contains(&"お書きになります".to_string()));
}

#[test]
fn suru_verb_decomposes_like_godan() {
    let result = candidates("する", "します", PartOfSpeech::Verb);
    // Rows swap the す (not the final る); the i-row candidate collides
    // with します and the tsu operator declines to empty the stem.
    assert_eq!(
        result,
        vec![
            "するます".to_string(),
            "さます".to_string(),
            "せます".to_string(),
            "そます".to_string(),
        ]
    );
}

#[test]
fn kuru_verb_alternates_to_hiragana_stems() {
    // The conjugator emits hiragana stems for 来る (きます/こない): the
    // kanji shares no prefix, so decomposition falls to the godan
    // empty-prefix branch.
    let result = candidates("来る", "きます", PartOfSpeech::Verb);
    assert!(result.contains(&"来るます".to_string()));
    assert!(result.len() >= 3);
}

#[test]
fn i_adjective_adverbial_glues_forgot_form() {
    let result = candidates("高い", "高く", PartOfSpeech::IAdjective);
    assert_eq!(
        result,
        vec![
            "高いく".to_string(),
            "高っく".to_string(),
            "高ぐ".to_string(),
        ]
    );
}

#[test]
fn i_adjective_past_gets_voicing_and_tsu_drop() {
    let result = candidates("高い", "高かった", PartOfSpeech::IAdjective);
    assert_eq!(
        result,
        vec![
            "高いかった".to_string(),
            "高っかった".to_string(),
            "高がった".to_string(),
            "高かた".to_string(),
        ]
    );
}

#[test]
fn i_adjective_garu_survives_via_reverse_voicing() {
    let result = candidates("高い", "高がる", PartOfSpeech::IAdjective);
    assert_eq!(
        result,
        vec![
            "高いがる".to_string(),
            "高っがる".to_string(),
            "高かる".to_string(),
        ]
    );
}

#[test]
fn na_adjective_pure_chains_yield_fewer_than_three() {
    // Pinned loss (#503 plan): kanji stems have no kana to mutate, so the
    // two pure na-adjective rules cannot field three distractors.
    let na = candidates("静か", "静かな", PartOfSpeech::NaAdjective);
    assert_eq!(na, vec!["静かっな".to_string()]);

    let de = candidates("静か", "静かで", PartOfSpeech::NaAdjective);
    assert_eq!(de, vec!["静かっで".to_string(), "静かて".to_string()]);
}

#[rstest::rstest]
#[case::godan_masu("書く", "書きます", PartOfSpeech::Verb)]
#[case::ichidan_te("食べる", "食べて", PartOfSpeech::Verb)]
#[case::godan_tsu("買う", "買って", PartOfSpeech::Verb)]
#[case::i_adj_negative("高い", "高くない", PartOfSpeech::IAdjective)]
#[case::na_adj_te("静か", "静かで", PartOfSpeech::NaAdjective)]
fn every_candidate_is_non_empty_and_differs_from_correct(
    #[case] word: &str,
    #[case] correct: &str,
    #[case] pos: PartOfSpeech,
) {
    for candidate in candidates(word, correct, pos) {
        assert!(!candidate.is_empty());
        assert_ne!(candidate, correct);
    }
}
