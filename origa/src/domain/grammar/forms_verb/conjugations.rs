use super::classify::{VerbGroup, classify_verb};
use super::godan_tables::{
    GODAN_TO_BA, GODAN_TO_CAUSATIVE, GODAN_TO_CAUSATIVE_PASSIVE, GODAN_TO_IMPERATIVE, GODAN_TO_NAI,
    GODAN_TO_PASSIVE, GODAN_TO_POTENTIAL, GODAN_TO_STEM, GODAN_TO_VOLITIONAL, GODAN_TO_ZU,
};
use super::te_ta::{to_ta_form, to_te_form};

fn apply_godan_conjugation<F>(
    word: &str,
    irregulars: F,
    ichidan_suffix: &str,
    godan_table: &[(char, &str)],
) -> String
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(result) = irregulars(word) {
        return result;
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push_str(ichidan_suffix);
        return result;
    }

    let last_char = chars[chars.len() - 1];
    for (from, to) in godan_table {
        if last_char == *from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
}

fn to_stem_form_godan(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    let last_char = chars[chars.len() - 1];
    for (from, to) in GODAN_TO_STEM {
        if last_char == *from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(to);
            return result;
        }
    }

    word.to_string()
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

pub fn to_nai_form(word: &str) -> String {
    fn irregulars(w: &str) -> Option<String> {
        match w {
            "する" => Some("しない".to_string()),
            "くる" | "来る" => Some("こない".to_string()),
            _ => None,
        }
    }
    apply_godan_conjugation(word, irregulars, "ない", GODAN_TO_NAI)
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
    fn irregulars(w: &str) -> Option<String> {
        match w {
            "する" => Some("すれば".to_string()),
            "くる" | "来る" => Some("くれば".to_string()),
            _ => None,
        }
    }
    apply_godan_conjugation(word, irregulars, "れば", GODAN_TO_BA)
}

pub fn to_potential_form(word: &str) -> String {
    fn irregulars(w: &str) -> Option<String> {
        match w {
            "する" => Some("できる".to_string()),
            "くる" | "来る" => Some("こられる".to_string()),
            _ => None,
        }
    }
    apply_godan_conjugation(word, irregulars, "られる", GODAN_TO_POTENTIAL)
}

pub fn to_passive_form(word: &str) -> String {
    fn irregulars(w: &str) -> Option<String> {
        match w {
            "する" => Some("される".to_string()),
            "くる" | "来る" => Some("こられる".to_string()),
            _ => None,
        }
    }
    apply_godan_conjugation(word, irregulars, "られる", GODAN_TO_PASSIVE)
}

pub fn to_causative_form(word: &str) -> String {
    fn irregulars(w: &str) -> Option<String> {
        match w {
            "する" => Some("させる".to_string()),
            "くる" | "来る" => Some("こさせる".to_string()),
            _ => None,
        }
    }
    apply_godan_conjugation(word, irregulars, "させる", GODAN_TO_CAUSATIVE)
}

pub fn to_causative_passive_form(word: &str) -> String {
    fn irregulars(w: &str) -> Option<String> {
        match w {
            "する" => Some("させられる".to_string()),
            "くる" | "来る" => Some("こさせられる".to_string()),
            _ => None,
        }
    }
    apply_godan_conjugation(word, irregulars, "させられる", GODAN_TO_CAUSATIVE_PASSIVE)
}

pub fn to_imperative_form(word: &str) -> String {
    fn irregulars(w: &str) -> Option<String> {
        match w {
            "する" => Some("しろ".to_string()),
            "くる" | "来る" => Some("こい".to_string()),
            _ => None,
        }
    }
    apply_godan_conjugation(word, irregulars, "ろ", GODAN_TO_IMPERATIVE)
}

pub fn to_volitional_form(word: &str) -> String {
    fn irregulars(w: &str) -> Option<String> {
        match w {
            "する" => Some("しよう".to_string()),
            "くる" | "来る" => Some("こよう".to_string()),
            _ => None,
        }
    }
    apply_godan_conjugation(word, irregulars, "よう", GODAN_TO_VOLITIONAL)
}

pub fn to_sou_form_verb(word: &str) -> String {
    format!("{}そう", to_main_view(word))
}

pub fn to_zu_form(word: &str) -> String {
    fn irregulars(w: &str) -> Option<String> {
        match w {
            "する" => Some("せず".to_string()),
            "くる" | "来る" => Some("こず".to_string()),
            _ => None,
        }
    }
    apply_godan_conjugation(word, irregulars, "ず", GODAN_TO_ZU)
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
