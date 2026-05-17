use std::collections::HashSet;

use ulid::Ulid;

use super::lesson::{LessonCard, LessonData, LessonViewGenerator};
use super::{Card, KnowledgeSet, MAX_COMPANION_WORDS};
use crate::dictionary::kanji::get_kanji_info;

const MAX_COMPANION_CARDS_PER_LESSON: usize = 15;

pub(crate) fn add_kanji_companions(
    lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
) -> LessonData {
    let kanji_ids = collect_kanji_ids(&lesson_data, knowledge_set);
    if kanji_ids.is_empty() {
        return lesson_data;
    }

    let already_in_lesson: HashSet<Ulid> = lesson_data.cards.iter().map(|(id, _)| *id).collect();

    let companions = find_companion_cards(&kanji_ids, knowledge_set, &already_in_lesson);
    if companions.is_empty() {
        return lesson_data;
    }

    append_companions(lesson_data, knowledge_set, &companions)
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
            (*card_id, LessonCard::new(view, false))
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
                (*id, LessonCard::new(view, false))
            })
            .collect();
        LessonData::reorder_core_first_phrases_last(cards)
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
        let result = add_kanji_companions(lesson, &ks);

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

        let kanji_chars = ["人", "魅", "誤", "姓", "剤", "干", "唐", "伴"];

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
        let result = add_kanji_companions(lesson, &ks);

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
        let result = add_kanji_companions(lesson.clone(), &ks);

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
        let result = add_kanji_companions(lesson.clone(), &ks);

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
}
