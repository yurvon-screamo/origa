use super::classify::{VerbGroup, classify_verb};
use super::godan_tables::TE_TA_MAPPING;

pub fn to_te_form(word: &str) -> String {
    apply_te_ta(word, true)
}

pub fn to_ta_form(word: &str) -> String {
    apply_te_ta(word, false)
}

fn apply_te_ta(word: &str, use_te: bool) -> String {
    let (suru_result, kuru_result, iku_result) = if use_te {
        ("して", "きて", "行って")
    } else {
        ("した", "きた", "行った")
    };

    if word == "する" {
        return suru_result.to_string();
    }
    if word == "くる" || word == "来る" {
        return kuru_result.to_string();
    }
    if word == "行く" {
        return iku_result.to_string();
    }

    let chars: Vec<char> = word.chars().collect();
    if chars.is_empty() {
        return word.to_string();
    }

    let last_char = chars[chars.len() - 1];
    let ichidan_suffix = if use_te { 'て' } else { 'た' };

    if classify_verb(word) == VerbGroup::Ichidan {
        let mut result = word.to_string();
        result.pop();
        result.push(ichidan_suffix);
        return result;
    }

    for (from, te_suffix, ta_suffix) in TE_TA_MAPPING {
        if last_char == *from {
            let mut result = word.to_string();
            result.pop();
            result.push_str(if use_te { te_suffix } else { ta_suffix });
            return result;
        }
    }

    word.to_string()
}
