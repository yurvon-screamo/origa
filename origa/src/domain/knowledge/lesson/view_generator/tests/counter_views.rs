//! Виды показа счётного суффикса (issue #415): ревью — классическое
//! «знаю / не знаю» (решение владельца: композитный квиз связок из
//! урока убран), новичок — только через руку знакомства.

use super::super::LessonViewGenerator;
use super::*;
use crate::domain::knowledge::lesson::types::LessonCardView;
use rand::{SeedableRng, rngs::StdRng};

mod counter_view_tests {
    use super::*;

    fn view_generator(ks: &KnowledgeSet) -> LessonViewGenerator<'_> {
        LessonViewGenerator::new(ks, NativeLanguage::Russian)
    }

    fn counter_card(suffix: &str) -> StudyCard {
        crate::dictionary::counters::tests::init_test_counters();
        let mut counter = crate::domain::CounterCard::new(suffix);
        counter.ensure_registry_bindings();
        StudyCard::new(Card::Counter(counter))
    }

    /// Новая counter-карта — только через руку знакомства (Exclude):
    /// случайный слот обязан деградировать в безопасный Normal.
    #[test]
    fn counter_new_card_always_normal() {
        let sc = counter_card("本");
        let ks = KnowledgeSet::new();
        let mut view_gen = view_generator(&ks);
        for seed in 0u64..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            assert!(
                matches!(
                    view_gen.apply_view(&sc, true, &mut rng),
                    LessonCardView::Normal(_)
                ),
                "seed {seed}: new counter card must not be tested"
            );
        }
    }

    /// Ревью — всегда семантический Normal: никаких квиз-видов, хоткеи и
    /// оценки — как у остальных карт (решение владельца).
    #[test]
    fn counter_review_always_normal() {
        let sc = counter_card("本");
        let ks = KnowledgeSet::new();
        let mut view_gen = view_generator(&ks);
        for seed in 0u64..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            assert!(
                matches!(
                    view_gen.apply_view(&sc, false, &mut rng),
                    LessonCardView::Normal(_)
                ),
                "seed {seed}: counter review must stay the classic know/don't-know"
            );
        }
    }

    /// Суффикс вне реестра (пустая таблица) — поведение то же: Normal.
    #[test]
    fn counter_review_unknown_suffix_still_normal() {
        crate::dictionary::counters::tests::init_test_counters();
        let sc = StudyCard::new(Card::Counter(crate::domain::CounterCard::new("虚")));
        let ks = KnowledgeSet::new();
        let mut view_gen = view_generator(&ks);
        let mut rng = StdRng::seed_from_u64(7);
        assert!(matches!(
            view_gen.apply_view(&sc, false, &mut rng),
            LessonCardView::Normal(_)
        ));
    }
}
