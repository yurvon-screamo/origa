use super::*;
use crate::domain::JapaneseLevel;
use crate::domain::JlptContent;
use crate::domain::memory::MemoryState;
use crate::domain::value_objects::Question;
use chrono::Duration;

fn create_vocab_card(word: &str) -> Card {
    Card::Vocabulary(VocabularyCard::new(
        Question::new(word.to_string()).unwrap(),
    ))
}

fn create_known_memory_state() -> MemoryState {
    MemoryState::new(
        crate::domain::memory::Stability::new(15.0).unwrap(),
        crate::domain::memory::Difficulty::new(2.0).unwrap(),
        chrono::Utc::now(),
    )
}

#[test]
fn cards_to_lesson_includes_favorite_cards() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let study_card = knowledge_set.create_card(card).unwrap();
    let card_id = *study_card.card_id();

    knowledge_set.toggle_favorite(card_id).unwrap();

    let result = knowledge_set.cards_to_lesson(10, &JlptContent::new());
    assert!(result.contains_key(&card_id));
}

#[test]
fn cards_to_lesson_includes_high_difficulty_cards() {
    let mut knowledge_set = KnowledgeSet::new();

    let card1 = create_vocab_card("猫");
    let card2 = create_vocab_card("犬");

    let study1 = knowledge_set.create_card(card1).unwrap();
    let study2 = knowledge_set.create_card(card2).unwrap();

    knowledge_set
        .rate_card(*study1.card_id(), Rating::Again, RateMode::ShortTerm)
        .unwrap();

    knowledge_set
        .rate_card(*study2.card_id(), Rating::Easy, RateMode::StandardLesson)
        .unwrap();

    let result = knowledge_set.cards_to_lesson(10, &JlptContent::new());

    assert!(result.contains_key(study1.card_id()));
}

#[test]
fn handle_favorite_rating_easy_increases_streak() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.add_review(
        memory.clone(),
        ReviewLog::new(Rating::Good, Duration::days(1)),
    );
    study_card.toggle_favorite();

    assert_eq!(study_card.perfect_streak_since_known(), 0);
    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.perfect_streak_since_known(), 1);
}

#[test]
fn handle_favorite_rating_good_does_not_change_streak() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.add_review(
        memory.clone(),
        ReviewLog::new(Rating::Good, Duration::days(1)),
    );
    study_card.toggle_favorite();

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.perfect_streak_since_known(), 1);

    study_card.handle_favorite_rating(Rating::Good);
    assert_eq!(study_card.perfect_streak_since_known(), 1);
}

#[test]
fn handle_favorite_rating_hard_resets_streak() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.add_review(
        memory.clone(),
        ReviewLog::new(Rating::Good, Duration::days(1)),
    );
    study_card.toggle_favorite();

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.perfect_streak_since_known(), 1);

    study_card.handle_favorite_rating(Rating::Hard);
    assert_eq!(study_card.perfect_streak_since_known(), 0);
}

#[test]
fn handle_favorite_rating_again_resets_streak() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.add_review(
        memory.clone(),
        ReviewLog::new(Rating::Good, Duration::days(1)),
    );
    study_card.toggle_favorite();

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.perfect_streak_since_known(), 1);

    study_card.handle_favorite_rating(Rating::Again);
    assert_eq!(study_card.perfect_streak_since_known(), 0);
}

#[test]
fn handle_favorite_rating_five_easy_removes_favorite() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.add_review(
        memory.clone(),
        ReviewLog::new(Rating::Good, Duration::days(1)),
    );
    study_card.toggle_favorite();

    assert!(study_card.is_favorite());

    for _ in 0..4 {
        study_card.handle_favorite_rating(Rating::Easy);
        assert!(study_card.is_favorite());
    }

    study_card.handle_favorite_rating(Rating::Easy);
    assert!(!study_card.is_favorite());
    assert_eq!(study_card.perfect_streak_since_known(), 0);
}

#[test]
fn handle_favorite_rating_good_does_not_interrupt_accumulation() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.add_review(
        memory.clone(),
        ReviewLog::new(Rating::Good, Duration::days(1)),
    );
    study_card.toggle_favorite();

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.perfect_streak_since_known(), 1);

    study_card.handle_favorite_rating(Rating::Good);
    assert_eq!(study_card.perfect_streak_since_known(), 1);

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.perfect_streak_since_known(), 2);

    study_card.handle_favorite_rating(Rating::Good);
    assert_eq!(study_card.perfect_streak_since_known(), 2);

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.perfect_streak_since_known(), 3);

    study_card.handle_favorite_rating(Rating::Good);
    assert_eq!(study_card.perfect_streak_since_known(), 3);

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.perfect_streak_since_known(), 4);

    study_card.handle_favorite_rating(Rating::Good);
    assert_eq!(study_card.perfect_streak_since_known(), 4);

    study_card.handle_favorite_rating(Rating::Easy);
    assert!(!study_card.is_favorite());
}

#[test]
fn handle_favorite_rating_non_favorited_does_nothing() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.add_review(
        memory.clone(),
        ReviewLog::new(Rating::Good, Duration::days(1)),
    );

    assert!(!study_card.is_favorite());

    let initial_streak = study_card.perfect_streak_since_known();
    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.perfect_streak_since_known(), initial_streak);
}

#[test]
fn handle_favorite_rating_unknown_card_does_nothing() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    study_card.toggle_favorite();

    let initial_streak = study_card.perfect_streak_since_known();
    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.perfect_streak_since_known(), initial_streak);
}

#[test]
fn create_card_updates_daily_stats() {
    let mut knowledge_set = KnowledgeSet::new();

    assert!(knowledge_set.lesson_history().is_empty());

    let card1 = create_vocab_card("猫");
    knowledge_set.create_card(card1).unwrap();

    assert_eq!(knowledge_set.lesson_history().len(), 1);
    let history_item = &knowledge_set.lesson_history()[0];
    assert_eq!(history_item.total_words(), 1);
    assert_eq!(history_item.new_words(), 1);
    assert_eq!(history_item.known_words(), 0);
    assert_eq!(history_item.lessons_completed(), 0);

    let card2 = create_vocab_card("犬");
    knowledge_set.create_card(card2).unwrap();

    assert_eq!(knowledge_set.lesson_history().len(), 1);
    let history_item = &knowledge_set.lesson_history()[0];
    assert_eq!(history_item.total_words(), 2);
    assert_eq!(history_item.new_words(), 2);
    assert_eq!(history_item.lessons_completed(), 0);
}

#[test]
fn merge_respects_tombstones() {
    let mut local = KnowledgeSet::new();
    local.create_card(create_vocab_card("猫")).unwrap();
    let study2 = local.create_card(create_vocab_card("犬")).unwrap();
    local.create_card(create_vocab_card("鳥")).unwrap();

    let remote = local.clone();

    let card2_id = *study2.card_id();
    local.delete_card(card2_id).unwrap();

    assert_eq!(local.study_cards().len(), 2);
    assert!(local.deleted_cards().contains(&card2_id));

    local.merge(&remote);

    assert_eq!(
        local.study_cards().len(),
        2,
        "card2 не должна восстановиться"
    );
    assert!(
        local.deleted_cards().contains(&card2_id),
        "tombstone должен сохраниться"
    );
}

#[test]
fn rate_card_increments_lessons_completed() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let study_card = knowledge_set.create_card(card).unwrap();

    knowledge_set
        .rate_card(
            *study_card.card_id(),
            Rating::Good,
            RateMode::StandardLesson,
        )
        .unwrap();

    let history_item = &knowledge_set.lesson_history()[0];
    assert_eq!(history_item.lessons_completed(), 1);
}

#[test]
fn merge_study_cards_updates_existing() {
    let mut local = KnowledgeSet::new();
    let study_card = local.create_card(create_vocab_card("猫")).unwrap();
    let card_id = *study_card.card_id();

    assert!(
        local.get_card(card_id).unwrap().is_new(),
        "карточка должна быть новой до merge"
    );

    let mut remote = local.clone();
    remote
        .rate_card(card_id, Rating::Good, RateMode::StandardLesson)
        .unwrap();

    local.merge(&remote);

    let merged_card = local.get_card(card_id).unwrap();
    assert!(
        !merged_card.is_new(),
        "карточка не должна быть новой после merge"
    );
}

#[test]
fn merge_lessons_completed_takes_max() {
    let mut local = KnowledgeSet::new();
    let card1 = local.create_card(create_vocab_card("猫")).unwrap();
    local
        .rate_card(*card1.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();
    local
        .rate_card(*card1.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();

    let history_item = &local.lesson_history()[0];
    assert_eq!(history_item.lessons_completed(), 2);

    let mut remote = KnowledgeSet::new();
    let card2 = remote.create_card(create_vocab_card("犬")).unwrap();
    remote
        .rate_card(*card2.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();
    remote
        .rate_card(*card2.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();
    remote
        .rate_card(*card2.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();

    let remote_history_item = &remote.lesson_history()[0];
    assert_eq!(remote_history_item.lessons_completed(), 3);

    local.merge(&remote);

    let merged_history = &local.lesson_history()[0];
    assert_eq!(
        merged_history.lessons_completed(),
        3,
        "lessons_completed должен использовать max для идемпотентности"
    );
}

#[test]
fn merge_stats_recalculated_from_actual() {
    let mut local = KnowledgeSet::new();
    for i in 0..100 {
        local
            .create_card(create_vocab_card(&format!("word{i}")))
            .unwrap();
    }

    let mut remote = local.clone();

    for i in 0..50 {
        local
            .create_card(create_vocab_card(&format!("known{i}")))
            .unwrap();
    }

    for i in 0..150 {
        remote
            .create_card(create_vocab_card(&format!("remote{i}")))
            .unwrap();
    }

    local.merge(&remote);

    let history_item = &local.lesson_history()[0];
    assert_eq!(history_item.total_words(), 300);
}

#[test]
fn recalculate_daily_stats_preserves_new_cards_on_create_card() {
    let mut knowledge_set = KnowledgeSet::new();

    let mut studied_ids = Vec::new();
    for i in 0..5 {
        let card = create_vocab_card(&format!("word{i}"));
        let study_card = knowledge_set.create_card(card).unwrap();
        studied_ids.push(*study_card.card_id());
    }

    for id in studied_ids {
        knowledge_set
            .rate_card(id, Rating::Good, RateMode::StandardLesson)
            .unwrap();
    }

    assert_eq!(knowledge_set.new_cards_studied_today(), 5);

    knowledge_set
        .create_card(create_vocab_card("extra"))
        .unwrap();

    assert_eq!(
        knowledge_set.new_cards_studied_today(),
        5,
        "new_cards_studied_today should be preserved after create_card"
    );

    for i in 0..10 {
        knowledge_set
            .create_card(create_vocab_card(&format!("new{i}")))
            .unwrap();
    }

    let lesson_cards = knowledge_set.cards_to_lesson(10, &JlptContent::new());
    let new_in_lesson = lesson_cards
        .iter()
        .filter(|(id, _)| knowledge_set.get_card(**id).unwrap().memory().is_new())
        .count();
    assert!(
        new_in_lesson <= 5,
        "Expected at most 5 new cards in lesson, got {new_in_lesson}"
    );
}

#[test]
fn recalculate_daily_stats_preserves_new_cards_on_delete_card() {
    let mut knowledge_set = KnowledgeSet::new();

    let card1 = knowledge_set.create_card(create_vocab_card("a")).unwrap();
    let card2 = knowledge_set.create_card(create_vocab_card("b")).unwrap();
    knowledge_set.create_card(create_vocab_card("c")).unwrap();

    knowledge_set
        .rate_card(*card1.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();
    knowledge_set
        .rate_card(*card2.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();

    assert_eq!(knowledge_set.new_cards_studied_today(), 2);

    knowledge_set.delete_card(*card1.card_id()).unwrap();

    assert_eq!(
        knowledge_set.new_cards_studied_today(),
        2,
        "new_cards_studied_today should be preserved after delete_card"
    );
}

#[test]
fn new_cards_sorted_by_jlpt_level() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["食べる".to_string()].into_iter().collect(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N4,
        ["走る".to_string()].into_iter().collect(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N3,
        ["挑戦する".to_string()].into_iter().collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    let study_chousen = knowledge_set
        .create_card(create_vocab_card("挑戦する"))
        .unwrap();
    let study_taberu = knowledge_set
        .create_card(create_vocab_card("食べる"))
        .unwrap();
    let study_hashiru = knowledge_set
        .create_card(create_vocab_card("走る"))
        .unwrap();

    let result = knowledge_set.cards_to_lesson(2, &jlpt_content);

    assert!(
        result.contains_key(study_taberu.card_id()),
        "食べる (N5) should be selected — highest JLPT priority"
    );
    assert!(
        result.contains_key(study_hashiru.card_id()),
        "走る (N4) should be selected — second highest JLPT priority"
    );
    assert!(
        !result.contains_key(study_chousen.card_id()),
        "挑戦する (N3) should not be selected — daily limit reached"
    );
}

#[test]
fn new_cards_unknown_level_go_last() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["食べる".to_string()].into_iter().collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    let study_michigo = knowledge_set
        .create_card(create_vocab_card("未知語"))
        .unwrap();
    let study_taberu = knowledge_set
        .create_card(create_vocab_card("食べる"))
        .unwrap();

    let result = knowledge_set.cards_to_lesson(1, &jlpt_content);

    assert!(
        result.contains_key(study_taberu.card_id()),
        "食べる (N5) should be selected — known JLPT level"
    );
    assert!(
        !result.contains_key(study_michigo.card_id()),
        "未知語 (Unknown) should not be selected — unknown level has lowest priority"
    );
}

#[test]
fn new_cards_jlpt_sort_does_not_affect_other_categories() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["食べる".to_string()].into_iter().collect(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N4,
        ["走る".to_string()].into_iter().collect(),
    );
    jlpt_content
        .kanji_by_level
        .insert(JapaneseLevel::N5, ["日".to_string()].into_iter().collect());

    let mut knowledge_set = KnowledgeSet::new();
    let study_taberu = knowledge_set
        .create_card(create_vocab_card("食べる"))
        .unwrap();
    let study_hashiru = knowledge_set
        .create_card(create_vocab_card("走る"))
        .unwrap();
    let study_nichi = knowledge_set
        .create_card(Card::Kanji(KanjiCard::new_test("日".to_string())))
        .unwrap();

    knowledge_set
        .rate_card(*study_taberu.card_id(), Rating::Again, RateMode::ShortTerm)
        .unwrap();
    knowledge_set
        .toggle_favorite(*study_nichi.card_id())
        .unwrap();

    let result = knowledge_set.cards_to_lesson(10, &jlpt_content);

    assert!(
        result.contains_key(study_taberu.card_id()),
        "食べる (due, high difficulty) should be in lesson"
    );
    assert!(
        result.contains_key(study_hashiru.card_id()),
        "走る (new, N4) should be in lesson"
    );
    assert!(
        result.contains_key(study_nichi.card_id()),
        "日 (favorite) should be in lesson"
    );
}

#[test]
fn new_cards_interleaved_by_type_within_jlpt_level() {
    let grammar_rule_id_1 = Ulid::new();
    let grammar_rule_id_2 = Ulid::new();

    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["w1", "w2", "w3", "w4", "w5", "w6"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );
    jlpt_content.kanji_by_level.insert(
        JapaneseLevel::N5,
        ["k1", "k2"].into_iter().map(|s| s.to_string()).collect(),
    );
    jlpt_content.grammar_by_level.insert(
        JapaneseLevel::N5,
        [grammar_rule_id_1.to_string(), grammar_rule_id_2.to_string()]
            .into_iter()
            .collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    for word in ["w1", "w2", "w3", "w4", "w5", "w6"] {
        knowledge_set.create_card(create_vocab_card(word)).unwrap();
    }
    knowledge_set
        .create_card(Card::Kanji(KanjiCard::new_test("k1".to_string())))
        .unwrap();
    knowledge_set
        .create_card(Card::Kanji(KanjiCard::new_test("k2".to_string())))
        .unwrap();
    knowledge_set
        .create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            grammar_rule_id_1,
        )))
        .unwrap();
    knowledge_set
        .create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            grammar_rule_id_2,
        )))
        .unwrap();

    let result = knowledge_set.cards_to_lesson(5, &jlpt_content);

    assert_eq!(result.len(), 5, "Should return exactly 5 cards (limit=5)");

    let has_kanji = result
        .keys()
        .any(|id| matches!(knowledge_set.get_card(*id).unwrap().card(), Card::Kanji(_)));
    let has_grammar = result.keys().any(|id| {
        matches!(
            knowledge_set.get_card(*id).unwrap().card(),
            Card::Grammar(_)
        )
    });

    assert!(
        has_kanji,
        "Should contain at least one kanji — interleaving distributes types"
    );
    assert!(
        has_grammar,
        "Should contain at least one grammar — interleaving distributes types"
    );
}

#[test]
fn new_cards_interleave_handles_missing_type() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["w1", "w2", "w3", "w4", "w5"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    for word in ["w1", "w2", "w3", "w4", "w5"] {
        knowledge_set.create_card(create_vocab_card(word)).unwrap();
    }

    let result = knowledge_set.cards_to_lesson(5, &jlpt_content);

    assert_eq!(
        result.len(),
        5,
        "All 5 vocab cards should be included when no kanji/grammar exist"
    );
}

#[test]
fn new_cards_interleave_across_jlpt_levels() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["n5w1", "n5w2", "n5w3"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );
    jlpt_content.kanji_by_level.insert(
        JapaneseLevel::N5,
        ["n5k1"].into_iter().map(|s| s.to_string()).collect(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N4,
        ["n4w1", "n4w2", "n4w3"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );
    jlpt_content.kanji_by_level.insert(
        JapaneseLevel::N4,
        ["n4k1"].into_iter().map(|s| s.to_string()).collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    for word in ["n5w1", "n5w2", "n5w3"] {
        knowledge_set.create_card(create_vocab_card(word)).unwrap();
    }
    knowledge_set
        .create_card(Card::Kanji(KanjiCard::new_test("n5k1".to_string())))
        .unwrap();
    for word in ["n4w1", "n4w2", "n4w3"] {
        knowledge_set.create_card(create_vocab_card(word)).unwrap();
    }
    knowledge_set
        .create_card(Card::Kanji(KanjiCard::new_test("n4k1".to_string())))
        .unwrap();

    let result = knowledge_set.cards_to_lesson(4, &jlpt_content);

    assert_eq!(result.len(), 4, "Should return exactly 4 cards (limit=4)");

    let n5_words: HashSet<&str> = ["n5w1", "n5w2", "n5w3"].into_iter().collect();
    let n5_kanji: &str = "n5k1";
    let n4_words: HashSet<&str> = ["n4w1", "n4w2", "n4w3"].into_iter().collect();
    let n4_kanji: &str = "n4k1";

    for id in result.keys() {
        let card = knowledge_set.get_card(*id).unwrap().card();
        match card {
            Card::Vocabulary(v) => {
                let word = v.word().text();
                assert!(n5_words.contains(word), "{word} should be N5");
                assert!(!n4_words.contains(word), "{word} should not be N4");
            },
            Card::Kanji(k) => {
                let kanji = k.kanji().text();
                assert_eq!(kanji, n5_kanji, "Only N5 kanji should be selected");
                assert_ne!(kanji, n4_kanji, "N4 kanji should not be selected");
            },
            Card::Grammar(_) => panic!("No grammar cards in this test"),
            Card::Phrase(_) => panic!("No phrase cards in this test"),
        }
    }

    let has_n5_kanji = result
        .keys()
        .any(|id| matches!(knowledge_set.get_card(*id).unwrap().card(), Card::Kanji(_)));
    assert!(
        has_n5_kanji,
        "N5 group should include at least one kanji via interleaving"
    );
}

#[test]
fn new_cards_interleave_preserves_jlpt_priority() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["n5w1"].into_iter().map(|s| s.to_string()).collect(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N4,
        ["n4w1", "n4w2", "n4w3", "n4w4", "n4w5"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    let study_n5 = knowledge_set
        .create_card(create_vocab_card("n5w1"))
        .unwrap();
    for word in ["n4w1", "n4w2", "n4w3", "n4w4", "n4w5"] {
        knowledge_set.create_card(create_vocab_card(word)).unwrap();
    }

    let result = knowledge_set.cards_to_lesson(3, &jlpt_content);

    assert!(
        result.contains_key(study_n5.card_id()),
        "N5 card (n5w1) should be selected — highest JLPT priority"
    );

    let n4_count = result
        .keys()
        .filter(|id| {
            let card = knowledge_set.get_card(**id).unwrap().card();
            matches!(card, Card::Vocabulary(v) if v.word().text().starts_with("n4"))
        })
        .count();

    assert_eq!(
        n4_count, 2,
        "Should contain exactly 2 N4 cards (limit=3 minus 1 N5)"
    );
}

#[test]
fn phrase_new_cards_limited() {
    let mut knowledge_set = KnowledgeSet::new();
    for _ in 0..20 {
        let phrase_id = Ulid::new();
        knowledge_set
            .create_card(Card::Phrase(PhraseCard::new_test_with_id(phrase_id)))
            .unwrap();
    }
    for i in 0..5 {
        knowledge_set
            .create_card(create_vocab_card(&format!("vocab{i}")))
            .unwrap();
    }

    let result = knowledge_set.cards_to_lesson(3, &JlptContent::new());

    let phrase_count = result
        .keys()
        .filter(|id| {
            matches!(
                knowledge_set.get_card(**id).unwrap().card(),
                Card::Phrase(_)
            )
        })
        .count();
    let vocab_count = result
        .keys()
        .filter(|id| {
            matches!(
                knowledge_set.get_card(**id).unwrap().card(),
                Card::Vocabulary(_)
            )
        })
        .count();

    let expected_limit = 3 / 3; // daily_new_limit / PHRASE_NEW_RATIO
    assert!(
        phrase_count <= expected_limit,
        "Phrase cards should be limited to daily_new_limit/3, got {phrase_count}, expected <={expected_limit}"
    );
    assert!(
        vocab_count <= 3,
        "Vocab cards should respect daily limit, got {vocab_count}"
    );
    assert!(
        result.len() <= 50,
        "Total should not exceed MAX_LESSON_SIZE, got {}",
        result.len()
    );
}

#[test]
fn phrase_does_not_increment_new_cards_studied() {
    let mut knowledge_set = KnowledgeSet::new();
    let mut phrase_ids = Vec::new();
    for _ in 0..5 {
        let phrase_id = Ulid::new();
        let study_card = knowledge_set
            .create_card(Card::Phrase(PhraseCard::new_test_with_id(phrase_id)))
            .unwrap();
        phrase_ids.push(*study_card.card_id());
    }

    for id in phrase_ids {
        knowledge_set
            .rate_card(id, Rating::Good, RateMode::StandardLesson)
            .unwrap();
    }

    assert_eq!(
        knowledge_set.new_cards_studied_today(),
        0,
        "Phrase cards should not increment new_cards_studied_today"
    );
}

#[test]
fn phrase_new_cards_zero_when_limit_below_ratio() {
    let mut knowledge_set = KnowledgeSet::new();
    for _ in 0..20 {
        let phrase_id = Ulid::new();
        knowledge_set
            .create_card(Card::Phrase(PhraseCard::new_test_with_id(phrase_id)))
            .unwrap();
    }
    for i in 0..5 {
        knowledge_set
            .create_card(create_vocab_card(&format!("vocab{i}")))
            .unwrap();
    }

    // daily_new_limit=2 < PHRASE_NEW_RATIO=3 → phrase_new_limit=0
    let result = knowledge_set.cards_to_lesson(2, &JlptContent::new());

    let phrase_count = result
        .keys()
        .filter(|id| {
            matches!(
                knowledge_set.get_card(**id).unwrap().card(),
                Card::Phrase(_)
            )
        })
        .count();

    assert_eq!(
        phrase_count, 0,
        "Phrase cards should be 0 when daily_new_limit < PHRASE_NEW_RATIO, got {phrase_count}"
    );
}

#[test]
fn phrase_excluded_from_stats() {
    let mut knowledge_set = KnowledgeSet::new();
    let phrase_id = Ulid::new();
    let phrase_study = knowledge_set
        .create_card(Card::Phrase(PhraseCard::new_test_with_id(phrase_id)))
        .unwrap();
    let vocab_study = knowledge_set.create_card(create_vocab_card("猫")).unwrap();

    knowledge_set
        .rate_card(
            *phrase_study.card_id(),
            Rating::Good,
            RateMode::StandardLesson,
        )
        .unwrap();
    knowledge_set
        .rate_card(
            *vocab_study.card_id(),
            Rating::Good,
            RateMode::StandardLesson,
        )
        .unwrap();

    let history_item = &knowledge_set.lesson_history()[0];
    assert_eq!(
        history_item.total_words(),
        1,
        "Only vocab should be counted in total_words"
    );
    assert_eq!(
        history_item.lessons_completed(),
        1,
        "Only vocab should increment lessons_completed"
    );
}

#[test]
fn limited_types_still_respect_daily_limit() {
    let mut knowledge_set = KnowledgeSet::new();
    for i in 0..10 {
        knowledge_set
            .create_card(create_vocab_card(&format!("vocab{i}")))
            .unwrap();
    }
    for i in 0..10 {
        knowledge_set
            .create_card(Card::Kanji(KanjiCard::new_test(format!("kanji{i}"))))
            .unwrap();
    }

    let result = knowledge_set.cards_to_lesson(5, &JlptContent::new());

    assert!(
        result.len() <= 5,
        "Vocab + Kanji should respect daily limit of 5, got {}",
        result.len()
    );
}

#[test]
fn lesson_size_respects_max_limit() {
    let mut knowledge_set = KnowledgeSet::new();

    for i in 0..100 {
        let study_card = knowledge_set
            .create_card(create_vocab_card(&format!("word{i}")))
            .unwrap();
        knowledge_set
            .rate_card(
                *study_card.card_id(),
                Rating::Easy,
                RateMode::StandardLesson,
            )
            .unwrap();
    }

    let result = knowledge_set.cards_to_lesson(100, &JlptContent::new());

    assert!(
        result.len() <= 50,
        "Total lesson size should not exceed MAX_LESSON_SIZE, got {}",
        result.len()
    );
}

#[test]
fn high_difficulty_cards_respect_max_lesson_size() {
    let mut knowledge_set = KnowledgeSet::new();

    // 100 карточек — все high-difficulty и due (rated Again + ShortTerm)
    for i in 0..100 {
        let study_card = knowledge_set
            .create_card(create_vocab_card(&format!("hard{i}")))
            .unwrap();
        knowledge_set
            .rate_card(*study_card.card_id(), Rating::Again, RateMode::ShortTerm)
            .unwrap();
    }

    let result = knowledge_set.cards_to_lesson(10, &JlptContent::new());

    assert!(
        result.len() <= 50,
        "High-difficulty cards should be capped at MAX_LESSON_SIZE, got {}",
        result.len()
    );
}
