//! Виды показа счётного суффикса (issue #415): новичок — только через
//! руку знакомства; ревью — пачка связок «число × суффикс», каждая
//! оценена классическим «знаю / не знаю»; пустая пачка деградирует в
//! Normal (слот без показов застопорил бы урок).

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

    /// Ревью — пачка связок: сиднутые новички-связки едут одной пачкой
    /// (порядок — binding_showcase), никаких квиз-видов.
    #[test]
    fn counter_review_is_the_bindings_pack() {
        let sc = counter_card("本");
        let ks = KnowledgeSet::new();
        let mut view_gen = view_generator(&ks);
        let mut rng = StdRng::seed_from_u64(7);
        match view_gen.apply_view(&sc, false, &mut rng) {
            LessonCardView::CounterBindings { items, .. } => {
                assert!(!items.is_empty(), "seeded pack carries every newcomer");
                assert_eq!(items.len(), 11, "本 fixture: 1..=10 + 何");
            },
            other => panic!("expected the bindings pack, got {other:?}"),
        }
    }

    /// Порядок связок в пачке — случайный (юзер-репорт 0.8.0-rc: числовой
    /// порядок 1..10 выдавал подсказку «иду по кругу»). Состав пачки не
    /// меняется — перемешиваются только цифры.
    #[test]
    fn counter_pack_order_is_random_not_numeric() {
        let sc = counter_card("本");
        let ks = KnowledgeSet::new();
        let mut view_gen = view_generator(&ks);

        let orders: Vec<Vec<u8>> = (0u64..8)
            .map(|seed| {
                let mut rng = StdRng::seed_from_u64(seed);
                match view_gen.apply_view(&sc, false, &mut rng) {
                    LessonCardView::CounterBindings { items, .. } => {
                        items.iter().map(|i| i.number()).collect()
                    },
                    other => panic!("expected the bindings pack, got {other:?}"),
                }
            })
            .collect();

        // Состав пачки одинаков во всех показах.
        for order in &orders {
            let mut sorted = order.clone();
            sorted.sort_unstable();
            assert_eq!(
                sorted,
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                "состав пачки не изменился: {order:?}"
            );
        }

        // Порядок не привязан к числовому и различается между показами.
        let numeric = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0];
        assert!(
            orders.iter().any(|o| o != &numeric),
            "порядок пачки не должен быть числовым: {orders:?}"
        );
        let distinct = orders
            .iter()
            .fold(Vec::<&Vec<u8>>::new(), |mut acc, order| {
                if !acc.contains(&order) {
                    acc.push(order);
                }
                acc
            });
        assert!(
            distinct.len() > 1,
            "порядок должен различаться между показами: {orders:?}"
        );
    }

    /// Суффикс вне реестра (пустая таблица) — деградация в Normal.
    #[test]
    fn counter_review_unknown_suffix_degrades_to_normal() {
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
