use crate::domain::{Card, NativeLanguage, User};

pub fn is_word_known(user: &User, word: &str, lang: &NativeLanguage) -> (bool, Option<String>) {
    for study_card in user.knowledge_set().study_cards().values() {
        if let Card::Vocabulary(vocab_card) = study_card.card()
            && vocab_card.word().text() == word
        {
            return (true, Some(vocab_card.answer(lang).text().to_string()));
        }
    }
    (false, None)
}
