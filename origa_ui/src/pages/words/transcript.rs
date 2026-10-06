//! Transcript screen logic: sentence segmentation, the unknown-content
//! preselection heuristic, and the selection join.
//!
//! Pure functions only — the stage wiring lives in the modal.

use std::collections::HashSet;

/// Splits a transcript into sentences on Japanese and ASCII terminators
/// (。！？!?). Terminators stay at the end of their sentence; a non-empty
/// tail without a terminator becomes the last sentence; empty fragments
/// are dropped. Quotation marks (「」) are NOT tracked — a terminator
/// inside quotes cuts the sentence mid-quote, which is cosmetic here: the
/// selection join restores the original order and content.
pub(super) fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        current.push(c);
        if matches!(c, '。' | '！' | '？' | '!' | '?') {
            sentences.push(std::mem::take(&mut current));
        }
    }
    if !current.trim().is_empty() {
        sentences.push(current);
    }
    sentences
        .into_iter()
        .map(|sentence| sentence.trim().to_string())
        .filter(|sentence| !sentence.is_empty())
        .collect()
}

/// CJK unified ideographs plus the iteration mark 々 (U+3005). Long-vowel
/// mark ー (U+30FC) and middle dot ・ (U+30FB) are deliberately excluded —
/// both appear inside hiragana words, so counting them as katakana would
/// flag every "らーめん" as unknown content.
fn is_kanji(c: char) -> bool {
    matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3005}')
}

/// Katakana LETTERS only (U+30A1..=U+30FA): loanwords are a decent
/// unknown-content proxy. The block's punctuation (ー ・ ヽ ヾ) is excluded
/// for the same reason as above.
fn is_katakana_letter(c: char) -> bool {
    matches!(c, '\u{30A1}'..='\u{30FA}')
}

/// Preselection heuristic: a sentence is "worth analyzing" when it carries
/// content beyond the user's known kanji, or any katakana letter. Pure
/// hiragana/latin/digit sentences are not preselected. An estimate by
/// design — the user corrects the checkboxes.
pub(super) fn sentence_has_unknown_content(sentence: &str, known_kanji: &HashSet<char>) -> bool {
    sentence
        .chars()
        .any(|c| (is_kanji(c) && !known_kanji.contains(&c)) || is_katakana_letter(c))
}

/// Whether the transcript screen should appear: more than one sentence
/// means real selection value; a single-sentence transcript (live
/// one-phrase cases) goes straight to analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TranscriptDecision {
    ShowScreen,
    DirectAnalysis,
}

pub(super) fn transcript_entry(sentences_count: usize) -> TranscriptDecision {
    if sentences_count > 1 {
        TranscriptDecision::ShowScreen
    } else {
        TranscriptDecision::DirectAnalysis
    }
}

/// Concatenates the selected sentences in their original order.
pub(super) fn join_selected(sentences: &[String], selected: &HashSet<usize>) -> String {
    sentences
        .iter()
        .enumerate()
        .filter(|(index, sentence)| selected.contains(index) && !sentence.trim().is_empty())
        .map(|(_, sentence)| sentence.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case::terminators_kept(
        "私は本を読みます。面白いです！",
        vec!["私は本を読みます。", "面白いです！"]
    )]
    #[case::ascii_terminators("Really? Yes!", vec!["Really?", "Yes!"])]
    #[case::no_terminator_is_one_sentence("今日は暑いですね", vec!["今日は暑いですね"])]
    #[case::empty_tail_dropped("行く。", vec!["行く。"])]
    #[case::empty_input("", vec![])]
    fn segmentation_covers_terminators_and_tails(#[case] text: &str, #[case] expected: Vec<&str>) {
        assert_eq!(split_sentences(text), expected);
    }

    #[rstest]
    #[case::unknown_kanji_flags("難しい本", &['本'], true)]
    #[case::known_kanji_only("本", &['本'], false)]
    #[case::katakana_letter_flags("ラーメン", &[], true)]
    #[case::hiragana_only_not_flagged("おいしいですね", &[], false)]
    #[case::latin_not_flagged("ABC 123", &[], false)]
    fn unknown_content_heuristic_flags_kanji_and_katakana(
        #[case] sentence: &str,
        #[case] known_kanji: &[char],
        #[case] expected: bool,
    ) {
        let known: HashSet<char> = known_kanji.iter().copied().collect();
        assert_eq!(sentence_has_unknown_content(sentence, &known), expected);
    }

    #[test]
    fn boundary_characters_are_neither_kanji_nor_katakana_letters() {
        let known: HashSet<char> = HashSet::new();
        // ー (long vowel) and ・ (middle dot) live in the katakana block
        // but appear in hiragana words — they must NOT flag content.
        assert!(!sentence_has_unknown_content("らーめん・おいしい", &known));
        // 々 is a kanji (iteration mark).
        assert!(sentence_has_unknown_content("人々", &known));
        // Full-width ！？ are terminators, not content markers.
        assert!(!sentence_has_unknown_content("すごい！", &known));
    }

    #[rstest]
    #[case::two_sentences_show(2, TranscriptDecision::ShowScreen)]
    #[case::one_sentence_direct(1, TranscriptDecision::DirectAnalysis)]
    #[case::empty_direct(0, TranscriptDecision::DirectAnalysis)]
    fn entry_decision_shows_the_screen_only_for_multiple_sentences(
        #[case] count: usize,
        #[case] expected: TranscriptDecision,
    ) {
        assert_eq!(transcript_entry(count), expected);
    }

    #[test]
    fn join_keeps_original_order_and_skips_unselected() {
        let sentences = vec![
            "一つ目。".to_string(),
            "二つ目。".to_string(),
            "三つ目。".to_string(),
        ];
        let mut selected = HashSet::new();
        selected.insert(0);
        selected.insert(2);
        assert_eq!(join_selected(&sentences, &selected), "一つ目。三つ目。");
    }

    #[test]
    fn join_with_nothing_selected_is_empty() {
        let sentences = vec!["一つ目。".to_string()];
        assert_eq!(join_selected(&sentences, &HashSet::new()), "");
    }
}
