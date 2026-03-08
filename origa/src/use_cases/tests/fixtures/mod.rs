mod file_well_known_set_loader;
mod in_memory_repository;
mod real_dictionaries;

pub use file_well_known_set_loader::FileWellKnownSetLoader;
pub use in_memory_repository::InMemoryUserRepository;
pub use real_dictionaries::init_real_dictionaries;

use std::path::PathBuf;

use crate::domain::{Answer, Card, KanjiCard, NativeLanguage, Question, User, VocabularyCard};

pub fn get_public_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("Failed to get parent directory of CARGO_MANIFEST_DIR")
        .join("origa_ui")
        .join("public")
}

pub fn create_test_vocab_card(word: &str, _meaning: &str) -> Card {
    Card::Vocabulary(VocabularyCard::new(
        Question::new(word.to_string()).expect("Failed to create Question"),
    ))
}

pub fn create_test_kanji_card(kanji: &str) -> Card {
    Card::Kanji(KanjiCard::new(kanji.to_string()).expect("Failed to create KanjiCard"))
}

pub fn create_user_with_vocab_cards(count: usize) -> User {
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );

    for i in 0..count {
        let word = format!("word_{}", i);
        let meaning = format!("meaning_{}", i);
        let card = create_test_vocab_card(&word, &meaning);
        user.create_card(card).expect("Failed to create card");
    }

    user
}
