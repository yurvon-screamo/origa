use origa::domain::JapaneseLevel;

pub fn extract_number_from_text(
    text: &str,
    pattern_start: &str,
    pattern_end: &str,
) -> Option<usize> {
    let start_idx = text.find(pattern_start)?;
    let after_start = &text[start_idx + pattern_start.len()..];
    let number_str = if pattern_end.is_empty() {
        after_start.trim()
    } else {
        let end_idx = after_start.find(pattern_end)?;
        after_start[..end_idx].trim()
    };
    number_str.parse::<usize>().ok()
}

pub fn parse_duolingo_module_unit(title: &str, is_ru: bool) -> Option<(usize, usize)> {
    // Try Russian format first if is_ru
    if is_ru {
        // Try "Модуль X Раздел Y" format
        if let Some(module_num) = extract_number_from_text(title, "Модуль ", " Раздел")
        {
            if let Some(unit_num) = extract_number_from_text(title, "Раздел ", "") {
                return Some((module_num, unit_num));
            }
        }

        // Try "Модуль X Юнит Y" format (alternative Russian)
        if let Some(module_num) = extract_number_from_text(title, "Модуль ", " Юнит") {
            if let Some(unit_num) = extract_number_from_text(title, "Юнит ", "") {
                return Some((module_num, unit_num));
            }
        }

        // Try extracting from ID if title parsing fails
        // e.g., "duolingo_ru_module_1_unit_2"
        if let Some(module_num) = extract_number_from_text(title, "module_", "_unit") {
            if let Some(unit_num) = extract_number_from_text(title, "unit_", "") {
                return Some((module_num, unit_num));
            }
        }
    } else {
        // Try English format "Section X Unit Y"
        if let Some(module_num) = extract_number_from_text(title, "Section ", " Unit") {
            if let Some(unit_num) = extract_number_from_text(title, "Unit ", "") {
                return Some((module_num, unit_num));
            }
        }

        // Try "Module X Unit Y" format (alternative English)
        if let Some(module_num) = extract_number_from_text(title, "Module ", " Unit") {
            if let Some(unit_num) = extract_number_from_text(title, "Unit ", "") {
                return Some((module_num, unit_num));
            }
        }

        // Try extracting from ID if title parsing fails
        if let Some(module_num) = extract_number_from_text(title, "module_", "_unit") {
            if let Some(unit_num) = extract_number_from_text(title, "unit_", "") {
                return Some((module_num, unit_num));
            }
        }
    }

    // Log warning if parsing fails
    tracing::warn!("Failed to parse Duolingo module/unit from title: {}", title);
    None
}

pub fn parse_migii_level_lesson(title: &str) -> Option<(JapaneseLevel, usize)> {
    let level = if title.contains("N5") {
        JapaneseLevel::N5
    } else if title.contains("N4") {
        JapaneseLevel::N4
    } else if title.contains("N3") {
        JapaneseLevel::N3
    } else if title.contains("N2") {
        JapaneseLevel::N2
    } else if title.contains("N1") {
        JapaneseLevel::N1
    } else {
        return None;
    };

    let lesson_num = extract_number_from_text(title, "Урок ", "")
        .or_else(|| extract_number_from_text(title, "Lesson ", ""))?;

    Some((level, lesson_num))
}

pub fn parse_minna_lesson(title: &str) -> Option<usize> {
    // Try Russian format "Урок X"
    if let Some(num) = extract_number_from_text(title, "Урок ", "") {
        return Some(num);
    }

    // Try English format "Lesson X"
    if let Some(num) = extract_number_from_text(title, "Lesson ", "") {
        return Some(num);
    }

    // Try ID format "minna_n5_X" or "minna_n4_X"
    let re_patterns = ["minna_n5_", "minna_n4_"];
    for pattern in re_patterns {
        if title.contains(pattern) {
            if let Some(start_idx) = title.find(pattern) {
                let start = start_idx + pattern.len();
                let rest = &title[start..];
                let end = rest.find('_').unwrap_or(rest.len());
                let num_str = &rest[..end];
                if let Ok(num) = num_str.parse::<usize>() {
                    return Some(num);
                }
            }
        }
    }

    // Log warning if parsing fails
    tracing::warn!("Failed to parse Minna lesson number from title: {}", title);
    None
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Duolingo 「RU」 - Модуль 5 Раздел 1", true, Some((5, 1)))]
    #[case("Duolingo 「RU」 - Модуль 6 Раздел 48", true, Some((6, 48)))]
    #[case("Duolingo 「RU」 - Модуль 1 Раздел 1", true, Some((1, 1)))]
    #[case("Invalid title", true, None)]
    #[case("Duolingo - без модуля", true, None)]
    fn test_parse_duolingo_module_unit_ru(
        #[case] title: &str,
        #[case] is_ru: bool,
        #[case] expected: Option<(usize, usize)>,
    ) {
        assert_eq!(parse_duolingo_module_unit(title, is_ru), expected);
    }

    #[rstest]
    #[case("Duolingo 「EN」 - Section 5 Unit 1", false, Some((5, 1)))]
    #[case("Duolingo 「EN」 - Section 10 Unit 25", false, Some((10, 25)))]
    #[case("Duolingo - Section 1 Unit 1", false, Some((1, 1)))]
    #[case("Invalid title", false, None)]
    #[case("Duolingo - без unit", false, None)]
    fn test_parse_duolingo_module_unit_en(
        #[case] title: &str,
        #[case] is_ru: bool,
        #[case] expected: Option<(usize, usize)>,
    ) {
        assert_eq!(parse_duolingo_module_unit(title, is_ru), expected);
    }

    #[rstest]
    #[case("Migii N1 Урок 1", Some((JapaneseLevel::N1, 1)))]
    #[case("Migii N5 Lesson 10", Some((JapaneseLevel::N5, 10)))]
    #[case("Migii N4 Урок 5", Some((JapaneseLevel::N4, 5)))]
    #[case("Migii N3 Lesson 20", Some((JapaneseLevel::N3, 20)))]
    #[case("Migii N2 Урок 15", Some((JapaneseLevel::N2, 15)))]
    #[case("Migii without level", None)]
    #[case("Invalid title", None)]
    fn test_parse_migii_level_lesson(
        #[case] title: &str,
        #[case] expected: Option<(JapaneseLevel, usize)>,
    ) {
        assert_eq!(parse_migii_level_lesson(title), expected);
    }

    #[rstest]
    #[case("Minna no Nihongo N5 Урок 1", Some(1))]
    #[case("minna_n5_1", Some(1))]
    #[case("minna_n4_26", Some(26))]
    #[case("Minna no Nihongo N5 Lesson 5", Some(5))]
    #[case("minna_n5_50", Some(50))]
    #[case("minna_n4_1", Some(1))]
    #[case("Invalid title", None)]
    #[case("minna without number", None)]
    fn test_parse_minna_lesson(#[case] title: &str, #[case] expected: Option<usize>) {
        assert_eq!(parse_minna_lesson(title), expected);
    }
}
