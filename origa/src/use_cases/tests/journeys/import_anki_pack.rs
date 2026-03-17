use serde_json::Value;

const FIELD_SEPARATOR: char = '\x1f';

fn clean_html_text(raw: &str) -> String {
    let mut result = String::new();
    let mut inside_tag = false;
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '<' {
            inside_tag = true;
            result.push(' ');
            i += 1;
            continue;
        }
        if chars[i] == '>' && inside_tag {
            inside_tag = false;
            i += 1;
            continue;
        }
        if inside_tag {
            i += 1;
            continue;
        }
        if i + 5 < chars.len()
            && chars[i] == '&'
            && chars[i + 1] == 'n'
            && chars[i + 2] == 'b'
            && chars[i + 3] == 's'
            && chars[i + 4] == 'p'
            && chars[i + 5] == ';'
        {
            result.push(' ');
            i += 6;
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn find_field_indices(
    models_json: &str,
    word_tag: &str,
    translation_tag: Option<&str>,
) -> Result<(usize, Option<usize>), String> {
    let models: Value =
        serde_json::from_str(models_json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let mut word_index = None;
    let mut translation_index = None;

    if let Some(models_map) = models.as_object() {
        for (_model_id, model_data) in models_map {
            if let Some(fields) = model_data["flds"].as_array() {
                for (index, field) in fields.iter().enumerate() {
                    if let Some(field_name) = field["name"].as_str() {
                        let field_name_lower = field_name.to_lowercase();
                        if field_name_lower == word_tag.to_lowercase() {
                            word_index = Some(index);
                        }
                        if let Some(trans_tag) = translation_tag
                            && field_name_lower == trans_tag.to_lowercase()
                        {
                            translation_index = Some(index);
                        }
                    }
                }

                if word_index.is_some()
                    && (translation_tag.is_none() || translation_index.is_some())
                {
                    break;
                }
            }
        }
    }

    let word_index =
        word_index.ok_or_else(|| format!("Field '{}' not found in Anki deck models", word_tag))?;

    Ok((word_index, translation_index))
}

fn parse_anki_fields(
    flds_str: &str,
    word_index: usize,
    translation_index: Option<usize>,
) -> (String, Option<String>) {
    let fields: Vec<&str> = flds_str.split(FIELD_SEPARATOR).collect();

    let raw_word = fields.get(word_index).unwrap_or(&"");
    let word = clean_html_text(raw_word);

    let translation = if let Some(idx) = translation_index {
        let raw_translation = fields.get(idx).unwrap_or(&"");
        let cleaned = clean_html_text(raw_translation);
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    } else {
        None
    };

    (word, translation)
}

#[test]
fn clean_html_removes_simple_tags() {
    let input = "<b>太字</b>テスト";
    let result = clean_html_text(input);

    assert_eq!(result, "太字 テスト");
}

#[test]
fn clean_html_removes_nested_tags() {
    let input = "<div><span>日本語</span></div>";
    let result = clean_html_text(input);

    assert_eq!(result, "日本語");
}

#[test]
fn clean_html_replaces_nbsp() {
    let input = "test&nbsp;space";
    let result = clean_html_text(input);

    assert_eq!(result, "test space");
}

#[test]
fn clean_html_handles_combined_html_and_nbsp() {
    let input = "<div>日本語&nbsp;テスト</div>";
    let result = clean_html_text(input);

    assert_eq!(result, "日本語 テスト");
}

#[test]
fn clean_html_handles_multiple_nbsp() {
    let input = "a&nbsp;b&nbsp;c";
    let result = clean_html_text(input);

    assert_eq!(result, "a b c");
}

#[test]
fn clean_html_preserves_plain_text() {
    let input = "日本語テスト";
    let result = clean_html_text(input);

    assert_eq!(result, "日本語テスト");
}

#[test]
fn clean_html_trims_whitespace() {
    let input = "  <b>word</b>  ";
    let result = clean_html_text(input);

    assert_eq!(result, "word");
}

#[test]
fn clean_html_handles_empty_string() {
    let input = "";
    let result = clean_html_text(input);

    assert!(result.is_empty());
}

#[test]
fn clean_html_handles_complex_html() {
    let input = "<p class=\"note\">Hello&nbsp;<strong>World</strong></p>";
    let result = clean_html_text(input);

    assert_eq!(result, "Hello World");
}

#[test]
fn find_field_indices_finds_word_field() {
    let models = r#"{
        "123": {
            "flds": [
                {"name": "Expression"},
                {"name": "Meaning"}
            ]
        }
    }"#;

    let result = find_field_indices(models, "expression", None);

    assert!(result.is_ok());
    let (word_index, _) = result.unwrap();
    assert_eq!(word_index, 0);
}

#[test]
fn find_field_indices_finds_both_fields() {
    let models = r#"{
        "123": {
            "flds": [
                {"name": "Expression"},
                {"name": "Meaning"}
            ]
        }
    }"#;

    let result = find_field_indices(models, "expression", Some("meaning"));

    assert!(result.is_ok());
    let (word_index, translation_index) = result.unwrap();
    assert_eq!(word_index, 0);
    assert_eq!(translation_index, Some(1));
}

#[test]
fn find_field_indices_is_case_insensitive() {
    let models = r#"{
        "123": {
            "flds": [
                {"name": "EXPRESSION"},
                {"name": "MEANING"}
            ]
        }
    }"#;

    let result = find_field_indices(models, "expression", Some("meaning"));

    assert!(result.is_ok());
    let (word_index, translation_index) = result.unwrap();
    assert_eq!(word_index, 0);
    assert_eq!(translation_index, Some(1));
}

#[test]
fn find_field_indices_returns_error_for_missing_field() {
    let models = r#"{
        "123": {
            "flds": [
                {"name": "Front"},
                {"name": "Back"}
            ]
        }
    }"#;

    let result = find_field_indices(models, "expression", None);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("expression"));
}

#[test]
fn find_field_indices_handles_multiple_models() {
    let models = r#"{
        "111": {
            "flds": [
                {"name": "Front"},
                {"name": "Back"}
            ]
        },
        "222": {
            "flds": [
                {"name": "Expression"},
                {"name": "Reading"},
                {"name": "Meaning"}
            ]
        }
    }"#;

    let result = find_field_indices(models, "expression", Some("meaning"));

    assert!(result.is_ok());
    let (word_index, translation_index) = result.unwrap();
    assert_eq!(word_index, 0);
    assert_eq!(translation_index, Some(2));
}

#[test]
fn find_field_indices_returns_none_for_missing_translation_when_optional() {
    let models = r#"{
        "123": {
            "flds": [
                {"name": "Expression"},
                {"name": "Reading"}
            ]
        }
    }"#;

    let result = find_field_indices(models, "expression", None);

    assert!(result.is_ok());
    let (_, translation_index) = result.unwrap();
    assert!(translation_index.is_none());
}

#[test]
fn find_field_indices_handles_invalid_json() {
    let models = "not valid json";

    let result = find_field_indices(models, "expression", None);

    assert!(result.is_err());
}

#[test]
fn parse_anki_fields_extracts_word_and_translation() {
    let flds = format!("日本語{}Japanese{}", FIELD_SEPARATOR, FIELD_SEPARATOR);

    let (word, translation) = parse_anki_fields(&flds, 0, Some(1));

    assert_eq!(word, "日本語");
    assert_eq!(translation, Some("Japanese".to_string()));
}

#[test]
fn parse_anki_fields_handles_missing_translation_index() {
    let flds = format!("日本語{}Japanese{}", FIELD_SEPARATOR, FIELD_SEPARATOR);

    let (word, translation) = parse_anki_fields(&flds, 0, None);

    assert_eq!(word, "日本語");
    assert!(translation.is_none());
}

#[test]
fn parse_anki_fields_cleans_html_in_word() {
    let flds = format!(
        "<b>日本語</b>{}Japanese{}",
        FIELD_SEPARATOR, FIELD_SEPARATOR
    );

    let (word, _) = parse_anki_fields(&flds, 0, Some(1));

    assert_eq!(word, "日本語");
}

#[test]
fn parse_anki_fields_cleans_html_in_translation() {
    let flds = format!(
        "日本語{}<i>Japanese</i>{}",
        FIELD_SEPARATOR, FIELD_SEPARATOR
    );

    let (_, translation) = parse_anki_fields(&flds, 0, Some(1));

    assert_eq!(translation, Some("Japanese".to_string()));
}

#[test]
fn parse_anki_fields_handles_empty_word() {
    let flds = format!("{}Japanese{}", FIELD_SEPARATOR, FIELD_SEPARATOR);

    let (word, _) = parse_anki_fields(&flds, 0, Some(1));

    assert!(word.is_empty());
}

#[test]
fn parse_anki_fields_handles_out_of_bounds_index() {
    let flds = format!("日本語{}", FIELD_SEPARATOR);

    let (word, translation) = parse_anki_fields(&flds, 0, Some(5));

    assert_eq!(word, "日本語");
    assert!(translation.is_none() || translation.unwrap().is_empty());
}

#[test]
fn field_separator_is_correct() {
    let test_str = format!("a{}b{}c", FIELD_SEPARATOR, FIELD_SEPARATOR);
    let parts: Vec<&str> = test_str.split(FIELD_SEPARATOR).collect();

    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], "a");
    assert_eq!(parts[1], "b");
    assert_eq!(parts[2], "c");
}

#[test]
fn parse_multiple_cards_from_fields() {
    let cards_data = [
        (
            format!(
                "日本語{}Japanese language{}",
                FIELD_SEPARATOR, FIELD_SEPARATOR
            ),
            "日本語",
            "Japanese language",
        ),
        (
            format!("勉強{}study{}", FIELD_SEPARATOR, FIELD_SEPARATOR),
            "勉強",
            "study",
        ),
        (
            format!("<b>太字</b>{}bold{}", FIELD_SEPARATOR, FIELD_SEPARATOR),
            "太字",
            "bold",
        ),
    ];

    for (flds, expected_word, expected_translation) in cards_data {
        let (word, translation) = parse_anki_fields(&flds, 0, Some(1));
        assert_eq!(word, expected_word);
        assert_eq!(translation.unwrap(), expected_translation);
    }
}

#[test]
fn empty_word_is_detected_as_skip_candidate() {
    let flds = format!("{}translation{}", FIELD_SEPARATOR, FIELD_SEPARATOR);
    let (word, _) = parse_anki_fields(&flds, 0, Some(1));

    let should_skip = word.is_empty();
    assert!(should_skip);
}
