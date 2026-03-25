use super::godan_tables::{
    GODAN_TO_BA, GODAN_TO_CAUSATIVE, GODAN_TO_CAUSATIVE_PASSIVE, GODAN_TO_IMPERATIVE, GODAN_TO_NAI,
    GODAN_TO_PASSIVE, GODAN_TO_POTENTIAL, GODAN_TO_VOLITIONAL, GODAN_TO_ZU,
};
use super::irregulars::{
    get_irregular_form, is_ichidan, stem_from_godan, te_form_stem, BA_IRREGULAR,
    CAUSATIVE_IRREGULAR, CAUSATIVE_PASSIVE_IRREGULAR, IMPERATIVE_IRREGULAR, MAIN_VIEW_IRREGULAR,
    NAI_IRREGULAR, O_KUDASAI_IRREGULAR, O_NI_NARIMASU_IRREGULAR, O_SHIMASU_IRREGULAR,
    PASSIVE_IRREGULAR, POTENTIAL_IRREGULAR, VOLITIONAL_IRREGULAR, ZU_IRREGULAR,
};

fn main_view_stem(word: &str) -> String {
    if let Some(result) = get_irregular_form(word, MAIN_VIEW_IRREGULAR) {
        return result;
    }
    if is_ichidan(word) {
        return word.strip_suffix('る').unwrap_or(word).to_string();
    }
    stem_from_godan(word).unwrap_or_else(|| word.to_string())
}

fn apply_conjugation(
    word: &str,
    irregular: super::irregulars::IrregularMapping,
    ichidan_suffix: &str,
    godan_table: &[(char, &str)],
) -> String {
    if let Some(result) = get_irregular_form(word, irregular) {
        return result;
    }
    if is_ichidan(word) {
        let mut result = word.to_string();
        result.pop();
        result.push_str(ichidan_suffix);
        return result;
    }
    let chars: Vec<char> = word.chars().collect();
    let last_char = chars.last().copied().unwrap_or(' ');
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

pub fn to_nai_form(word: &str) -> String {
    apply_conjugation(word, NAI_IRREGULAR, "ない", GODAN_TO_NAI)
}

pub fn to_ba_form(word: &str) -> String {
    apply_conjugation(word, BA_IRREGULAR, "れば", GODAN_TO_BA)
}

pub fn to_potential_form(word: &str) -> String {
    apply_conjugation(word, POTENTIAL_IRREGULAR, "られる", GODAN_TO_POTENTIAL)
}

pub fn to_passive_form(word: &str) -> String {
    apply_conjugation(word, PASSIVE_IRREGULAR, "られる", GODAN_TO_PASSIVE)
}

pub fn to_causative_form(word: &str) -> String {
    apply_conjugation(word, CAUSATIVE_IRREGULAR, "させる", GODAN_TO_CAUSATIVE)
}

pub fn to_causative_passive_form(word: &str) -> String {
    apply_conjugation(
        word,
        CAUSATIVE_PASSIVE_IRREGULAR,
        "させられる",
        GODAN_TO_CAUSATIVE_PASSIVE,
    )
}

pub fn to_imperative_form(word: &str) -> String {
    apply_conjugation(word, IMPERATIVE_IRREGULAR, "ろ", GODAN_TO_IMPERATIVE)
}

pub fn to_volitional_form(word: &str) -> String {
    apply_conjugation(word, VOLITIONAL_IRREGULAR, "よう", GODAN_TO_VOLITIONAL)
}

pub fn to_zu_form(word: &str) -> String {
    apply_conjugation(word, ZU_IRREGULAR, "ず", GODAN_TO_ZU)
}

fn replace_te_ending(word: &str, new_suffix: &str) -> String {
    format!("{}{}", te_form_stem(word), new_suffix)
}

pub fn to_chau_form(word: &str) -> String {
    replace_te_ending(word, "ちゃう")
}

pub fn to_toku_form(word: &str) -> String {
    replace_te_ending(word, "とく")
}

pub fn to_teru_form(word: &str) -> String {
    replace_te_ending(word, "てる")
}

pub fn to_o_ni_narimasu_form(word: &str) -> String {
    if let Some(result) = get_irregular_form(word, O_NI_NARIMASU_IRREGULAR) {
        return result;
    }
    format!("お{}になる", main_view_stem(word))
}

pub fn to_o_kudasai_form(word: &str) -> String {
    if let Some(result) = get_irregular_form(word, O_KUDASAI_IRREGULAR) {
        return result;
    }
    format!("お{}ください", main_view_stem(word))
}

pub fn to_o_shimasu_form(word: &str) -> String {
    if let Some(result) = get_irregular_form(word, O_SHIMASU_IRREGULAR) {
        return result;
    }
    format!("お{}する", main_view_stem(word))
}

pub fn to_tara_form(word: &str) -> String {
    if let Some(result) = get_irregular_form(word, super::irregulars::TARA_IRREGULAR) {
        return result;
    }
    format!("{}ら", super::te_ta::to_ta_form(word))
}

pub fn to_main_view(word: &str) -> String {
    main_view_stem(word)
}

pub fn to_stem_form(word: &str) -> String {
    main_view_stem(word)
}

pub fn to_masu_form(word: &str) -> String {
    format!("{}ます", main_view_stem(word))
}

pub fn to_masen_form(word: &str) -> String {
    format!("{}ません", main_view_stem(word))
}

pub fn to_mashita_form(word: &str) -> String {
    format!("{}ました", main_view_stem(word))
}

pub fn to_masen_deshita_form(word: &str) -> String {
    format!("{}ませんでした", main_view_stem(word))
}

pub fn to_mashou_form(word: &str) -> String {
    format!("{}ましょう", main_view_stem(word))
}

pub fn to_sou_form_verb(word: &str) -> String {
    format!("{}そう", main_view_stem(word))
}

pub fn to_tai_form(word: &str) -> String {
    format!("{}たい", main_view_stem(word))
}

pub fn to_yasui_form(word: &str) -> String {
    format!("{}やすい", main_view_stem(word))
}

pub fn to_nikui_form(word: &str) -> String {
    format!("{}にくい", main_view_stem(word))
}

pub fn to_sugiru_form_verb(word: &str) -> String {
    format!("{}すぎる", main_view_stem(word))
}
