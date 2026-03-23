pub trait JapaneseChar {
    fn is_japanese(&self) -> bool;
    fn is_hiragana(&self) -> bool;
    fn is_katakana(&self) -> bool;
    fn is_kanji(&self) -> bool;
}

pub trait JapaneseText {
    fn is_japanese(&self) -> bool;
    fn contains_japanese(&self) -> bool;
    fn contains_kanji(&self) -> bool;
}

impl JapaneseChar for char {
    fn is_japanese(&self) -> bool {
        self.is_hiragana() || self.is_katakana() || self.is_kanji()
    }

    fn is_hiragana(&self) -> bool {
        ('\u{3040}'..='\u{309F}').contains(self)
    }

    fn is_katakana(&self) -> bool {
        ('\u{30A0}'..='\u{30FF}').contains(self)
    }

    fn is_kanji(&self) -> bool {
        ('\u{4E00}'..='\u{9FFF}').contains(self)
            || ('\u{3400}'..='\u{4DBF}').contains(self)
            || ('\u{20000}'..='\u{2A6DF}').contains(self)
    }
}

impl JapaneseText for str {
    fn is_japanese(&self) -> bool {
        self.chars().all(|c| c.is_japanese())
    }

    fn contains_japanese(&self) -> bool {
        self.chars().any(|c| c.is_japanese())
    }

    fn contains_kanji(&self) -> bool {
        self.chars().any(|c| c.is_kanji())
    }
}

pub fn filter_japanese_text(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_japanese() || is_cjk_punctuation(c) {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn is_cjk_punctuation(c: char) -> bool {
    ('\u{3000}'..='\u{303F}').contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case('あ', true, false, false, true)]
    #[case('い', true, false, false, true)]
    #[case('う', true, false, false, true)]
    #[case('ア', false, true, false, true)]
    #[case('イ', false, true, false, true)]
    #[case('ウ', false, true, false, true)]
    #[case('日', false, false, true, true)]
    #[case('本', false, false, true, true)]
    #[case('語', false, false, true, true)]
    #[case('a', false, false, false, false)]
    #[case('Z', false, false, false, false)]
    #[case('1', false, false, false, false)]
    fn test_japanese_char_classification(
        #[case] input: char,
        #[case] is_hiragana: bool,
        #[case] is_katakana: bool,
        #[case] is_kanji: bool,
        #[case] is_japanese: bool,
    ) {
        assert_eq!(input.is_hiragana(), is_hiragana);
        assert_eq!(input.is_katakana(), is_katakana);
        assert_eq!(input.is_kanji(), is_kanji);
        assert_eq!(input.is_japanese(), is_japanese);
    }

    #[rstest]
    #[case("こんにちは", true, true, false)]
    #[case("コンニチハ", true, true, false)]
    #[case("日本語", true, true, true)]
    #[case("こんにちは日本語", true, true, true)]
    #[case("Hello", false, false, false)]
    #[case("", true, false, false)]
    fn test_japanese_text_is_japanese(
        #[case] input: &str,
        #[case] expected_is_japanese: bool,
        #[case] expected_contains_japanese: bool,
        #[case] expected_contains_kanji: bool,
    ) {
        assert_eq!(input.is_japanese(), expected_is_japanese);
        assert_eq!(input.contains_japanese(), expected_contains_japanese);
        assert_eq!(input.contains_kanji(), expected_contains_kanji);
    }

    #[rstest]
    #[case("こんにちは世界", "こんにちは世界")]
    #[case("Hello Worldこんにちは", "こんにちは")]
    #[case("日本語123テスト", "日本語 テスト")]
    #[case("Test", "")]
    #[case("あいうえお", "あいうえお")]
    #[case("  ", "")]
    fn test_filter_japanese_text(#[case] input: &str, #[case] expected: &str) {
        let result = filter_japanese_text(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_text_with_punctuation() {
        let input = "こんにちは、世界！";
        let result = filter_japanese_text(input);
        assert_eq!(result, "こんにちは、世界");
    }

    #[test]
    fn test_cjk_punctuation_preservation() {
        let text_with_punctuation = "こんにちは。テスト。";
        let result = filter_japanese_text(text_with_punctuation);
        assert!(result.contains("。"));
        assert!(result.contains("こんにちは"));
        assert!(result.contains("テスト"));
    }

    #[test]
    fn test_multiple_spaces_collapsed() {
        let input = "こんにちは  世界   テスト";
        let result = filter_japanese_text(input);
        assert_eq!(result, "こんにちは 世界 テスト");
    }
}
