use crate::domain::{KeikakuError, furigana::furiganize_text};

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

    fn has_furigana(&self) -> Result<bool, KeikakuError>;
    fn as_furigana(&self) -> Result<String, KeikakuError>;
    fn equals_by_reading(&self, other: &Self) -> Result<bool, KeikakuError>;
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

    fn as_furigana(&self) -> Result<String, KeikakuError> {
        furiganize_text(self)
    }

    fn has_furigana(&self) -> Result<bool, KeikakuError> {
        self.as_furigana().map(|furigana| furigana != self)
    }

    fn equals_by_reading(&self, other: &Self) -> Result<bool, KeikakuError> {
        let left = self.as_furigana()?;
        let right = other.as_furigana()?;
        Ok(left == right)
    }

    fn contains_kanji(&self) -> bool {
        self.chars().any(|c| c.is_kanji())
    }
}
