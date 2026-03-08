use std::sync::Once;

use crate::domain::{
    init_kanji_dictionary, init_vocabulary_dictionary, Answer, Card, KanjiCard, KanjiData,
    NativeLanguage, Question, User, VocabularyCard, VocabularyChunkData,
};

static INIT: Once = Once::new();

pub fn init_test_dictionaries() {
    INIT.call_once(|| {
        let vocabulary_data = VocabularyChunkData {
            chunk_01: r#"{"猫": {"russian_translation": "кошка", "english_translation": "cat"}, "犬": {"russian_translation": "собака", "english_translation": "dog"}, "人": {"russian_translation": "человек", "english_translation": "person"}, "食べる": {"russian_translation": "есть", "english_translation": "to eat"}, "行く": {"russian_translation": "идти", "english_translation": "to go"}, "あさって": {"russian_translation": "послезавтра", "english_translation": "day after tomorrow"}, "こんな": {"russian_translation": "такой, подобный", "english_translation": "such, this kind of"}, "そう": {"russian_translation": "так, таким образом", "english_translation": "so, thus"}, "そして": {"russian_translation": "и затем", "english_translation": "and then"}, "で": {"russian_translation": "при помощи, с помощью", "english_translation": "by means of"}}"#.to_string(),
            chunk_02: "{}".to_string(),
            chunk_03: "{}".to_string(),
            chunk_04: "{}".to_string(),
            chunk_05: "{}".to_string(),
            chunk_06: "{}".to_string(),
            chunk_07: "{}".to_string(),
            chunk_08: "{}".to_string(),
            chunk_09: "{}".to_string(),
            chunk_10: "{}".to_string(),
            chunk_11: "{}".to_string(),
        };

        let kanji_data = KanjiData {
            kanji_json: r#"{"kanji": [{"kanji": "人", "jlpt": "N5", "used_in": 2357, "description": "человек", "radicals": ["人"], "popular_words": ["人", "人々", "人間"], "on_readings": ["ジン", "ニン"], "kun_readings": ["ひと", "り", "と"]}, {"kanji": "一", "jlpt": "N5", "used_in": 2077, "description": "один", "radicals": ["一"], "popular_words": ["一", "一つ", "一番"], "on_readings": ["イチ", "イツ"], "kun_readings": ["ひと", "ひとつ"]}, {"kanji": "日", "jlpt": "N5", "used_in": 1500, "description": "день, солнце", "radicals": ["日"], "popular_words": ["日", "日本", "日曜日"], "on_readings": ["ニチ", "ジツ"], "kun_readings": ["ひ", "か"]}]}"#.to_string(),
        };

        init_vocabulary_dictionary(vocabulary_data).expect("Failed to init vocabulary");
        init_kanji_dictionary(kanji_data).expect("Failed to init kanji");
    });
}

pub fn create_test_vocab_card(word: &str, meaning: &str) -> Card {
    Card::Vocabulary(VocabularyCard::new(
        Question::new(word.to_string()).unwrap(),
        Answer::new(meaning.to_string()).unwrap(),
    ))
}

pub fn create_test_kanji_card(kanji: &str) -> Card {
    Card::Kanji(KanjiCard::new(kanji.to_string(), &NativeLanguage::Russian).unwrap())
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
        user.create_card(card).unwrap();
    }

    user
}
