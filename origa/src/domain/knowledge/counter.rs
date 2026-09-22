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
//! - связки (三本 → さんぼん) — своя `MemoryHistory` у каждой ячейки,
//!   режим `CounterReview` (мини-оценки идут мимо `rate_card`).

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
    pub fn binding_memory(&self, number: u8) -> Option<&MemoryHistory> {
        self.bindings
            .iter()
            .find(|b| b.number == number)
            .map(|b| &b.memory)
    }

    /// Мутируемая память связки; несуществующее число — доменная ошибка
    /// (никакого молчаливого создания ячеек из рейтингового пути).
    pub(crate) fn binding_memory_mut(
        &mut self,
        number: u8,
    ) -> Result<&mut MemoryHistory, OrigaError> {
        self.bindings
            .iter_mut()
            .find(|b| b.number == number)
            .map(|b| &mut b.memory)
            .ok_or(OrigaError::CounterBindingNotFound {
                suffix: self.suffix.clone(),
                number,
            })
    }

    /// Полностью ли нова карта: семантика нова И все связки новы.
    /// Семантика — память самого `StudyCard`, её новизну проверяет
    /// вызывающий (см. `KnowledgeSet::counter_aggregate_is_new`).
    pub fn all_bindings_new(&self) -> bool {
        self.bindings.iter().all(|b| b.memory.is_new())
    }

    /// Состав и порядок показа пачки: due-связки по возрастанию срока
    /// следующего показа (тай-брейк — нерегулярная выше), затем новички
    /// (нерегулярные первыми). Числа без памяти вообще (недосид) не
    /// показываются — `ensure_registry_bindings` закрывает этот случай
    /// при сидировании.
    pub fn binding_showcase(&self) -> Vec<u8> {
        let Some(entry) = crate::dictionary::counters::get_counter(&self.suffix) else {
            return Vec::new();
        };
        let irregular = |number: u8| {
            entry
                .readings()
                .iter()
                .find(|r| r.number() == number)
                .is_some_and(|r| r.irregular())
        };

        let mut due: Vec<(u8, Option<&chrono::DateTime<chrono::Utc>>)> = self
            .bindings
            .iter()
            .filter(|b| !b.memory.is_new() && b.memory.is_due())
            .map(|b| (b.number, b.memory.next_review_date()))
            .collect();
        due.sort_by(|a, b| {
            let date_a = a.1.unwrap_or(&chrono::DateTime::<chrono::Utc>::MIN_UTC);
            let date_b = b.1.unwrap_or(&chrono::DateTime::<chrono::Utc>::MIN_UTC);
            date_a
                .cmp(date_b)
                .then_with(|| irregular(b.0).cmp(&irregular(a.0)))
        });

        let mut newcomers: Vec<u8> = self
            .bindings
            .iter()
            .filter(|b| b.memory.is_new())
            .map(|b| b.number)
            .collect();
        newcomers.sort_by_key(|&n| std::cmp::Reverse(irregular(n)));

        due.into_iter().map(|(n, _)| n).chain(newcomers).collect()
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

    fn rate_binding_good(card: &mut CounterCard, number: u8) {
        let memory = card.binding_memory_mut(number).unwrap();
        let next = crate::domain::srs::rate_memory(
            crate::domain::RateMode::CounterReview,
            crate::domain::Rating::Good,
            memory,
        )
        .unwrap();
        memory.apply_review(next, crate::domain::Rating::Good);
    }

    #[test]
    fn ensure_registry_bindings_creates_every_registry_cell_as_new() {
        init_test_counters();
        let card = seeded_card(TEST_HON);
        // 本-фикстура: чтения для 1..=10 и 何 — все новички
        assert_eq!(card.bindings().len(), 11);
        assert!(card.all_bindings_new());
        assert!(card.binding_showcase().len() == 11);
    }

    #[test]
    fn ensure_registry_bindings_is_idempotent_and_keeps_orphans() {
        init_test_counters();
        let mut card = seeded_card(TEST_HON);
        rate_binding_good(&mut card, 3);
        card.ensure_registry_bindings();
        card.ensure_registry_bindings();
        assert_eq!(card.bindings().len(), 11, "no duplicates after re-run");
        assert!(!card.binding_memory(3).unwrap().is_new(), "progress kept");

        // Сирота: память числа, которого нет в реестре, не удаляется
        card.bindings_mut()[0] = CounterBindingMemory::new(99);
        card.ensure_registry_bindings();
        assert!(card.binding_memory(99).is_some(), "orphan memory survives");
    }

    #[test]
    fn unknown_binding_number_is_a_domain_error() {
        init_test_counters();
        let mut card = seeded_card(TEST_HON);
        let err = card.binding_memory_mut(42).unwrap_err();
        assert!(matches!(err, OrigaError::CounterBindingNotFound { .. }));
    }

    #[test]
    fn showcase_orders_due_by_date_then_irregular_and_newcomers_last() {
        init_test_counters();
        let mut card = seeded_card(TEST_NIN);

        // 3×人 и 4×人 просрочены сильнее 7×人 (прошлые даты — будущие not due);
        // одинаковый срок: нерегулярная 4 (よにん) выше
        let now = chrono::Utc::now();
        fn make_due(card: &mut CounterCard, number: u8, date: chrono::DateTime<chrono::Utc>) {
            let memory = card.binding_memory_mut(number).unwrap();
            let state = crate::domain::MemoryState::with_card_state(
                crate::domain::Stability::new(10.0).unwrap(),
                crate::domain::Difficulty::new(5.0).unwrap(),
                date,
                crate::domain::CardState::Review,
            );
            memory.seed(state);
        }
        make_due(&mut card, 3, now - chrono::Duration::hours(2));
        make_due(&mut card, 7, now - chrono::Duration::hours(1));
        make_due(&mut card, 4, now - chrono::Duration::hours(2));

        let showcase = card.binding_showcase();
        // Одинаковый срок: нерегулярная 4 (よにん) стоит раньше 3 (さんにん)
        assert_eq!(showcase[0], 4);
        assert_eq!(showcase[1], 3);
        assert_eq!(showcase[2], 7);
        // Новички — после due, нерегулярные первыми (1 ひとり, 2 ふたり)
        assert_eq!(showcase[3], 1);
        assert_eq!(showcase[4], 2);
    }

    #[test]
    fn merge_bindings_combines_memories_pairwise_by_number() {
        init_test_counters();
        let mut device_a = seeded_card(TEST_HON);
        let mut device_b = seeded_card(TEST_HON);
        rate_binding_good(&mut device_a, 3);
        rate_binding_good(&mut device_b, 6);

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
        rate_binding_good(&mut card, 1);

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
        assert!(card.binding_showcase().is_empty());
        assert!(card.all_bindings_new());
    }
}
