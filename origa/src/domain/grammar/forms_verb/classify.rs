#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbGroup {
    Ichidan,
    Godan,
    Irregular,
}

pub(super) const ICHIDAN_SHORT_VERBS: &[&str] = &[
    "見る", "居る", "着る", "似る", "煮る", "射る", "鋳る", "寝る", "経る", "蹴る", "乾る",
];

pub fn classify_verb(word: &str) -> VerbGroup {
    if word == "する" || word == "くる" || word == "来る" {
        return VerbGroup::Irregular;
    }

    if ICHIDAN_SHORT_VERBS.contains(&word) {
        return VerbGroup::Ichidan;
    }

    if super::godan_tables::GODAN_IRU_ERU_VERBS.contains(&word) {
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
