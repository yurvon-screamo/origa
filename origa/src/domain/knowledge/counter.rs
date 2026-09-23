//! Карточка «Счётный суффикс» (issue #415): один суффикс (本) со всеми
//! связками (число × чтение) — одна карта. Карта хранит только пер-юзер
//! стейт — прецедент `GrammarRuleCard`: контент (глоссы, чтения,
//! нерегулярность, JLPT-уровень) резолвится из реестра `COUNTERS`
//! на рендере, поэтому правки датасета доходят до существующих карт без
//! миграций, а merge устройств сводится к слиянию памятей.
//!
//! Два вида знаний внутри одной карты:
//! - семантика («что считают») — `MemoryHistory` самого `StudyCard`,
//!   режим `StandardLesson`;
//! - связки (三本 → さんぼん) — своя `MemoryHistory` у каждой ячейки:
//!   ревью карты — классическое «знаю / не знаю» (решение владельца),
//!   связки гасятся «уже знаю» и подсвечиваются в таблице чтений.

use serde::{Deserialize, Serialize};

use crate::domain::OrigaError;
use crate::domain::memory::MemoryHistory;
use crate::domain::value_objects::{CardAnswer, NativeLanguage, Question};

use super::Card;

/// Ячейка памяти связки: число + собственная FSRS-память.
/// `number`: 1..=10, 0 = вопросительное 何; лексикализованные исключения
/// за пределами десятки (日 14/20/24, 歳 20) — такие же полноценные связки.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CounterBindingMemory {
    number: u8,
    #[serde(default)]
    memory: MemoryHistory,
}

impl CounterBindingMemory {
    pub fn new(number: u8) -> Self {
        Self {
            number,
            memory: MemoryHistory::new(),
        }
    }

    pub fn number(&self) -> u8 {
        self.number
    }

    pub fn memory(&self) -> &MemoryHistory {
        &self.memory
    }
}

/// Карта счётного суффикса: суффикс-ключ + памяти связок.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CounterCard {
    suffix: String,
    #[serde(default)]
    bindings: Vec<CounterBindingMemory>,
}

impl CounterCard {
    /// Создаёт карту с пустым набором связок. Сидирование (онбординг или
    /// миграция) обязано сразу вызвать [`Self::ensure_registry_bindings`],
    /// чтобы первый композитный показ увидел все ячейки реестра новичками.
    pub fn new(suffix: impl Into<String>) -> Self {
        Self {
            suffix: suffix.into(),
            bindings: Vec::new(),
        }
    }

    pub fn suffix(&self) -> &str {
        &self.suffix
    }

    pub fn bindings(&self) -> &[CounterBindingMemory] {
        &self.bindings
    }

    #[cfg(test)]
    pub(crate) fn bindings_mut(&mut self) -> &mut [CounterBindingMemory] {
        &mut self.bindings
    }
    /// Дополняет набор связок до реестра: новые числа реестра получают
    /// свежую (новую) память; существующие и сироты (числа, удалённые из
    /// реестра) не трогаются. Идемпотентен.
    pub fn ensure_registry_bindings(&mut self) {
        let Some(entry) = crate::dictionary::counters::get_counter(&self.suffix) else {
            return;
        };
        let known: std::collections::HashSet<u8> = self.bindings.iter().map(|b| b.number).collect();
        for reading in entry.readings() {
            if !known.contains(&reading.number()) {
                self.bindings
                    .push(CounterBindingMemory::new(reading.number()));
            }
        }
    }

    /// Память конкретной связки по числу.
    /// «Уже знаю»/скоринг (issue #415): юзер, знающий суффикс, знает и
    /// связки — каждая НОВАЯ ячейка получает известную память тем же
    /// сидом, что семантика в `mark_card_as_known` (без раздувания reps у
    /// уже выученных). Иначе первый композитный показ дриллил бы всю
    /// таблицу даже «знающего» юзера.
    pub fn mark_all_bindings_known(&mut self) {
        use crate::domain::memory::{
            Difficulty, KNOWN_CARD_STABILITY_THRESHOLD, MemoryState, Rating, Stability,
        };
        use chrono::{Duration, Utc};
        for binding in &mut self.bindings {
            if binding.memory().is_known_card() {
                continue;
            }
            let memory = MemoryState::new(
                Stability::new(KNOWN_CARD_STABILITY_THRESHOLD + 1.0)
                    .unwrap_or_else(|_| Stability::new(1.0).expect("1.0 is valid")),
                Difficulty::new(3.0).expect("3.0 is valid"),
                Utc::now() - Duration::days(1),
            );
            binding.memory.apply_review(memory, Rating::Easy);
        }
    }

    pub fn binding_memory(&self, number: u8) -> Option<&MemoryHistory> {
        self.bindings
            .iter()
            .find(|b| b.number == number)
            .map(|b| &b.memory)
    }

    /// Полностью ли нова карта: семантика нова И все связки новы.
    /// Семантика — память самого `StudyCard`, её новизну проверяет
    /// вызывающий (см. `KnowledgeSet::counter_aggregate_is_new`).
    pub fn all_bindings_new(&self) -> bool {
        self.bindings.iter().all(|b| b.memory.is_new())
    }

    /// Попарный мерж памятей связок по числу (синхронизация устройств).
    /// Контент не мержится — он резолвится из реестра, конфликтов
    /// слияния контента не существует как класс.
    pub fn merge_bindings(&mut self, other: &CounterCard) {
        if self.suffix != other.suffix {
            return;
        }
        for other_binding in &other.bindings {
            if let Some(mine) = self
                .bindings
                .iter_mut()
                .find(|b| b.number == other_binding.number)
            {
                mine.memory.merge(&other_binding.memory);
            } else {
                self.bindings.push(other_binding.clone());
            }
        }
    }

    /// Вопрос карты — сам суффикс.
    pub fn question(&self) -> Result<Question, OrigaError> {
        Question::new(self.suffix.clone()).map_err(|e| OrigaError::InvalidQuestion {
            reason: e.to_string(),
        })
    }

    /// Ответ карты — глосс реестра в локали юзера (цепочка
    /// requested → English → any → пусто; пусто недостижимо при
    /// прохождении валидатора датасета).
    pub fn answer(&self, lang: &NativeLanguage) -> Result<CardAnswer, OrigaError> {
        let gloss = crate::dictionary::counters::get_counter(&self.suffix)
            .map(|entry| crate::dictionary::counters::gloss_for(entry, *lang))
            .unwrap_or("")
            .to_string();
        CardAnswer::text(gloss).map_err(|e| OrigaError::InvalidAnswer {
            reason: e.to_string(),
        })
    }
}

impl From<CounterCard> for Card {
    fn from(card: CounterCard) -> Card {
        Card::Counter(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::counters::tests::{TEST_HON, TEST_NIN, init_test_counters};

    fn seeded_card(suffix: &str) -> CounterCard {
        let mut card = CounterCard::new(suffix);
        card.ensure_registry_bindings();
        card
    }

    fn rate_binding_progress(card: &mut CounterCard) {
        card.mark_all_bindings_known();
    }

    #[test]
    fn ensure_registry_bindings_creates_every_registry_cell_as_new() {
        init_test_counters();
        let card = seeded_card(TEST_HON);
        // 本-фикстура: чтения для 1..=10 и 何 — все новички
        assert_eq!(card.bindings().len(), 11);
        assert!(card.all_bindings_new());
    }

    #[test]
    fn ensure_registry_bindings_is_idempotent_and_keeps_orphans() {
        init_test_counters();
        let mut card = seeded_card(TEST_HON);
        rate_binding_progress(&mut card);
        card.ensure_registry_bindings();
        card.ensure_registry_bindings();
        assert_eq!(card.bindings().len(), 11, "no duplicates after re-run");
        assert!(!card.binding_memory(3).unwrap().is_new(), "progress kept");

        // Сирота: память числа, которого нет в реестре, не удаляется
        card.bindings_mut()[0] = CounterBindingMemory::new(99);
        card.ensure_registry_bindings();
        assert!(card.binding_memory(99).is_some(), "orphan memory survives");
    }

    /// Посев памяти отдельной связки для merge/serde-тестов (приватные
    /// поля доступны внутри модуля; рейтингового пути связок больше нет).
    fn seed_binding(card: &mut CounterCard, number: u8) {
        for binding in &mut card.bindings {
            if binding.number == number {
                let state = crate::domain::MemoryState::with_card_state(
                    crate::domain::Stability::new(10.0).unwrap(),
                    crate::domain::Difficulty::new(5.0).unwrap(),
                    chrono::Utc::now() - chrono::Duration::days(1),
                    crate::domain::CardState::Review,
                );
                binding.memory.seed(state);
            }
        }
    }

    #[test]
    fn merge_bindings_combines_memories_pairwise_by_number() {
        init_test_counters();
        let mut device_a = seeded_card(TEST_HON);
        let mut device_b = seeded_card(TEST_HON);
        seed_binding(&mut device_a, 3);
        seed_binding(&mut device_b, 6);

        device_a.merge_bindings(&device_b);
        assert!(!device_a.binding_memory(3).unwrap().is_new());
        assert!(!device_a.binding_memory(6).unwrap().is_new());
        assert!(device_a.binding_memory(1).unwrap().is_new());
        assert_eq!(device_a.bindings().len(), 11);

        // Разные суффиксы не мержатся
        let mut hon = seeded_card(TEST_HON);
        let nin = seeded_card(TEST_NIN);
        hon.merge_bindings(&nin);
        assert_eq!(hon.bindings().len(), 11);
    }

    #[test]
    fn serde_roundtrip_preserves_suffix_and_binding_memories() {
        init_test_counters();
        let mut card = seeded_card(TEST_HON);
        seed_binding(&mut card, 1);

        let json = serde_json::to_string(&card).unwrap();
        let restored: CounterCard = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, card);
        assert!(json.contains("\"suffix\""));
    }

    #[test]
    fn study_card_with_counter_survives_serde_roundtrip() {
        init_test_counters();
        let study = crate::domain::StudyCard::new(Card::Counter(seeded_card(TEST_HON)));
        let json = serde_json::to_string(&study).unwrap();
        let restored: crate::domain::StudyCard = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, study);
        assert!(matches!(restored.card(), Card::Counter(_)));
    }

    #[test]
    fn empty_registry_degrades_to_card_without_bindings() {
        init_test_counters();
        let mut card = CounterCard::new("虚");
        card.ensure_registry_bindings();
        assert!(card.bindings().is_empty());
        assert!(card.all_bindings_new());
    }
}
