use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{DailyHistoryItem, StudyCard};
use crate::domain::local_day;
use crate::domain::{RateMode, Rating};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StatsTracker {
    lesson_history: Vec<DailyHistoryItem>,
}

impl StatsTracker {
    pub fn new() -> Self {
        Self {
            lesson_history: Vec::new(),
        }
    }

    pub fn history(&self) -> &[DailyHistoryItem] {
        &self.lesson_history
    }

    pub fn new_cards_studied_today(&self) -> usize {
        let today_start = local_day::today_start();
        self.lesson_history
            .iter()
            .rev()
            .find(|item| item.timestamp() >= today_start)
            .map(|item| item.new_cards_studied_today() as usize)
            .unwrap_or(0)
    }

    pub fn phrase_cards_studied_today(&self) -> usize {
        let today_start = local_day::today_start();
        self.lesson_history
            .iter()
            .rev()
            .find(|item| item.timestamp() >= today_start)
            .map(|item| item.phrase_cards_studied_today() as usize)
            .unwrap_or(0)
    }

    pub fn update(
        &mut self,
        study_cards: &HashMap<Ulid, StudyCard>,
        rating: Rating,
        was_new: bool,
        is_phrase: bool,
        mode: RateMode,
    ) {
        super::stats_updater::update_history(
            study_cards,
            &mut self.lesson_history,
            rating,
            was_new,
            is_phrase,
            mode,
            local_day::today_start(),
        );
    }

    /// Закрытие руки знакомства: списывает дневной лимит за `count` карт
    /// одной операцией, без рейтингового пути.
    pub fn register_acquaintance_completions(
        &mut self,
        study_cards: &HashMap<Ulid, StudyCard>,
        count: usize,
    ) {
        super::stats_updater::register_new_cards_without_rating(
            study_cards,
            &mut self.lesson_history,
            count,
            local_day::today_start(),
        );
    }

    pub fn recalculate(&mut self, study_cards: &HashMap<Ulid, StudyCard>) {
        super::stats_updater::recalculate_daily_stats(
            study_cards,
            &mut self.lesson_history,
            local_day::today_start(),
        );
    }

    pub fn merge(&mut self, other: &StatsTracker) {
        for item in &other.lesson_history {
            // Ключ слияния — локальная календарная дата: одна зона на
            // устройство юзера, инстанты одного локального дня сливаются
            // в один айтем.
            let date = local_day::local_date(item.timestamp());
            if let Some(existing_item) = self
                .lesson_history
                .iter_mut()
                .rev()
                .find(|h| local_day::local_date(h.timestamp()) == date)
            {
                existing_item.merge_with(item);
            } else {
                self.lesson_history.push(item.clone());
            }
        }

        self.lesson_history.sort_by_key(|h| h.timestamp());
    }
}

#[cfg(test)]
#[path = "stats_tracker_tests.rs"]
mod tests;
