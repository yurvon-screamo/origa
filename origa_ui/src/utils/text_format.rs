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

/// Split Japanese text by sentence delimiter `。`, preserving the delimiter.
/// Returns empty Vec if input is empty/whitespace-only.
pub fn split_japanese_sentences(text: &str) -> Vec<String> {
    text.split_inclusive('。')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Split text by sentence-ending punctuation (. ! ?) followed by whitespace,
/// format with markdown hard-breaks between sentences.
/// Note: Does not handle abbreviations (e.g., "Dr. Smith" will be split).
/// Returns original text unchanged if no sentence boundaries found.
pub fn split_sentences_to_markdown(text: &str) -> String {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        current.push(chars[i]);
        if matches!(chars[i], '.' | '!' | '?')
            && i + 1 < chars.len()
            && chars[i + 1].is_whitespace()
        {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_string());
            }
            current.clear();
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        sentences.push(trimmed.to_string());
    }
    if sentences.len() <= 1 {
        return text.to_string();
    }
    sentences.join("  \n")
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
