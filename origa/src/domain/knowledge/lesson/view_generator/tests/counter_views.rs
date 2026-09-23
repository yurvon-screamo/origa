//! Виды показа счётного суффикса (issue #415): монета ревью
//! Normal/CounterBindings, safe-default новичка и деградация пустой пачки.

use super::super::LessonViewGenerator;
use super::*;
use crate::domain::LessonCardView;
use rand::{SeedableRng, rngs::StdRng};

mod counter_view_tests {
    use super::*;

    fn counter_card(suffix: &str) -> StudyCard {
        crate::dictionary::counters::tests::init_test_counters();
        let mut counter = crate::domain::CounterCard::new(suffix);
        counter.ensure_registry_bindings();
        StudyCard::new(Card::Counter(counter))
    }

    /// KnowledgeSet живёт в вызывающем тесте: генератор заимствует его.
    fn generator(ks: &KnowledgeSet) -> LessonViewGenerator<'_> {
        LessonViewGenerator::new(ks, NativeLanguage::Russian)
    }

    /// Новая counter-карта — только через руку знакомства (Exclude):
    /// случайный слот в ревью обязан деградировать в безопасный Normal.
    #[test]
    fn counter_new_card_always_normal() {
        let sc = counter_card("本");
        let ks = KnowledgeSet::new();
        let mut view_gen = generator(&ks);
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

    /// Ревью-монета: оба вида достижимы, посторонних — нет. Пачка 本
    /// непуста (11 связок в фикстуре), поэтому Bindings несёт items.
    #[test]
    fn counter_review_coin_yields_both_kinds_only() {
        let sc = counter_card("本");
        let ks = KnowledgeSet::new();
        let mut view_gen = generator(&ks);
        let mut saw_normal = false;
        let mut saw_bindings = false;
        for seed in 0u64..200 {
            let mut rng = StdRng::seed_from_u64(seed);
            match view_gen.apply_view(&sc, false, &mut rng) {
                LessonCardView::Normal(_) => saw_normal = true,
                LessonCardView::CounterBindings { items, .. } => {
                    saw_bindings = true;
                    assert!(!items.is_empty(), "non-empty pack must carry prompts");
                },
                other => panic!("seed {seed}: unexpected view {other:?}"),
            }
        }
        assert!(saw_normal, "coin must reach the semantic showing");
        assert!(saw_bindings, "coin must reach the bindings session");
    }

    /// Пустая пачка (суффикс исчез из реестра → bindings не созданы)
    /// деградирует в Normal: слот «1/0 без кнопки дальше» стопорил бы урок.
    #[test]
    fn counter_review_empty_pack_degrades_to_normal() {
        crate::dictionary::counters::tests::init_test_counters();
        let sc = StudyCard::new(Card::Counter(crate::domain::CounterCard::new("虚")));
        let ks = KnowledgeSet::new();
        let mut view_gen = generator(&ks);
        for seed in 0u64..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            assert!(
                matches!(
                    view_gen.apply_view(&sc, false, &mut rng),
                    LessonCardView::Normal(_)
                ),
                "seed {seed}: empty pack must degrade to Normal"
            );
        }
    }
}
