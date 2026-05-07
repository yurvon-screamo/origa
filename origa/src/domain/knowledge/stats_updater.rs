use std::collections::HashMap;

use chrono::Utc;
use ulid::Ulid;

use super::daily_history::DailyStatsUpdate;
use super::{Card, DailyHistoryItem, StudyCard};
use crate::domain::Rating;

pub(crate) fn update_history(
    study_cards: &HashMap<Ulid, StudyCard>,
    lesson_history: &mut Vec<DailyHistoryItem>,
    rating: Rating,
    was_new: bool,
    is_phrase: bool,
) {
    let mut avg_stability = 0.0;
    let mut avg_difficulty = 0.0;
    let mut total_words = 0;
    let mut known_words = 0;
    let mut new_words = 0;
    let mut in_progress_words = 0;
    let mut high_difficulty_words = 0;

    for study_card in study_cards.values() {
        if matches!(study_card.card(), Card::Phrase(_)) {
            continue;
        }
        let memory = study_card.memory();
        avg_stability += memory.stability().map(|x| x.value()).unwrap_or(0.0);
        avg_difficulty += memory.difficulty().map(|x| x.value()).unwrap_or(0.0);
        total_words += 1;
        known_words += memory.is_known_card() as usize;
        new_words += memory.is_new() as usize;
        in_progress_words += memory.is_in_progress() as usize;
        high_difficulty_words += memory.is_high_difficulty() as usize;
    }

    if total_words == 0 {
        return;
    }

    avg_stability /= total_words as f64;
    avg_difficulty /= total_words as f64;

    let now = Utc::now();
    let today = now.date_naive();

    if let Some(existing_item) = lesson_history
        .iter_mut()
        .find(|item| item.timestamp().date_naive() == today)
    {
        if was_new && !is_phrase {
            existing_item.increment_new_cards_studied();
        }
        if was_new && is_phrase {
            existing_item.increment_phrase_cards_studied();
        }
        let new_cards_today = existing_item.new_cards_studied_today();
        let phrase_cards_today = existing_item.phrase_cards_studied_today();

        if is_phrase {
            existing_item.update_stats(DailyStatsUpdate {
                avg_stability,
                avg_difficulty,
                total_words,
                known_words,
                new_words,
                in_progress_words,
                high_difficulty_words,
                positive_ratings: existing_item.positive_ratings(),
                negative_ratings: existing_item.negative_ratings(),
                total_ratings: existing_item.total_ratings(),
                new_cards_studied_today: new_cards_today,
                phrase_cards_studied_today: phrase_cards_today,
            });
        } else {
            existing_item.update(
                DailyStatsUpdate {
                    avg_stability,
                    avg_difficulty,
                    total_words,
                    known_words,
                    new_words,
                    in_progress_words,
                    high_difficulty_words,
                    positive_ratings: existing_item.positive_ratings(),
                    negative_ratings: existing_item.negative_ratings(),
                    total_ratings: existing_item.total_ratings(),
                    new_cards_studied_today: new_cards_today,
                    phrase_cards_studied_today: phrase_cards_today,
                },
                rating,
            );
        }
    } else {
        let mut item = DailyHistoryItem::new();
        if was_new && !is_phrase {
            item.increment_new_cards_studied();
        }
        if was_new && is_phrase {
            item.increment_phrase_cards_studied();
        }

        if is_phrase {
            item.update_stats(DailyStatsUpdate {
                avg_stability,
                avg_difficulty,
                total_words,
                known_words,
                new_words,
                in_progress_words,
                high_difficulty_words,
                positive_ratings: 0,
                negative_ratings: 0,
                total_ratings: 0,
                new_cards_studied_today: item.new_cards_studied_today(),
                phrase_cards_studied_today: item.phrase_cards_studied_today(),
            });
        } else {
            item.update(
                DailyStatsUpdate {
                    avg_stability,
                    avg_difficulty,
                    total_words,
                    known_words,
                    new_words,
                    in_progress_words,
                    high_difficulty_words,
                    positive_ratings: 0,
                    negative_ratings: 0,
                    total_ratings: 0,
                    new_cards_studied_today: item.new_cards_studied_today(),
                    phrase_cards_studied_today: item.phrase_cards_studied_today(),
                },
                rating,
            );
        }
        lesson_history.push(item);
    }
}

pub(crate) fn recalculate_daily_stats(
    study_cards: &HashMap<Ulid, StudyCard>,
    lesson_history: &mut Vec<DailyHistoryItem>,
) {
    let mut avg_stability = 0.0;
    let mut avg_difficulty = 0.0;
    let mut total_words = 0;
    let mut known_words = 0;
    let mut new_words = 0;
    let mut in_progress_words = 0;
    let mut high_difficulty_words = 0;

    for study_card in study_cards.values() {
        if matches!(study_card.card(), Card::Phrase(_)) {
            continue;
        }
        let memory = study_card.memory();
        avg_stability += memory.stability().map(|x| x.value()).unwrap_or(0.0);
        avg_difficulty += memory.difficulty().map(|x| x.value()).unwrap_or(0.0);
        total_words += 1;
        known_words += memory.is_known_card() as usize;
        new_words += memory.is_new() as usize;
        in_progress_words += memory.is_in_progress() as usize;
        high_difficulty_words += memory.is_high_difficulty() as usize;
    }

    if total_words == 0 {
        return;
    }

    avg_stability /= total_words as f64;
    avg_difficulty /= total_words as f64;

    let today = Utc::now().date_naive();
    let (positive, negative, total) = study_cards
        .values()
        .filter(|card| !matches!(card.card(), Card::Phrase(_)))
        .flat_map(|card| card.memory().reviews())
        .filter(|review| review.timestamp().date_naive() == today)
        .fold((0, 0, 0), |(pos, neg, tot), review| match review.rating() {
            Rating::Easy | Rating::Good => (pos + 1, neg, tot + 1),
            Rating::Hard | Rating::Again => (pos, neg + 1, tot + 1),
        });

    let preserved_new_cards = lesson_history
        .iter()
        .rev()
        .find(|item| item.timestamp().date_naive() == today)
        .map(|item| item.new_cards_studied_today())
        .unwrap_or(0);

    let preserved_phrase_cards = lesson_history
        .iter()
        .rev()
        .find(|item| item.timestamp().date_naive() == today)
        .map(|item| item.phrase_cards_studied_today())
        .unwrap_or(0);

    if let Some(existing_item) = lesson_history
        .iter_mut()
        .find(|item| item.timestamp().date_naive() == today)
    {
        existing_item.update_stats(DailyStatsUpdate {
            avg_stability,
            avg_difficulty,
            total_words,
            known_words,
            new_words,
            in_progress_words,
            high_difficulty_words,
            positive_ratings: positive,
            negative_ratings: negative,
            total_ratings: total,
            new_cards_studied_today: preserved_new_cards,
            phrase_cards_studied_today: preserved_phrase_cards,
        });
    } else {
        let mut item = DailyHistoryItem::new();
        item.update_stats(DailyStatsUpdate {
            avg_stability,
            avg_difficulty,
            total_words,
            known_words,
            new_words,
            in_progress_words,
            high_difficulty_words,
            positive_ratings: positive,
            negative_ratings: negative,
            total_ratings: total,
            new_cards_studied_today: preserved_new_cards,
            phrase_cards_studied_today: preserved_phrase_cards,
        });
        lesson_history.push(item);
    }
}
