use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use ulid::Ulid;

use super::lesson::{LessonCard, LessonData, LessonViewGenerator};
use super::{Card, KnowledgeSet, MAX_COMPANION_WORDS};
use crate::dictionary::kanji::get_kanji_info;
use crate::domain::japanese::JapaneseChar;
use crate::domain::value_objects::{JapaneseLevel, NativeLanguage};

const MAX_COMPANION_CARDS_PER_LESSON: usize = 15;
const MAX_REVERSE_COMPANION_CARDS_PER_LESSON: usize = 10;

pub(crate) fn add_kanji_companions(
    lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    user_level: JapaneseLevel,
    native_language: NativeLanguage,
    now: DateTime<Utc>,
) -> LessonData {
    let already_in_lesson: HashSet<Ulid> = lesson_data.cards.iter().map(|(id, _)| *id).collect();

    let (lesson_data, already_in_lesson) = add_forward_companions(
        lesson_data,
        knowledge_set,
        already_in_lesson,
        native_language,
        now,
    );

    add_reverse_companions(
        lesson_data,
        knowledge_set,
        user_level,
        &already_in_lesson,
        native_language,
        now,
    )
}

fn add_forward_companions(
    lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    mut already_in_lesson: HashSet<Ulid>,
    native_language: NativeLanguage,
    now: DateTime<Utc>,
) -> (LessonData, HashSet<Ulid>) {
    let kanji_ids = collect_kanji_ids(&lesson_data, knowledge_set);
    if kanji_ids.is_empty() {
        return (lesson_data, already_in_lesson);
    }

    let companions = find_companion_cards(&kanji_ids, knowledge_set, &already_in_lesson, now);
    if companions.is_empty() {
        return (lesson_data, already_in_lesson);
    }

    for (id, _) in &companions {
        already_in_lesson.insert(*id);
    }

    (
        append_companions(lesson_data, knowledge_set, &companions, native_language),
        already_in_lesson,
    )
}

fn add_reverse_companions(
    lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    user_level: JapaneseLevel,
    already_in_lesson: &HashSet<Ulid>,
    native_language: NativeLanguage,
    now: DateTime<Utc>,
) -> LessonData {
    let kanji_index: HashMap<char, (&Ulid, &super::StudyCard)> = knowledge_set
        .study_cards()
        .iter()
        .filter_map(|(id, sc)| {
            let Card::Kanji(k) = sc.card() else {
                return None;
            };
            k.kanji().text().chars().next().map(|ch| (ch, (id, sc)))
        })
        .collect();

    let vocab_kanji_chars = collect_kanji_from_vocab(&lesson_data, knowledge_set);
    if vocab_kanji_chars.is_empty() {
        return lesson_data;
    }

    let candidates = find_reverse_companions(
        &vocab_kanji_chars,
        &kanji_index,
        already_in_lesson,
        user_level,
        now,
    );
    if candidates.is_empty() {
        return lesson_data;
    }

    let capped: Vec<_> = candidates
        .into_iter()
        .take(MAX_REVERSE_COMPANION_CARDS_PER_LESSON)
        .collect();

    append_companions(lesson_data, knowledge_set, &capped, native_language)
}

fn collect_kanji_from_vocab(
    lesson_data: &LessonData,
    knowledge_set: &KnowledgeSet,
) -> HashSet<char> {
    let mut kanji_chars = HashSet::new();

    for (id, _) in &lesson_data.cards {
        let study_card = match knowledge_set.get_card(*id) {
            Some(sc) => sc,
            None => continue,
        };

        if let Card::Vocabulary(v) = study_card.card() {
            for ch in v.word().text().chars() {
                if ch.is_kanji() {
                    kanji_chars.insert(ch);
                }
            }
        }
    }

    kanji_chars
}

fn find_reverse_companions<'a>(
    kanji_chars: &HashSet<char>,
    kanji_index: &HashMap<char, (&'a Ulid, &'a super::StudyCard)>,
    already_in_lesson: &HashSet<Ulid>,
    user_level: JapaneseLevel,
    now: DateTime<Utc>,
) -> Vec<(Ulid, &'a super::StudyCard)> {
    let mut companions = Vec::new();

    for &ch in kanji_chars {
        let (card_id, study_card) = match kanji_index.get(&ch) {
            Some(&(id, sc)) => (id, sc),
            None => continue,
        };

        if already_in_lesson.contains(card_id) {
            continue;
        }

        // Отметка «знаю» сегодняшнего дня гасит карту в канале компаньонов
        // до конца дня (решение владельца: семантика «только сегодня»).
        if study_card.memory().marked_known_today(now) {
            continue;
        }

        let Card::Kanji(kanji_card) = study_card.card() else {
            continue;
        };
        let kanji_level = kanji_card.jlpt();
        if kanji_level > user_level {
            continue;
        }

        companions.push((*card_id, study_card));
    }

    companions
}

fn collect_kanji_ids(lesson_data: &LessonData, knowledge_set: &KnowledgeSet) -> Vec<Ulid> {
    lesson_data
        .cards
        .iter()
        .filter_map(|(id, _)| {
            let study_card = knowledge_set.get_card(*id)?;
            matches!(study_card.card(), Card::Kanji(_)).then_some(*id)
        })
        .collect()
}

fn find_companion_cards<'a>(
    kanji_ids: &[Ulid],
    knowledge_set: &'a KnowledgeSet,
    already_in_lesson: &HashSet<Ulid>,
    now: DateTime<Utc>,
) -> Vec<(Ulid, &'a super::StudyCard)> {
    let mut companions = Vec::new();
    let mut seen_companion_ids: HashSet<Ulid> = HashSet::new();

    for kanji_id in kanji_ids {
        if companions.len() >= MAX_COMPANION_CARDS_PER_LESSON {
            break;
        }

        let study_card = match knowledge_set.get_card(*kanji_id) {
            Some(sc) => sc,
            None => continue,
        };

        let kanji_char = match study_card.card() {
            Card::Kanji(k) => k.kanji().text(),
            _ => continue,
        };

        let kanji_info = match get_kanji_info(kanji_char) {
            Ok(info) => info,
            Err(_) => continue,
        };

        for word in kanji_info.popular_words().iter().take(MAX_COMPANION_WORDS) {
            if companions.len() >= MAX_COMPANION_CARDS_PER_LESSON {
                break;
            }

            if let Some((card_id, matching_sc)) = find_vocab_card(knowledge_set, word) {
                // Отметка «знаю» сегодняшнего дня гасит слово-кандидата:
                // слот потребляется без замещения из глубины списка —
                // семантика как у already_in_lesson (тише для юзера).
                if !already_in_lesson.contains(card_id)
                    && !seen_companion_ids.contains(card_id)
                    && !matching_sc.memory().marked_known_today(now)
                {
                    seen_companion_ids.insert(*card_id);
                    companions.push((*card_id, matching_sc));
                }
            }
        }
    }

    companions
}

fn find_vocab_card<'a>(
    knowledge_set: &'a KnowledgeSet,
    word: &str,
) -> Option<(&'a Ulid, &'a super::StudyCard)> {
    knowledge_set
        .study_cards()
        .iter()
        .find(|(_, sc)| matches!(sc.card(), Card::Vocabulary(vocab) if vocab.word().text() == word))
}

fn append_companions(
    mut lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    companions: &[(Ulid, &super::StudyCard)],
    native_language: NativeLanguage,
) -> LessonData {
    let mut generator = LessonViewGenerator::new(knowledge_set, native_language);

    let companion_lessons: Vec<(Ulid, LessonCard)> = companions
        .iter()
        .map(|(card_id, study_card)| {
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
            (*card_id, LessonCard::new(*card_id, view, false))
        })
        .collect();

    let insert_pos = lesson_data.core_count;
    for (i, companion) in companion_lessons.into_iter().enumerate() {
        lesson_data.cards.insert(insert_pos + i, companion);
    }
    lesson_data.core_count += companions.len();

    lesson_data
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::knowledge::{KanjiCard, VocabularyCard};
    use crate::domain::value_objects::Question;
    use crate::use_cases::init_real_dictionaries;
    use chrono::Utc;

    fn create_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    fn create_kanji_card(kanji: &str) -> Card {
        Card::Kanji(KanjiCard::new_test(kanji.to_string()))
    }

    fn build_empty_lesson_with_cards(
        knowledge_set: &KnowledgeSet,
        card_ids: &[Ulid],
    ) -> LessonData {
        let mut generator = LessonViewGenerator::new(knowledge_set, NativeLanguage::Russian);
        let cards: Vec<(Ulid, LessonCard)> = card_ids
            .iter()
            .map(|id| {
                let sc = knowledge_set.get_card(*id).unwrap();
                let view = generator.apply_view(sc, sc.is_new(), &mut rand::rng());
                (*id, LessonCard::new(*id, view, false))
            })
            .collect();
        let core_count = cards.len();
        LessonData { cards, core_count }
    }

    #[test]
    fn add_kanji_companions_includes_vocab_cards_when_kanji_in_lesson() {
        init_real_dictionaries();

        let mut ks = KnowledgeSet::new();
        let kanji_sc = ks.create_card(create_kanji_card("日")).unwrap();

        let kanji_info = get_kanji_info("日").unwrap();
        let first_popular = kanji_info.popular_words().first().unwrap().clone();
        let vocab_sc = ks.create_card(create_vocab_card(&first_popular)).unwrap();

        let lesson = build_empty_lesson_with_cards(&ks, &[*kanji_sc.card_id()]);
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        assert!(
            result.contains_key(vocab_sc.card_id()),
            "Companion vocab card '{}' should be in lesson",
            first_popular
        );
        assert!(
            result.core_count > 1,
            "core_count should include companion cards"
        );
    }

    #[test]
    fn add_kanji_companions_respects_max_limit() {
        init_real_dictionaries();

        let kanji_chars = [
            "人", "日", "年", "大", "出", "見", "食", "飲", "行", "来", "読", "書", "話", "聞",
            "買", "立", "走", "歩", "待", "使",
        ];

        let mut ks = KnowledgeSet::new();
        let mut lesson_card_ids = Vec::new();
        let mut all_popular_words = Vec::new();

        for kanji_char in &kanji_chars {
            let kanji_sc = ks.create_card(create_kanji_card(kanji_char)).unwrap();
            lesson_card_ids.push(*kanji_sc.card_id());

            let kanji_info = get_kanji_info(kanji_char).unwrap();
            for word in kanji_info.popular_words() {
                if !all_popular_words.contains(&word.as_str()) {
                    all_popular_words.push(word.as_str());
                }
            }
        }

        for word in &all_popular_words {
            ks.create_card(create_vocab_card(word)).unwrap();
        }

        assert!(
            all_popular_words.len() > MAX_COMPANION_CARDS_PER_LESSON,
            "Test setup must have more potential companions ({}) than the cap ({})",
            all_popular_words.len(),
            MAX_COMPANION_CARDS_PER_LESSON,
        );

        let lesson = build_empty_lesson_with_cards(&ks, &lesson_card_ids);
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        let companion_count = result.len() - lesson_card_ids.len();
        assert!(
            companion_count <= MAX_COMPANION_CARDS_PER_LESSON,
            "Companion cards should be capped at {MAX_COMPANION_CARDS_PER_LESSON}, got {companion_count}",
        );
        assert!(
            companion_count > 0,
            "Companion cards should be non-zero with real kanji and matching vocab cards",
        );
    }

    #[test]
    fn add_kanji_companions_no_duplicates() {
        init_real_dictionaries();

        let mut ks = KnowledgeSet::new();
        let kanji_sc = ks.create_card(create_kanji_card("日")).unwrap();

        let kanji_info = get_kanji_info("日").unwrap();
        let first_popular = kanji_info.popular_words().first().unwrap().clone();
        let vocab_sc = ks.create_card(create_vocab_card(&first_popular)).unwrap();

        let lesson =
            build_empty_lesson_with_cards(&ks, &[*kanji_sc.card_id(), *vocab_sc.card_id()]);
        let result = add_kanji_companions(
            lesson.clone(),
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        let count = result
            .cards
            .iter()
            .filter(|(id, _)| *id == *vocab_sc.card_id())
            .count();
        assert_eq!(
            count, 1,
            "Companion already in lesson should not be added again"
        );
    }

    #[test]
    fn add_kanji_companions_empty_when_no_kanji() {
        init_real_dictionaries();

        let mut ks = KnowledgeSet::new();
        let vocab_sc = ks.create_card(create_vocab_card("猫")).unwrap();

        let lesson = build_empty_lesson_with_cards(&ks, &[*vocab_sc.card_id()]);
        let result = add_kanji_companions(
            lesson.clone(),
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        assert_eq!(
            result.len(),
            lesson.len(),
            "No kanji in lesson should mean no companions added"
        );
        assert_eq!(
            result.core_count, lesson.core_count,
            "core_count should remain unchanged"
        );
    }

    // --- Reverse companion tests ---

    #[test]
    fn reverse_adds_kanji_from_vocab() {
        init_real_dictionaries();

        let mut ks = KnowledgeSet::new();
        let vocab_sc = ks.create_card(create_vocab_card("日本")).unwrap();
        let kanji_nichi_sc = ks.create_card(create_kanji_card("日")).unwrap();
        let kanji_hon_sc = ks.create_card(create_kanji_card("本")).unwrap();

        let lesson = build_empty_lesson_with_cards(&ks, &[*vocab_sc.card_id()]);
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        assert!(
            result.contains_key(kanji_nichi_sc.card_id()),
            "Kanji 日 should be added as reverse companion from vocab 日本"
        );
        assert!(
            result.contains_key(kanji_hon_sc.card_id()),
            "Kanji 本 should be added as reverse companion from vocab 日本"
        );
    }

    #[test]
    fn reverse_respects_jlpt_level_filter() {
        init_real_dictionaries();

        let mut ks = KnowledgeSet::new();

        let kanji_nichi_sc = ks.create_card(create_kanji_card("日")).unwrap();
        let nichi_level = if let Card::Kanji(k) = kanji_nichi_sc.card() {
            k.jlpt()
        } else {
            panic!("Expected Kanji card");
        };

        let n1_kanji = "鬱";
        let kanji_utsu_sc = ks.create_card(create_kanji_card(n1_kanji)).unwrap();
        let utsu_level = if let Card::Kanji(k) = kanji_utsu_sc.card() {
            k.jlpt()
        } else {
            panic!("Expected Kanji card");
        };

        assert!(
            utsu_level > nichi_level,
            "鬱 ({utsu_level:?}) should be higher JLPT than 日 ({nichi_level:?})"
        );

        let vocab_with_both = format!("{n1_kanji}日");
        let vocab_sc = ks.create_card(create_vocab_card(&vocab_with_both)).unwrap();

        let lesson = build_empty_lesson_with_cards(&ks, &[*vocab_sc.card_id()]);

        let user_level = nichi_level;
        let result =
            add_kanji_companions(lesson, &ks, user_level, NativeLanguage::Russian, Utc::now());

        assert!(
            result.contains_key(kanji_nichi_sc.card_id()),
            "Kanji 日 ({nichi_level:?}) should be included at user_level={user_level:?}"
        );
        assert!(
            !result.contains_key(kanji_utsu_sc.card_id()),
            "Kanji 鬱 ({utsu_level:?}) should be excluded at user_level={user_level:?}"
        );
    }

    #[test]
    fn reverse_no_duplicate_kanji() {
        init_real_dictionaries();

        let mut ks = KnowledgeSet::new();
        let kanji_nichi_sc = ks.create_card(create_kanji_card("日")).unwrap();
        let vocab_sc = ks.create_card(create_vocab_card("日本")).unwrap();

        let lesson =
            build_empty_lesson_with_cards(&ks, &[*kanji_nichi_sc.card_id(), *vocab_sc.card_id()]);
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        let count = result
            .cards
            .iter()
            .filter(|(id, _)| *id == *kanji_nichi_sc.card_id())
            .count();
        assert_eq!(
            count, 1,
            "Kanji 日 already in lesson should not be duplicated by reverse"
        );
    }

    #[test]
    fn reverse_skips_kanji_not_in_deck() {
        init_real_dictionaries();

        let mut ks = KnowledgeSet::new();
        let vocab_sc = ks.create_card(create_vocab_card("日本")).unwrap();
        let kanji_nichi_sc = ks.create_card(create_kanji_card("日")).unwrap();

        let lesson = build_empty_lesson_with_cards(&ks, &[*vocab_sc.card_id()]);
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        assert!(
            result.contains_key(kanji_nichi_sc.card_id()),
            "Kanji 日 should be added (exists in KnowledgeSet)"
        );

        let reverse_kanji_count = result
            .cards
            .iter()
            .filter(|(id, _)| *id != *vocab_sc.card_id() && *id != *kanji_nichi_sc.card_id())
            .count();
        assert_eq!(
            reverse_kanji_count, 0,
            "Kanji 本 should NOT be added (not in KnowledgeSet)"
        );
    }

    #[test]
    fn reverse_intra_dedup_shared_kanji() {
        init_real_dictionaries();

        let mut ks = KnowledgeSet::new();
        let vocab_nihon_sc = ks.create_card(create_vocab_card("日本")).unwrap();
        let vocab_nichiyoubi_sc = ks.create_card(create_vocab_card("日曜日")).unwrap();
        let kanji_nichi_sc = ks.create_card(create_kanji_card("日")).unwrap();

        let lesson = build_empty_lesson_with_cards(
            &ks,
            &[*vocab_nihon_sc.card_id(), *vocab_nichiyoubi_sc.card_id()],
        );
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        let count = result
            .cards
            .iter()
            .filter(|(id, _)| *id == *kanji_nichi_sc.card_id())
            .count();
        assert_eq!(
            count, 1,
            "Kanji 日 shared by two vocab words should be added exactly once"
        );
    }

    #[test]
    fn reverse_respects_max_limit() {
        init_real_dictionaries();

        let kanji_chars = [
            "日", "本", "人", "大", "出", "見", "食", "飲", "行", "来", "読",
        ];

        let mut ks = KnowledgeSet::new();
        let mut vocab_ids = Vec::new();

        for ch in &kanji_chars {
            ks.create_card(create_kanji_card(ch)).unwrap();
        }

        let vocab_word: String = kanji_chars.concat();
        let vocab_sc = ks.create_card(create_vocab_card(&vocab_word)).unwrap();
        vocab_ids.push(*vocab_sc.card_id());

        let lesson = build_empty_lesson_with_cards(&ks, &vocab_ids);
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        let reverse_count = result.cards.len() - vocab_ids.len();
        assert!(
            reverse_count <= MAX_REVERSE_COMPANION_CARDS_PER_LESSON,
            "Reverse companions should be capped at {MAX_REVERSE_COMPANION_CARDS_PER_LESSON}, got {reverse_count}"
        );
        assert!(reverse_count > 0, "Should have some reverse companions");
    }

    // Regression: previously the cap was tightened by `MAX_LESSON_SIZE.saturating_sub(cards.len())`,
    // so once the core section grew large enough the reverse budget shrank below the intended 10.
    // With a 41-card core (MAX_LESSON_SIZE - 9) the old code would have offered only 9 reverse slots.
    #[test]
    fn reverse_companions_uncapped_by_lesson_size() {
        init_real_dictionaries();

        let reverse_kanji_chars = ["日", "本", "人", "大", "出", "見", "食", "飲", "行", "来"];

        let mut ks = KnowledgeSet::new();

        for ch in &reverse_kanji_chars {
            ks.create_card(create_kanji_card(ch)).unwrap();
        }

        let mut lesson_card_ids = Vec::new();
        for i in 0..41 {
            let filler_sc = ks
                .create_card(create_vocab_card(&format!("filler{i}")))
                .unwrap();
            lesson_card_ids.push(*filler_sc.card_id());
        }

        let anchor_vocab: String = reverse_kanji_chars.concat();
        let anchor_sc = ks.create_card(create_vocab_card(&anchor_vocab)).unwrap();
        lesson_card_ids.push(*anchor_sc.card_id());

        assert_eq!(
            lesson_card_ids.len(),
            42,
            "Test setup must exceed the historical bug threshold (41), got {}",
            lesson_card_ids.len()
        );

        let lesson = build_empty_lesson_with_cards(&ks, &lesson_card_ids);
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N1,
            NativeLanguage::Russian,
            Utc::now(),
        );

        let reverse_count = result.cards.len() - lesson_card_ids.len();
        assert_eq!(
            reverse_count, MAX_REVERSE_COMPANION_CARDS_PER_LESSON,
            "Reverse companions should reach the full {} even with a large core section, got {}",
            MAX_REVERSE_COMPANION_CARDS_PER_LESSON, reverse_count
        );
    }

    #[test]
    fn reverse_uses_forward_vocab_as_source() {
        init_real_dictionaries();

        let mut ks = KnowledgeSet::new();
        let kanji_nichi_sc = ks.create_card(create_kanji_card("日")).unwrap();

        let kanji_info = get_kanji_info("日").unwrap();

        let popular_word = kanji_info
            .popular_words()
            .iter()
            .find(|w| w.chars().any(|c| c.is_kanji() && c != '日'))
            .expect("日 should have a popular word with a different kanji")
            .clone();

        let extra_kanji: char = popular_word
            .chars()
            .find(|c| c.is_kanji() && *c != '日')
            .unwrap();
        let extra_kanji_sc = ks
            .create_card(create_kanji_card(&extra_kanji.to_string()))
            .unwrap();

        let _vocab_sc = ks.create_card(create_vocab_card(&popular_word)).unwrap();

        let lesson = build_empty_lesson_with_cards(&ks, &[*kanji_nichi_sc.card_id()]);
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        assert!(
            result.contains_key(extra_kanji_sc.card_id()),
            "Kanji {extra_kanji} should be added via reverse from forward companion vocab '{popular_word}'"
        );
    }

    // --- Отметка «знаю» [marked_known_at] в канале компаньонов ---

    fn set_known_mark(ks: &mut KnowledgeSet, card_id: &Ulid, ts: Option<DateTime<Utc>>) {
        ks.study_cards_mut_for_test()
            .get_mut(card_id)
            .unwrap()
            .memory_history_mut_for_test()
            .set_marked_known_at_for_test(ts);
    }

    fn seed_known_memory(ks: &mut KnowledgeSet, card_id: &Ulid) {
        ks.study_cards_mut_for_test()
            .get_mut(card_id)
            .unwrap()
            .seed_first_review(crate::domain::memory::MemoryState::new(
                crate::domain::memory::Stability::new(
                    crate::domain::memory::KNOWN_CARD_STABILITY_THRESHOLD + 1.0,
                )
                .unwrap(),
                crate::domain::memory::Difficulty::new(3.0).unwrap(),
                Utc::now() - chrono::Duration::days(1),
            ));
    }

    #[test]
    fn reverse_skips_kanji_marked_known_today() {
        init_real_dictionaries();

        // Arrange: слово 日本 тянет кандзи 日 и 本; 日 отмечен «Знаю» сегодня
        let mut ks = KnowledgeSet::new();
        let vocab_sc = ks.create_card(create_vocab_card("日本")).unwrap();
        let kanji_nichi_sc = ks.create_card(create_kanji_card("日")).unwrap();
        let kanji_hon_sc = ks.create_card(create_kanji_card("本")).unwrap();
        set_known_mark(&mut ks, kanji_nichi_sc.card_id(), Some(Utc::now()));

        // Act
        let lesson = build_empty_lesson_with_cards(&ks, [*vocab_sc.card_id()].as_slice());
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        // Assert: отмеченный кандзи заглушен до конца дня, сосед вошёл
        assert!(
            !result.contains_key(kanji_nichi_sc.card_id()),
            "kanji marked known today must not enter as a reverse companion"
        );
        assert!(
            result.contains_key(kanji_hon_sc.card_id()),
            "unmarked kanji of the same word must still enter"
        );
    }

    #[test]
    fn reverse_includes_kanji_marked_known_yesterday() {
        init_real_dictionaries();

        // Arrange: отметка «Знаю» вчерашняя — семантика «только сегодня»
        let mut ks = KnowledgeSet::new();
        let vocab_sc = ks.create_card(create_vocab_card("日本")).unwrap();
        let kanji_nichi_sc = ks.create_card(create_kanji_card("日")).unwrap();
        set_known_mark(
            &mut ks,
            kanji_nichi_sc.card_id(),
            Some(Utc::now() - chrono::Duration::hours(25)),
        );

        // Act
        let lesson = build_empty_lesson_with_cards(&ks, [*vocab_sc.card_id()].as_slice());
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        // Assert: назавтра канал компаньонов снова открыт
        assert!(
            result.contains_key(kanji_nichi_sc.card_id()),
            "yesterday's mark must not silence the companion channel today"
        );
    }

    #[test]
    fn forward_skips_word_marked_known_today() {
        init_real_dictionaries();

        // Arrange: кандзи 日 в уроке, его первое популярное слово отмечено сегодня
        let mut ks = KnowledgeSet::new();
        let kanji_sc = ks.create_card(create_kanji_card("日")).unwrap();
        let kanji_info = get_kanji_info("日").unwrap();
        let first_popular = kanji_info.popular_words().first().unwrap().clone();
        let vocab_sc = ks.create_card(create_vocab_card(&first_popular)).unwrap();
        set_known_mark(&mut ks, vocab_sc.card_id(), Some(Utc::now()));

        // Act
        let lesson = build_empty_lesson_with_cards(&ks, [*kanji_sc.card_id()].as_slice());
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        // Assert: слово-компаньон заглушено, слот не замещается глубиной списка
        assert!(
            !result.contains_key(vocab_sc.card_id()),
            "word marked known today must not attach as a forward companion"
        );
        assert_eq!(
            result.core_count, 1,
            "silenced candidate consumes its slot without substitution"
        );
    }

    #[test]
    fn forward_words_of_known_source_kanji_still_attach() {
        init_real_dictionaries();

        // Arrange: ИЗВЕСТНЫЙ кандзи 日 в уроке (законное core-ревью),
        // его популярное слово чисто — решение владельца №2: источник
        // не фильтруется
        let mut ks = KnowledgeSet::new();
        let kanji_sc = ks.create_card(create_kanji_card("日")).unwrap();
        seed_known_memory(&mut ks, kanji_sc.card_id());
        let kanji_info = get_kanji_info("日").unwrap();
        let first_popular = kanji_info.popular_words().first().unwrap().clone();
        let vocab_sc = ks.create_card(create_vocab_card(&first_popular)).unwrap();

        // Act
        let lesson = build_empty_lesson_with_cards(&ks, [*kanji_sc.card_id()].as_slice());
        let result = add_kanji_companions(
            lesson,
            &ks,
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            Utc::now(),
        );

        // Assert: слова известного источника притягиваются как раньше
        assert!(
            result.contains_key(vocab_sc.card_id()),
            "known source kanji in core must keep attaching its popular words"
        );
    }

    #[test]
    fn marked_today_due_card_still_enters_core() {
        init_real_dictionaries();

        // Arrange: due-карта с сегодняшней отметкой — фильтр живёт только
        // в канале компаньонов, core-отбор её не видит
        let mut ks = KnowledgeSet::new();
        let due_sc = ks.create_card(create_vocab_card("期限")).unwrap();
        ks.study_cards_mut_for_test()
            .get_mut(due_sc.card_id())
            .unwrap()
            .seed_first_review(crate::domain::memory::MemoryState::new(
                crate::domain::memory::Stability::new(5.0).unwrap(),
                crate::domain::memory::Difficulty::new(5.0).unwrap(),
                Utc::now() - chrono::Duration::days(2),
            ));
        set_known_mark(&mut ks, due_sc.card_id(), Some(Utc::now()));

        // Act
        let result = ks.cards_to_lesson_with_policy(
            crate::domain::DailyBudget::with_daily_cards(5),
            &crate::domain::JlptContent::new(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            crate::domain::NewCardPolicy::Exclude,
        );

        // Assert
        assert!(
            result.contains_key(due_sc.card_id()),
            "due card marked known today must still enter the lesson core"
        );
    }
}
