mod file_well_known_set_loader;
mod in_memory_repository;

pub(crate) use crate::test_support::{
    get_cdn_dir, init_phrase_index_from_cdn, init_real_dictionaries,
};
pub use file_well_known_set_loader::MockCdnProvider;
pub use in_memory_repository::InMemoryUserRepository;

use crate::domain::{Card, NativeLanguage, Question, User, VocabularyCard};

pub fn create_test_vocab_card(word: &str) -> Card {
    Card::Vocabulary(VocabularyCard::new(
        Question::new(word.to_string()).expect("Failed to create Question"),
    ))
}

pub fn create_user_with_vocab_cards(count: usize) -> User {
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );

    for i in 0..count {
        let word = format!("word_{}", i);
        let card = create_test_vocab_card(&word);
        user.create_card(card).expect("Failed to create card");
    }

    user
}
