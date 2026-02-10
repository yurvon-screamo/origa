/// Преобразует глагол в て-форму
/// Правила для групп глаголов:
/// - Группа II (годан): зависит от последнего слога
/// - Группа I (итидан): る → て
/// - Группа III (неправильные): する → して, くる/来る → きて
pub fn to_te_form(word: &str) -> String {
    // Проверяем на неправильные глаголы
    if word == "する" {
        return "して".to_string();
    }

    if word == "くる" || word == "来る" {
        return "きて".to_string();
    }

    // Определяем группу по последнему символу
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    let last_char = chars[chars.len() - 1];

    // Группа I (итидан) - оканчиваются на る, но не входят в группу II
    if last_char == 'る' {
        // Проверяем, не является ли это глаголом группы II
        // Глаголы группы II не оканчиваются наiru, ari, ori, uri
        if chars.len() >= 2 {
            let second_last = chars[chars.len() - 2];
            if !matches!(second_last, 'い' | 'え' | 'お' | 'う') {
                // Группа I - меняем る на て
                let mut result = word.to_string();
                result.pop(); // убираем る
                result.push('て');
                return result;
            }
        }
    }

    // Группа II (годан) - правила по последнему слогу
    match last_char {
        'く' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("いて");
            result
        }
        'ぐ' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("いで");
            result
        }
        'す' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("して");
            result
        }
        'つ' | 'る' | 'う' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("って");
            result
        }
        'ぬ' | 'ぶ' | 'む' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("んで");
            result
        }
        _ => word.to_string(),
    }
}

/// Преобразует глагол в основу глагола
pub fn to_main_view(word: &str) -> String {
    // TODO: исключения
    if word == "する" {
        return "し".to_string();
    }
    if word == "くる" || word == "来る" {
        return "来".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }
    let last_char = chars[chars.len() - 1];

    // Группа I - оканчиваются на る
    if last_char == 'る' && chars.len() >= 2 {
        let second_last = chars[chars.len() - 2];
        if !matches!(second_last, 'い' | 'え' | 'お' | 'う') {
            // Группа I - убираем る
            let mut result = word.to_string();
            result.pop();
            return result;
        }
    }

    // TODO: Группа II - меняем последний слог
    word.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_te_form_group2() {
        assert_eq!(to_te_form("行く"), "行って");
        assert_eq!(to_te_form("話す"), "話して");
        assert_eq!(to_te_form("読む"), "読んで");
        assert_eq!(to_te_form("書く"), "書いて");
        assert_eq!(to_te_form("泳ぐ"), "泳いで");
    }

    #[test]
    fn test_te_form_group1() {
        assert_eq!(to_te_form("食べる"), "食べて");
        assert_eq!(to_te_form("見る"), "見て");
    }

    #[test]
    fn test_te_form_irregular() {
        assert_eq!(to_te_form("する"), "して");
        assert_eq!(to_te_form("くる"), "きて");
        assert_eq!(to_te_form("来る"), "きて");
    }
}
