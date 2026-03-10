use crate::domain::{Card, NativeLanguage, User};

pub fn is_word_known(user: &User, word: &str, lang: &NativeLanguage) -> (bool, Option<String>) {
    for study_card in user.knowledge_set().study_cards().values() {
        if let Card::Vocabulary(vocab_card) = study_card.card()
            && vocab_card.word().text() == word
        {
            let meaning = vocab_card.answer(lang).ok().map(|a| a.text().to_string());
            return (true, meaning);
        }
    }
    (false, None)
}
