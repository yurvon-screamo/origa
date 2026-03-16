#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbGroup {
    Ichidan,
    Godan,
    Irregular,
}

const ICHIDAN_SHORT_VERBS: &[&str] = &[
    "見る", "居る", "着る", "似る", "煮る", "射る", "鋳る", "寝る", "経る", "蹴る", "乾る",
];

const GODAN_IRU_ERU_VERBS: &[&str] = &[
    "要る", "入る", "減る", "茂る", "耽る", "喋る", "遮る", "罵る", "悟る",
];

pub fn classify_verb(word: &str) -> VerbGroup {
    if word == "する" || word == "くる" || word == "来る" {
        return VerbGroup::Irregular;
    }

    if ICHIDAN_SHORT_VERBS.contains(&word) {
        return VerbGroup::Ichidan;
    }

    if GODAN_IRU_ERU_VERBS.contains(&word) {
        return VerbGroup::Godan;
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return VerbGroup::Godan;
    }

    let last_char = chars[chars.len() - 1];

    if last_char == 'る' && chars.len() >= 2 {
        let second_last = chars[chars.len() - 2];
        let i_row = [
            'い', 'き', 'ぎ', 'し', 'じ', 'ち', 'ぢ', 'に', 'ひ', 'び', 'ぴ', 'み', 'り',
        ];
        let e_row = [
            'え', 'け', 'げ', 'せ', 'ぜ', 'て', 'で', 'ね', 'へ', 'べ', 'ぺ', 'め', 'れ',
        ];
        if i_row.contains(&second_last) || e_row.contains(&second_last) {
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
    if word == "行く" {
        return "行って".to_string();
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
    if word == "行く" {
        return "行った".to_string();
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
        result.push('ろ');
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
        result.push('ず');
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
        let mut stem = te;
        stem.pop();
        format!("{}ちゃう", stem)
    } else if te.ends_with("で") {
        let mut stem = te;
        stem.pop();
        format!("{}じゃう", stem)
    } else {
        format!("{}ちゃう", te)
    }
}

pub fn to_toku_form(word: &str) -> String {
    let te = to_te_form(word);
    if te.ends_with("て") {
        let mut stem = te;
        stem.pop();
        format!("{}とく", stem)
    } else if te.ends_with("で") {
        let mut stem = te;
        stem.pop();
        format!("{}どく", stem)
    } else {
        format!("{}とく", te)
    }
}

pub fn to_teru_form(word: &str) -> String {
    let te = to_te_form(word);
    if te.ends_with("て") {
        let mut stem = te;
        stem.pop();
        format!("{}てる", stem)
    } else if te.ends_with("で") {
        let mut stem = te;
        stem.pop();
        format!("{}でる", stem)
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
    use rstest::rstest;

    #[rstest]
    #[case("する", VerbGroup::Irregular)]
    #[case("くる", VerbGroup::Irregular)]
    #[case("食べる", VerbGroup::Ichidan)]
    #[case("見る", VerbGroup::Ichidan)]
    #[case("行く", VerbGroup::Godan)]
    #[case("話す", VerbGroup::Godan)]
    fn classify_verb(#[case] input: &str, #[case] expected: VerbGroup) {
        assert_eq!(super::classify_verb(input), expected);
    }

    #[rstest]
    #[case("行く", "行って")]
    #[case("話す", "話して")]
    #[case("読む", "読んで")]
    #[case("書く", "書いて")]
    #[case("泳ぐ", "泳いで")]
    #[case("食べる", "食べて")]
    #[case("見る", "見て")]
    #[case("する", "して")]
    #[case("くる", "きて")]
    fn te_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_te_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行きます")]
    #[case("食べる", "食べます")]
    #[case("する", "します")]
    fn masu_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_masu_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行った")]
    #[case("話す", "話した")]
    #[case("読む", "読んだ")]
    #[case("書く", "書いた")]
    #[case("食べる", "食べた")]
    #[case("する", "した")]
    fn ta_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_ta_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行かない")]
    #[case("食べる", "食べない")]
    #[case("する", "しない")]
    #[case("くる", "こない")]
    fn nai_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_nai_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行ったら")]
    #[case("食べる", "食べたら")]
    #[case("する", "したら")]
    fn tara_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_tara_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行けば")]
    #[case("食べる", "食べれば")]
    #[case("する", "すれば")]
    fn ba_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_ba_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行ける")]
    #[case("食べる", "食べられる")]
    #[case("する", "できる")]
    fn potential_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_potential_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行かれる")]
    #[case("食べる", "食べられる")]
    #[case("する", "される")]
    fn passive_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_passive_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行かせる")]
    #[case("食べる", "食べさせる")]
    #[case("する", "させる")]
    fn causative_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_causative_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行け")]
    #[case("食べる", "食べろ")]
    #[case("する", "しろ")]
    fn imperative_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_imperative_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行こう")]
    #[case("食べる", "食べよう")]
    #[case("する", "しよう")]
    fn volitional_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_volitional_form(input), expected);
    }

    #[rstest]
    #[case("行く", "行かず")]
    #[case("食べる", "食べず")]
    #[case("する", "せず")]
    fn zu_form(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(to_zu_form(input), expected);
    }

    #[rstest]
    #[case("要る", VerbGroup::Godan)]
    #[case("入る", VerbGroup::Godan)]
    #[case("減る", VerbGroup::Godan)]
    #[case("茂る", VerbGroup::Godan)]
    #[case("喋る", VerbGroup::Godan)]
    #[case("遮る", VerbGroup::Godan)]
    #[case("悟る", VerbGroup::Godan)]
    fn godan_iru_eru_exceptions(#[case] input: &str, #[case] expected: VerbGroup) {
        assert_eq!(super::classify_verb(input), expected);
    }

    #[rstest]
    #[case("見る", VerbGroup::Ichidan)]
    #[case("居る", VerbGroup::Ichidan)]
    #[case("着る", VerbGroup::Ichidan)]
    #[case("寝る", VerbGroup::Ichidan)]
    #[case("経る", VerbGroup::Ichidan)]
    #[case("蹴る", VerbGroup::Ichidan)]
    fn ichidan_short_verbs(#[case] input: &str, #[case] expected: VerbGroup) {
        assert_eq!(super::classify_verb(input), expected);
    }

    #[rstest]
    #[case("要る", "要って", "要ります")]
    #[case("入る", "入って", "入ります")]
    #[case("喋る", "喋って", "喋ります")]
    fn godan_exceptions_conjugation(#[case] verb: &str, #[case] te: &str, #[case] masu: &str) {
        assert_eq!(to_te_form(verb), te);
        assert_eq!(to_masu_form(verb), masu);
    }
}
