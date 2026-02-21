#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbGroup {
    Ichidan,
    Godan,
    Irregular,
}

pub fn classify_verb(word: &str) -> VerbGroup {
    if word == "する" || word == "くる" || word == "来る" {
        return VerbGroup::Irregular;
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return VerbGroup::Godan;
    }

    let last_char = chars[chars.len() - 1];

    if last_char == 'る' && chars.len() >= 2 {
        let second_last = chars[chars.len() - 2];
        if matches!(second_last, 'い' | 'え') {
            return VerbGroup::Ichidan;
        }
    }

    VerbGroup::Godan
}

pub fn to_te_form(word: &str) -> String {
    if word == "する" {
        return "して".to_string();
    }
    if word == "くる" || word == "来る" {
        return "きて".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    let last_char = chars[chars.len() - 1];

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push('て');
        return result;
    }

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

pub fn to_main_view(word: &str) -> String {
    if word == "する" {
        return "し".to_string();
    }
    if word == "くる" || word == "来る" {
        return "き".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        return result;
    }

    to_stem_form_godan(word)
}

fn to_stem_form_godan(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    let last_char = chars[chars.len() - 1];
    let godan_to_stem = [
        ('う', "い"),
        ('く', "き"),
        ('ぐ', "ぎ"),
        ('す', "し"),
        ('ず', "じ"),
        ('つ', "ち"),
        ('づ', "ぢ"),
        ('ぬ', "に"),
        ('ふ', "ひ"),
        ('ぶ', "び"),
        ('ぷ', "ぴ"),
        ('む', "み"),
        ('る', "り"),
    ];

    for (from, to) in godan_to_stem {
        if last_char == from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
}

pub fn to_stem_form(word: &str) -> String {
    to_main_view(word)
}

pub fn to_masu_form(word: &str) -> String {
    format!("{}ます", to_main_view(word))
}

pub fn to_masen_form(word: &str) -> String {
    format!("{}ません", to_main_view(word))
}

pub fn to_mashita_form(word: &str) -> String {
    format!("{}ました", to_main_view(word))
}

pub fn to_masen_deshita_form(word: &str) -> String {
    format!("{}ませんでした", to_main_view(word))
}

pub fn to_mashou_form(word: &str) -> String {
    format!("{}ましょう", to_main_view(word))
}

pub fn to_ta_form(word: &str) -> String {
    if word == "する" {
        return "した".to_string();
    }
    if word == "くる" || word == "来る" {
        return "きた".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    let last_char = chars[chars.len() - 1];

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push('た');
        return result;
    }

    match last_char {
        'く' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("いた");
            result
        }
        'ぐ' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("いだ");
            result
        }
        'す' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("した");
            result
        }
        'つ' | 'る' | 'う' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("った");
            result
        }
        'ぬ' | 'ぶ' | 'む' => {
            let mut result = word.to_string();
            result.pop();
            result.push_str("んだ");
            result
        }
        _ => word.to_string(),
    }
}

pub fn to_nai_form(word: &str) -> String {
    if word == "する" {
        return "しない".to_string();
    }
    if word == "くる" || word == "来る" {
        return "こない".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push_str("ない");
        return result;
    }

    let last_char = chars[chars.len() - 1];
    let godan_to_nai = [
        ('う', "わない"),
        ('く', "かない"),
        ('ぐ', "がない"),
        ('す', "さない"),
        ('ず', "ざない"),
        ('つ', "たない"),
        ('づ', "だない"),
        ('ぬ', "なない"),
        ('ふ', "はない"),
        ('ぶ', "ばない"),
        ('ぷ', "ぱない"),
        ('む', "まない"),
        ('る', "らない"),
    ];

    for (from, to) in godan_to_nai {
        if last_char == from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
}

pub fn to_dictionary_form(word: &str) -> String {
    word.to_string()
}

pub fn to_tara_form(word: &str) -> String {
    if word == "する" {
        return "したら".to_string();
    }
    if word == "くる" || word == "来る" {
        return "きたら".to_string();
    }

    format!("{}ら", to_ta_form(word))
}

pub fn to_ba_form(word: &str) -> String {
    if word == "する" {
        return "すれば".to_string();
    }
    if word == "くる" || word == "来る" {
        return "くれば".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push_str("れば");
        return result;
    }

    let last_char = chars[chars.len() - 1];
    let godan_to_ba = [
        ('う', "えば"),
        ('く', "けば"),
        ('ぐ', "げば"),
        ('す', "せば"),
        ('ず', "ぜば"),
        ('つ', "てば"),
        ('づ', "でば"),
        ('ぬ', "ねば"),
        ('ふ', "へば"),
        ('ぶ', "べば"),
        ('ぷ', "ぺば"),
        ('む', "めば"),
        ('る', "れば"),
    ];

    for (from, to) in godan_to_ba {
        if last_char == from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
}

pub fn to_potential_form(word: &str) -> String {
    if word == "する" {
        return "できる".to_string();
    }
    if word == "くる" || word == "来る" {
        return "こられる".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push_str("られる");
        return result;
    }

    let last_char = chars[chars.len() - 1];
    let godan_to_potential = [
        ('う', "える"),
        ('く', "ける"),
        ('ぐ', "げる"),
        ('す', "せる"),
        ('ず', "ぜる"),
        ('つ', "てる"),
        ('づ', "でる"),
        ('ぬ', "ねる"),
        ('ふ', "へる"),
        ('ぶ', "べる"),
        ('ぷ', "ぺる"),
        ('む', "める"),
        ('る', "れる"),
    ];

    for (from, to) in godan_to_potential {
        if last_char == from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
}

pub fn to_passive_form(word: &str) -> String {
    if word == "する" {
        return "される".to_string();
    }
    if word == "くる" || word == "来る" {
        return "こられる".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push_str("られる");
        return result;
    }

    let last_char = chars[chars.len() - 1];
    let godan_to_passive = [
        ('う', "われる"),
        ('く', "かれる"),
        ('ぐ', "がれる"),
        ('す', "される"),
        ('ず', "ざれる"),
        ('つ', "たれる"),
        ('づ', "だれる"),
        ('ぬ', "なれる"),
        ('ふ', "はれる"),
        ('ぶ', "ばれる"),
        ('ぷ', "ぱれる"),
        ('む', "まれる"),
        ('る', "られる"),
    ];

    for (from, to) in godan_to_passive {
        if last_char == from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
}

pub fn to_causative_form(word: &str) -> String {
    if word == "する" {
        return "させる".to_string();
    }
    if word == "くる" || word == "来る" {
        return "こさせる".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push_str("させる");
        return result;
    }

    let last_char = chars[chars.len() - 1];
    let godan_to_causative = [
        ('う', "わせる"),
        ('く', "かせる"),
        ('ぐ', "がせる"),
        ('す', "させる"),
        ('ず', "ざせる"),
        ('つ', "たせる"),
        ('づ', "だせる"),
        ('ぬ', "なせる"),
        ('ふ', "はせる"),
        ('ぶ', "ばせる"),
        ('ぷ', "ぱせる"),
        ('む', "ませる"),
        ('る', "らせる"),
    ];

    for (from, to) in godan_to_causative {
        if last_char == from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
}

pub fn to_causative_passive_form(word: &str) -> String {
    if word == "する" {
        return "させられる".to_string();
    }
    if word == "くる" || word == "来る" {
        return "こさせられる".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push_str("させられる");
        return result;
    }

    let last_char = chars[chars.len() - 1];
    let godan_to_caupass = [
        ('う', "わされる"),
        ('く', "かされる"),
        ('ぐ', "がされる"),
        ('す', "させられる"),
        ('ず', "ざされる"),
        ('つ', "たされる"),
        ('づ', "だされる"),
        ('ぬ', "なされる"),
        ('ふ', "はされる"),
        ('ぶ', "ばされる"),
        ('ぷ', "ぱされる"),
        ('む', "まされる"),
        ('る', "らされる"),
    ];

    for (from, to) in godan_to_caupass {
        if last_char == from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
}

pub fn to_imperative_form(word: &str) -> String {
    if word == "する" {
        return "しろ".to_string();
    }
    if word == "くる" || word == "来る" {
        return "こい".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push_str("ろ");
        return result;
    }

    let last_char = chars[chars.len() - 1];
    let godan_to_imperative = [
        ('う', "え"),
        ('く', "け"),
        ('ぐ', "げ"),
        ('す', "せ"),
        ('ず', "ぜ"),
        ('つ', "て"),
        ('づ', "で"),
        ('ぬ', "ね"),
        ('ふ', "へ"),
        ('ぶ', "べ"),
        ('ぷ', "ぺ"),
        ('む', "め"),
        ('る', "れ"),
    ];

    for (from, to) in godan_to_imperative {
        if last_char == from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
}

pub fn to_volitional_form(word: &str) -> String {
    if word == "する" {
        return "しよう".to_string();
    }
    if word == "くる" || word == "来る" {
        return "こよう".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push_str("よう");
        return result;
    }

    let last_char = chars[chars.len() - 1];
    let godan_to_volitional = [
        ('う', "おう"),
        ('く', "こう"),
        ('ぐ', "ごう"),
        ('す', "そう"),
        ('ず', "ぞう"),
        ('つ', "とう"),
        ('づ', "どう"),
        ('ぬ', "のう"),
        ('ふ', "ほう"),
        ('ぶ', "ぼう"),
        ('ぷ', "ぽう"),
        ('む', "もう"),
        ('る', "ろう"),
    ];

    for (from, to) in godan_to_volitional {
        if last_char == from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
}

pub fn to_sou_form_verb(word: &str) -> String {
    format!("{}そう", to_main_view(word))
}

pub fn to_zu_form(word: &str) -> String {
    if word == "する" {
        return "せず".to_string();
    }
    if word == "くる" || word == "来る" {
        return "こず".to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push_str("ず");
        return result;
    }

    let last_char = chars[chars.len() - 1];
    let godan_to_zu = [
        ('う', "わず"),
        ('く', "かず"),
        ('ぐ', "がず"),
        ('す', "さず"),
        ('ず', "ざず"),
        ('つ', "たず"),
        ('づ', "だず"),
        ('ぬ', "なず"),
        ('ふ', "はず"),
        ('ぶ', "ばず"),
        ('ぷ', "ぱず"),
        ('む', "まず"),
        ('る', "らず"),
    ];

    for (from, to) in godan_to_zu {
        if last_char == from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
}

pub fn to_tai_form(word: &str) -> String {
    format!("{}たい", to_main_view(word))
}

pub fn to_yasui_form(word: &str) -> String {
    format!("{}やすい", to_main_view(word))
}

pub fn to_nikui_form(word: &str) -> String {
    format!("{}にくい", to_main_view(word))
}

pub fn to_sugiru_form_verb(word: &str) -> String {
    format!("{}すぎる", to_main_view(word))
}

pub fn to_chau_form(word: &str) -> String {
    let te = to_te_form(word);
    if te.ends_with("て") {
        format!("{}ちゃう", &te[..te.len() - 1])
    } else if te.ends_with("で") {
        format!("{}じゃう", &te[..te.len() - 1])
    } else {
        format!("{}ちゃう", te)
    }
}

pub fn to_toku_form(word: &str) -> String {
    let te = to_te_form(word);
    if te.ends_with("て") {
        format!("{}とく", &te[..te.len() - 1])
    } else if te.ends_with("で") {
        format!("{}どく", &te[..te.len() - 1])
    } else {
        format!("{}とく", te)
    }
}

pub fn to_teru_form(word: &str) -> String {
    let te = to_te_form(word);
    if te.ends_with("て") {
        format!("{}てる", &te[..te.len() - 1])
    } else if te.ends_with("で") {
        format!("{}でる", &te[..te.len() - 1])
    } else {
        format!("{}てる", te)
    }
}

pub fn to_o_ni_narimasu_form(word: &str) -> String {
    if word == "する" {
        return "なさる".to_string();
    }
    if word == "くる" || word == "来る" {
        return "いらっしゃる".to_string();
    }
    format!("お{}になる", to_main_view(word))
}

pub fn to_o_kudasai_form(word: &str) -> String {
    if word == "する" {
        return "なさってください".to_string();
    }
    if word == "くる" || word == "来る" {
        return "いらっしゃってください".to_string();
    }
    format!("お{}ください", to_main_view(word))
}

pub fn to_o_shimasu_form(word: &str) -> String {
    if word == "する" {
        return "いたす".to_string();
    }
    if word == "くる" || word == "来る" {
        return "参る".to_string();
    }
    format!("お{}する", to_main_view(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_verb() {
        assert_eq!(classify_verb("する"), VerbGroup::Irregular);
        assert_eq!(classify_verb("くる"), VerbGroup::Irregular);
        assert_eq!(classify_verb("食べる"), VerbGroup::Ichidan);
        assert_eq!(classify_verb("見る"), VerbGroup::Ichidan);
        assert_eq!(classify_verb("行く"), VerbGroup::Godan);
        assert_eq!(classify_verb("話す"), VerbGroup::Godan);
    }

    #[test]
    fn test_te_form() {
        assert_eq!(to_te_form("行く"), "行って");
        assert_eq!(to_te_form("話す"), "話して");
        assert_eq!(to_te_form("読む"), "読んで");
        assert_eq!(to_te_form("書く"), "書いて");
        assert_eq!(to_te_form("泳ぐ"), "泳いで");
        assert_eq!(to_te_form("食べる"), "食べて");
        assert_eq!(to_te_form("見る"), "見て");
        assert_eq!(to_te_form("する"), "して");
        assert_eq!(to_te_form("くる"), "きて");
    }

    #[test]
    fn test_masu_form() {
        assert_eq!(to_masu_form("行く"), "行きます");
        assert_eq!(to_masu_form("食べる"), "食べます");
        assert_eq!(to_masu_form("する"), "します");
    }

    #[test]
    fn test_ta_form() {
        assert_eq!(to_ta_form("行く"), "行った");
        assert_eq!(to_ta_form("話す"), "話した");
        assert_eq!(to_ta_form("読む"), "読んだ");
        assert_eq!(to_ta_form("書く"), "書いた");
        assert_eq!(to_ta_form("食べる"), "食べた");
        assert_eq!(to_ta_form("する"), "した");
    }

    #[test]
    fn test_nai_form() {
        assert_eq!(to_nai_form("行く"), "行かない");
        assert_eq!(to_nai_form("食べる"), "食べない");
        assert_eq!(to_nai_form("する"), "しない");
        assert_eq!(to_nai_form("くる"), "こない");
    }

    #[test]
    fn test_tara_form() {
        assert_eq!(to_tara_form("行く"), "行ったら");
        assert_eq!(to_tara_form("食べる"), "食べたら");
        assert_eq!(to_tara_form("する"), "したら");
    }

    #[test]
    fn test_ba_form() {
        assert_eq!(to_ba_form("行く"), "行けば");
        assert_eq!(to_ba_form("食べる"), "食べれば");
        assert_eq!(to_ba_form("する"), "すれば");
    }

    #[test]
    fn test_potential_form() {
        assert_eq!(to_potential_form("行く"), "行ける");
        assert_eq!(to_potential_form("食べる"), "食べられる");
        assert_eq!(to_potential_form("する"), "できる");
    }

    #[test]
    fn test_passive_form() {
        assert_eq!(to_passive_form("行く"), "行かれる");
        assert_eq!(to_passive_form("食べる"), "食べられる");
        assert_eq!(to_passive_form("する"), "される");
    }

    #[test]
    fn test_causative_form() {
        assert_eq!(to_causative_form("行く"), "行かせる");
        assert_eq!(to_causative_form("食べる"), "食べさせる");
        assert_eq!(to_causative_form("する"), "させる");
    }

    #[test]
    fn test_imperative_form() {
        assert_eq!(to_imperative_form("行く"), "行け");
        assert_eq!(to_imperative_form("食べる"), "食べろ");
        assert_eq!(to_imperative_form("する"), "しろ");
    }

    #[test]
    fn test_volitional_form() {
        assert_eq!(to_volitional_form("行く"), "行こう");
        assert_eq!(to_volitional_form("食べる"), "食べよう");
        assert_eq!(to_volitional_form("する"), "しよう");
    }

    #[test]
    fn test_zu_form() {
        assert_eq!(to_zu_form("行く"), "行かず");
        assert_eq!(to_zu_form("食べる"), "食べず");
        assert_eq!(to_zu_form("する"), "せず");
    }
}
