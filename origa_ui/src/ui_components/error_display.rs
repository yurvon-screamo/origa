use origa::domain::OrigaError;

#[allow(dead_code)]
pub fn format_missing_data(error: &OrigaError) -> String {
    match error {
        OrigaError::TranslationNotFound { word, .. } => format!("[нет перевода: {}]", word),
        OrigaError::KanjiNotFound { kanji } => format!("[нет описания: {}]", kanji),
        OrigaError::GrammarRuleNotFound { rule_id } => format!("[правило не найдено: {}]", rule_id),
        OrigaError::GrammarContentNotFound { .. } => "[контент недоступен]".to_string(),
        OrigaError::InvalidQuestion { reason } => format!("[ошибка: {}]", reason),
        OrigaError::InvalidAnswer { reason } => format!("[ошибка: {}]", reason),
        _ => "[данные недоступны]".to_string(),
    }
}

#[allow(dead_code)]
pub fn answer_or_error(answer: Result<origa::domain::Answer, OrigaError>) -> String {
    answer
        .map(|a| a.text().to_string())
        .unwrap_or_else(|e| format_missing_data(&e))
}

#[allow(dead_code)]
pub fn question_or_error(question: Result<origa::domain::Question, OrigaError>) -> String {
    question
        .map(|q| q.text().to_string())
        .unwrap_or_else(|e| format_missing_data(&e))
}
