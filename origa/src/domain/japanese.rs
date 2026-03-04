pub trait JapaneseChar {
    fn is_japanese(&self) -> bool;
    fn is_hiragana(&self) -> bool;
    fn is_katakana(&self) -> bool;
    fn is_kanji(&self) -> bool;
}

pub trait JapaneseText {
    fn is_japanese(&self) -> bool;
    fn contains_japanese(&self) -> bool;
    fn contains_kanji(&self) -> bool;
}

impl JapaneseChar for char {
    fn is_japanese(&self) -> bool {
        self.is_hiragana() || self.is_katakana() || self.is_kanji()
    }

    fn is_hiragana(&self) -> bool {
        ('\u{3040}'..='\u{309F}').contains(self)
    }

    fn is_katakana(&self) -> bool {
        ('\u{30A0}'..='\u{30FF}').contains(self)
    }

    fn is_kanji(&self) -> bool {
        ('\u{4E00}'..='\u{9FFF}').contains(self)
            || ('\u{3400}'..='\u{4DBF}').contains(self)
            || ('\u{20000}'..='\u{2A6DF}').contains(self)
    }
}

impl JapaneseText for str {
    fn is_japanese(&self) -> bool {
        self.chars().all(|c| c.is_japanese())
    }

    fn contains_japanese(&self) -> bool {
        self.chars().any(|c| c.is_japanese())
    }

    fn contains_kanji(&self) -> bool {
        self.chars().any(|c| c.is_kanji())
    }
}

pub fn filter_japanese_text(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_japanese() || is_cjk_punctuation(c) {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn is_cjk_punctuation(c: char) -> bool {
    ('\u{3000}'..='\u{303F}').contains(&c)
}
