//! Фронт тренировки грамматики: японская строка примера без перевода.
//!
//! Формат `examples` (cdn/grammar): последовательные code-fence по две
//! строки — японское предложение, затем перевод. Фронт показывает только
//! японскую строку первого примера: перевод раскрывает смысл, а смысл и
//! есть ответ (спека §Тренировка: «пример с конструкцией → её смысл»).

/// Первая непустая строка первого code-fence: японское предложение без
/// перевода. `None`, если examples пуст или fence не найден — фронт
/// вырождается в заголовок конструкции.
pub fn grammar_example_front(examples_markdown: &str) -> Option<String> {
    let fence_start = examples_markdown.find("```")?;
    let after_fence = &examples_markdown[fence_start + 3..];
    // Пропускаем строку языка fence (в контенте грамматики она пустая).
    let body_start = after_fence.find('\n')? + 1;
    let body = &after_fence[body_start..];
    // Незакрытый fence — битые данные: фронтом становится заголовок.
    let body_end = body.find("```")?;
    body[..body_end]
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

/// Первый пример целиком — code-fence с японской строкой и переводом.
/// Блок ответа тренировки грамматики; `None` при пустых/битых examples.
pub fn first_example_markdown(examples_markdown: &str) -> Option<String> {
    let start = examples_markdown.find("```")?;
    let after = &examples_markdown[start..];
    let closing = after[3..].find("```")? + 3;
    Some(after[..closing + 3].to_string())
}

/// Единый источник текста фронта тренировки грамматики (рендер и
/// озвучка — один и тот же текст): JP-строка первого примера без
/// emphasis-маркеров, при пустых/битых examples — паттерн. Дедупликация:
/// раньше фронт собирался здесь, а озвучка собирала бы свой вариант —
/// и кнопка могла бы озвучить строку, которой нет на экране.
pub fn grammar_training_front_text(pattern: &str, examples_markdown: &str) -> String {
    grammar_example_front(examples_markdown)
        .map(|line| crate::ui_components::strip_emphasis_markers(&line))
        .unwrap_or_else(|| pattern.to_string())
}

#[cfg(test)]
mod grammar_example_front_tests {
    use super::*;

    #[test]
    fn returns_japanese_line_without_translation() {
        // Arrange: реальный формат контента — JP-строка, затем перевод
        let examples = "```\n私は学生です。\nI am a student.\n```";

        // Act
        let front = grammar_example_front(examples);

        // Assert
        assert_eq!(front.as_deref(), Some("私は学生です。"));
    }

    #[test]
    fn takes_only_the_first_fence_of_several() {
        // Arrange
        let examples = "```\n今日は晴れです。\nIt is sunny today.\n```\n\n```\n猫がいます。\nThere is a cat.\n```";

        // Act
        let front = grammar_example_front(examples);

        // Assert
        assert_eq!(front.as_deref(), Some("今日は晴れです。"));
    }

    #[test]
    fn skips_blank_lines_inside_the_fence() {
        // Arrange
        let examples = "```\n\n明日行きます。\nI will go tomorrow.\n```";

        // Act / Assert
        assert_eq!(
            grammar_example_front(examples).as_deref(),
            Some("明日行きます。")
        );
    }

    #[rstest::rstest]
    #[case::empty(String::new())]
    #[case::no_fence("просто текст без разметки")]
    #[case::unclosed_fence("```\n訳があります")]
    #[test]
    fn returns_none_when_no_complete_example(#[case] examples: String) {
        assert_eq!(grammar_example_front(&examples), None);
        assert_eq!(first_example_markdown(&examples), None);
    }

    #[test]
    fn first_example_markdown_returns_single_closed_fence() {
        // Arrange: два примера — в ответе только первый, целиком
        let examples =
            "```\n私は学生です。\nI am a student.\n```\n\n```\n猫がいます。\nThere is a cat.\n```";

        // Act / Assert
        assert_eq!(
            first_example_markdown(examples).as_deref(),
            Some("```\n私は学生です。\nI am a student.\n```")
        );
    }

    #[test]
    fn training_front_text_is_japanese_line_without_emphasis_markers() {
        // Arrange: реальный формат контента — маркеры emphasis только
        // в JP-строке (аудит CDN-данных, strip_emphasis_markers)
        let examples = "```\n彼は**学生**です。\nHe is a student.\n```";

        // Act / Assert: рендер и озвучка получают один и тот же текст
        assert_eq!(
            grammar_training_front_text("～は～です", examples),
            "彼は学生です。"
        );
    }

    #[test]
    fn training_front_text_falls_back_to_pattern_without_examples() {
        // Arrange / Act / Assert
        assert_eq!(
            grammar_training_front_text("～たことがある", ""),
            "～たことがある"
        );
    }

    #[test]
    fn training_front_text_falls_back_to_pattern_on_broken_fence() {
        // Arrange: незакрытый fence — битые данные (кейс из матрицы
        // grammar_example_front)
        // Act / Assert
        assert_eq!(
            grammar_training_front_text("～わけではない", "```\n訳があります"),
            "～わけではない"
        );
    }
}
