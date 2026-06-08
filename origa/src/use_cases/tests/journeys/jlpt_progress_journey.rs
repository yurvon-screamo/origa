use std::collections::HashSet;

use crate::dictionary::grammar::GRAMMAR_RULES;
use crate::dictionary::kanji::KANJI_DICTIONARY;
use crate::domain::{
    Card, JapaneseLevel, JlptContent, NativeLanguage, User, VocabularyCard, tokenize_text,
};
use crate::traits::UserRepository;
use crate::use_cases::MarkCardAsKnownUseCase;
use crate::use_cases::tests::fixtures::{
    InMemoryUserRepository, get_cdn_dir, init_real_dictionaries,
};

fn load_jlpt_n5_words() -> HashSet<String> {
    let cdn_dir = get_cdn_dir();
    let path = cdn_dir.join("well_known_set").join("jlpt_n5.json");
    let content = std::fs::read_to_string(&path).expect("Failed to read jlpt_n5.json");
    let data: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse jlpt_n5.json");
    data["words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn build_real_jlpt_content() -> JlptContent {
    let mut content = JlptContent::new();

    if let Some(db) = KANJI_DICTIONARY.get() {
        for level in JapaneseLevel::ALL {
            let kanji_list = db.get_kanji_list(&level);
            let set: HashSet<String> = kanji_list.iter().map(|k| k.kanji().to_string()).collect();
            if !set.is_empty() {
                content.kanji_by_level.insert(level, set);
            }
        }
    }

    if let Some(rules) = GRAMMAR_RULES.get() {
        for rule in rules.iter() {
            content
                .grammar_by_level
                .entry(*rule.level())
                .or_default()
                .insert(rule.rule_id().to_string());
        }
    }

    let n5_words = load_jlpt_n5_words();
    if !n5_words.is_empty() {
        content.words_by_level.insert(JapaneseLevel::N5, n5_words);
    }

    content
}

#[test]
fn token_base_forms_match_jlpt_n5_words() {
    init_real_dictionaries();

    let n5_words = load_jlpt_n5_words();
    let tokens = tokenize_text("私は本を読みます").expect("Tokenization failed");

    let vocab_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.part_of_speech().is_vocabulary_word())
        .collect();

    assert!(
        !vocab_tokens.is_empty(),
        "Should have at least one vocabulary token"
    );

    let any_matched = vocab_tokens
        .iter()
        .any(|t| n5_words.contains(t.orthographic_base_form()));

    assert!(
        any_matched,
        "At least one token base form should match JLPT N5 words. Tokens: {:?}",
        vocab_tokens
            .iter()
            .map(|t| (t.orthographic_base_form(), t.part_of_speech()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn vocabulary_card_content_key_matches_jlpt_n5() {
    init_real_dictionaries();

    let n5_words = load_jlpt_n5_words();
    let result = VocabularyCard::from_text("私は本を読みます", &NativeLanguage::Russian);

    assert!(
        !result.cards.is_empty(),
        "Should create at least one card. Skipped: {:?}",
        result.skipped_no_translation
    );

    let any_matched = result.cards.iter().any(|card| {
        let content_key = Card::Vocabulary(card.clone()).content_key();
        n5_words.contains(&content_key)
    });

    assert!(
        any_matched,
        "At least one card content_key should match JLPT N5. Keys: {:?}",
        result
            .cards
            .iter()
            .map(|c| Card::Vocabulary(c.clone()).content_key())
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn mark_known_and_recalculate_updates_jlpt_progress() {
    init_real_dictionaries();

    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );

    let result = VocabularyCard::from_text("私は本を読みます", &NativeLanguage::Russian);
    assert!(
        !result.cards.is_empty(),
        "Should create cards. Skipped: {:?}",
        result.skipped_no_translation
    );

    let mut card_ids = Vec::new();
    for vocab_card in result.cards {
        let card = Card::Vocabulary(vocab_card);
        let study_card = user.create_card(card).expect("Failed to create card");
        card_ids.push(*study_card.card_id());
    }

    let repo = InMemoryUserRepository::with_user(user);
    for card_id in &card_ids {
        let use_case = MarkCardAsKnownUseCase::new(&repo);
        use_case
            .execute(*card_id)
            .await
            .expect("Failed to mark as known");
    }

    let mut user = repo.get_current_user().await.unwrap().unwrap();

    let content = build_real_jlpt_content();

    let n5_total = content.total_words(JapaneseLevel::N5);
    assert!(n5_total > 0, "JLPT N5 word list should not be empty");

    user.recalculate_jlpt_progress(&content);

    let n5_progress = user
        .jlpt_progress()
        .level_progress(JapaneseLevel::N5)
        .unwrap();
    let known_count = user
        .knowledge_set()
        .study_cards()
        .values()
        .filter(|sc| sc.memory().is_known_card())
        .count();

    assert!(
        n5_progress.words.learned > 0,
        "At least one N5 word should be learned. Learned: {}, Total: {}, Cards known: {}",
        n5_progress.words.learned,
        n5_progress.words.total,
        known_count,
    );
}
