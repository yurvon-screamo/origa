use std::collections::{HashMap, HashSet};

use ulid::Ulid;

use super::lesson::{LessonCard, LessonData, LessonViewGenerator};
use super::{Card, KnowledgeSet, MAX_COMPANION_WORDS};
use crate::dictionary::kanji::get_kanji_info;
use crate::domain::japanese::JapaneseChar;
use crate::domain::value_objects::JapaneseLevel;

const MAX_COMPANION_CARDS_PER_LESSON: usize = 15;
const MAX_REVERSE_COMPANION_CARDS_PER_LESSON: usize = 10;

pub(crate) fn add_kanji_companions(
    lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    user_level: JapaneseLevel,
) -> LessonData {
    let already_in_lesson: HashSet<Ulid> = lesson_data.cards.iter().map(|(id, _)| *id).collect();

    let (lesson_data, already_in_lesson) =
        add_forward_companions(lesson_data, knowledge_set, already_in_lesson);

    add_reverse_companions(lesson_data, knowledge_set, user_level, &already_in_lesson)
}

fn add_forward_companions(
    lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    mut already_in_lesson: HashSet<Ulid>,
) -> (LessonData, HashSet<Ulid>) {
    let kanji_ids = collect_kanji_ids(&lesson_data, knowledge_set);
    if kanji_ids.is_empty() {
        return (lesson_data, already_in_lesson);
    }

    let companions = find_companion_cards(&kanji_ids, knowledge_set, &already_in_lesson);
    if companions.is_empty() {
        return (lesson_data, already_in_lesson);
    }

    for (id, _) in &companions {
        already_in_lesson.insert(*id);
    }

    (
        append_companions(lesson_data, knowledge_set, &companions),
        already_in_lesson,
    )
}

fn add_reverse_companions(
    lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    user_level: JapaneseLevel,
    already_in_lesson: &HashSet<Ulid>,
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
    );
    if candidates.is_empty() {
        return lesson_data;
    }

    let capped: Vec<_> = candidates
        .into_iter()
        .take(MAX_REVERSE_COMPANION_CARDS_PER_LESSON)
        .collect();

    append_companions(lesson_data, knowledge_set, &capped)
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
                if !already_in_lesson.contains(card_id) && !seen_companion_ids.contains(card_id) {
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
) -> LessonData {
    let mut generator = LessonViewGenerator::new(knowledge_set);

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
        let mut generator = LessonViewGenerator::new(knowledge_set);
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
        let result = add_kanji_companions(lesson, &ks, JapaneseLevel::N5);

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
        let result = add_kanji_companions(lesson, &ks, JapaneseLevel::N5);

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
        let result = add_kanji_companions(lesson.clone(), &ks, JapaneseLevel::N5);

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
        let result = add_kanji_companions(lesson.clone(), &ks, JapaneseLevel::N5);

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
        let result = add_kanji_companions(lesson, &ks, JapaneseLevel::N5);

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
        let result = add_kanji_companions(lesson, &ks, user_level);

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
        let result = add_kanji_companions(lesson, &ks, JapaneseLevel::N5);

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
        let result = add_kanji_companions(lesson, &ks, JapaneseLevel::N5);

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
        let result = add_kanji_companions(lesson, &ks, JapaneseLevel::N5);

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
        let result = add_kanji_companions(lesson, &ks, JapaneseLevel::N5);

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
        let result = add_kanji_companions(lesson, &ks, JapaneseLevel::N1);

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
        let result = add_kanji_companions(lesson, &ks, JapaneseLevel::N5);

        assert!(
            result.contains_key(extra_kanji_sc.card_id()),
            "Kanji {extra_kanji} should be added via reverse from forward companion vocab '{popular_word}'"
        );
    }
}
