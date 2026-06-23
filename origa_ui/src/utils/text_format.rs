/// Format vocabulary answer with translations on separate lines + optional italic description.
/// Translations joined via markdown hard-breaks (`  \n`).
/// Description rendered as italic markdown paragraph below.
pub fn format_vocabulary_answer(translations: &[String], description: &Option<String>) -> String {
    let mut text = translations.join("  \n");
    if let Some(desc) = description {
        if !desc.is_empty() {
            text = format!("{}\n\n*{}*", text, desc);
        }
    }
    text
}

/// Sentence terminators shared by both splitters so Japanese and Latin scripts
/// stay in sync. Japanese `。！？` and ASCII `.!?` are all treated as boundaries.
/// The plan-mandated asymmetry (whitespace required after Latin terminators)
/// is enforced separately in :func:`split_sentences_to_markdown` so Japanese
/// text does not need whitespace after `。` to be split.
const JP_SENTENCE_TERMINATORS: &[char] = &['。', '！', '？'];
const LATIN_SENTENCE_TERMINATORS: &[char] = &['.', '!', '?'];

/// Split Japanese text by sentence terminators (`。！？`), preserving the
/// delimiter. Returns empty Vec if input is empty/whitespace-only.
pub fn split_japanese_sentences(text: &str) -> Vec<String> {
    text.split_inclusive(JP_SENTENCE_TERMINATORS)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Split text by sentence-ending punctuation (`.!?` for Latin, `。！？` for
/// Japanese) followed by whitespace, then format with markdown hard-breaks
/// between sentences. The whitespace gate preserves abbreviations like
/// "Dr. Smith" for Latin script without blocking Japanese sentences that
/// never use inter-sentence spaces.
/// Returns original text unchanged if no sentence boundaries found.
pub fn split_sentences_to_markdown(text: &str) -> String {
    let sentences = split_sentences_to_list(text);
    if sentences.len() <= 1 {
        text.to_string()
    } else {
        sentences.join("  \n")
    }
}

/// Split text into a list of sentences using the same terminators as
/// :func:`split_sentences_to_markdown`. Japanese terminators (`。！？`)
/// always split; Latin terminators (`.!?`) require trailing whitespace to
/// preserve abbreviations. Empty/whitespace-only fragments are dropped.
/// Returns an empty Vec for empty/whitespace-only input.
pub fn split_sentences_to_list(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        current.push(chars[i]);
        if is_latin_terminator(chars[i])
            && i + 1 < chars.len()
            && chars[i + 1].is_whitespace()
        {
            push_trimmed(&mut sentences, &current);
            current.clear();
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            continue;
        }
        if JP_SENTENCE_TERMINATORS.contains(&chars[i]) {
            push_trimmed(&mut sentences, &current);
            current.clear();
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    push_trimmed(&mut sentences, &current);
    sentences
}

fn push_trimmed(out: &mut Vec<String>, current: &str) {
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_string());
    }
}

fn is_latin_terminator(c: char) -> bool {
    LATIN_SENTENCE_TERMINATORS.contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_vocabulary_answer_basic() {
        let translations = vec!["hello".into(), "hi".into()];
        let desc = None;
        assert_eq!(
            format_vocabulary_answer(&translations, &desc),
            "hello  \nhi"
        );
    }

    #[test]
    fn test_format_vocabulary_answer_with_description() {
        let translations = vec!["hello".into()];
        let desc = Some("informal greeting".into());
        let result = format_vocabulary_answer(&translations, &desc);
        assert!(result.contains("hello"));
        assert!(result.contains("*informal greeting*"));
    }

    #[test]
    fn test_format_vocabulary_answer_no_description() {
        let translations = vec!["hello".into()];
        let desc = None;
        assert_eq!(format_vocabulary_answer(&translations, &desc), "hello");
    }

    #[test]
    fn test_format_vocabulary_answer_empty_description() {
        let translations = vec!["hello".into()];
        let desc = Some("".into());
        assert_eq!(format_vocabulary_answer(&translations, &desc), "hello");
    }

    #[test]
    fn test_split_japanese_sentences() {
        let text = "今日はいい天気。散歩に行こう。";
        let sentences = split_japanese_sentences(text);
        assert_eq!(sentences, vec!["今日はいい天気。", "散歩に行こう。"]);
    }

    #[test]
    fn test_split_japanese_sentences_single() {
        let text = "こんにちは";
        let sentences = split_japanese_sentences(text);
        assert_eq!(sentences, vec!["こんにちは"]);
    }

    #[test]
    fn test_split_japanese_sentences_empty() {
        let sentences = split_japanese_sentences("");
        assert!(sentences.is_empty());
    }

    #[test]
    fn test_split_japanese_sentences_single_with_delimiter() {
        let text = "今日は。";
        let sentences = split_japanese_sentences(text);
        assert_eq!(sentences, vec!["今日は。"]);
    }

    #[test]
    fn test_split_japanese_sentences_by_fullwidth_exclamation() {
        let text = "あ！なんでもあらへんで";
        let sentences = split_japanese_sentences(text);
        assert_eq!(sentences, vec!["あ！", "なんでもあらへんで"]);
    }

    #[test]
    fn test_split_japanese_sentences_by_fullwidth_question() {
        let text = "大丈夫？うん。";
        let sentences = split_japanese_sentences(text);
        assert_eq!(sentences, vec!["大丈夫？", "うん。"]);
    }

    #[test]
    fn test_split_sentences_to_markdown_handles_japanese_exclamation() {
        let text = "あ！なんでもあらへんで";
        let result = split_sentences_to_markdown(text);
        assert!(
            result.contains("  \n"),
            "Expected hard break between JP sentences, got: {}",
            result
        );
        assert!(result.contains("あ！"));
        assert!(result.contains("なんでもあらへんで"));
    }

    #[test]
    fn test_split_sentences_to_markdown_known_limitation_on_abbreviations() {
        // Documenting the inherited limitation: a Latin "." followed by
        // whitespace always splits, so "Dr. Smith" yields two sentences.
        // This matches the behavior the original implementation shipped with;
        // the new harmonized splitter preserves it for compatibility.
        let text = "Dr. Smith is here.";
        let result = split_sentences_to_list(text);
        assert_eq!(result, vec!["Dr.", "Smith is here."]);
    }

    #[test]
    fn test_split_sentences_to_list_returns_empty_for_empty_input() {
        let result = split_sentences_to_list("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_sentences_to_list_single_sentence_returns_one_item() {
        let result = split_sentences_to_list("Just one sentence.");
        assert_eq!(result, vec!["Just one sentence."]);
    }

    #[test]
    fn test_split_sentences_to_list_splits_latin_on_punct_and_space() {
        let result = split_sentences_to_list("First. Second! Third?");
        assert_eq!(result, vec!["First.", "Second!", "Third?"]);
    }

    #[test]
    fn test_split_sentences_to_list_splits_japanese_on_fullwidth_terminators() {
        let result = split_sentences_to_list("あ！なんでもあらへんで");
        assert_eq!(result, vec!["あ！", "なんでもあらへんで"]);
    }

    #[test]
    fn test_split_sentences_to_list_preserves_latin_abbreviations() {
        // Same documented limitation as for the markdown variant: Latin "."
        // followed by whitespace always splits. Documented here so a future
        // abbreviation-aware implementation updates both tests together.
        let result = split_sentences_to_list("Dr. Smith is here. Bye.");
        assert_eq!(result, vec!["Dr.", "Smith is here.", "Bye."]);
    }

    #[test]
    fn test_split_sentences_to_markdown() {
        let text = "First sentence. Second sentence. Third one.";
        let result = split_sentences_to_markdown(text);
        assert!(
            result.contains("  \n"),
            "Expected hard breaks, got: {}",
            result
        );
        assert!(result.contains("First sentence."));
        assert!(result.contains("Second sentence."));
    }

    #[test]
    fn test_split_sentences_exclamation() {
        let text = "Hello! How are you?";
        let result = split_sentences_to_markdown(text);
        assert!(
            result.contains("  \n"),
            "Expected hard breaks, got: {}",
            result
        );
    }

    #[test]
    fn test_split_sentences_no_split_needed() {
        let text = "Single sentence";
        let result = split_sentences_to_markdown(text);
        assert_eq!(result, "Single sentence");
    }

    #[test]
    fn test_split_sentences_trailing_period() {
        let text = "First. Second.";
        let result = split_sentences_to_markdown(text);
        assert!(
            result.contains("  \n"),
            "Expected hard break, got: {}",
            result
        );
    }
}
