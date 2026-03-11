use crate::domain::{Card, NativeLanguage, User, get_translation};

/*
 * Returns a tuple of (is_known, meaning)
 */
pub fn is_word_known(user: &User, word: &str, lang: &NativeLanguage) -> (bool, Option<String>) {
    let meaning = get_translation(word, lang);

    for study_card in user.knowledge_set().study_cards().values() {
        if let Card::Vocabulary(vocab_card) = study_card.card()
            && vocab_card.word().text() == word
        {
            return (true, meaning);
        }
    }

    (false, meaning)
}
