//! Тесты сборки руки: валидации состава и стартовые состояния.

use crate::domain::acquaintance::{AcquaintanceHand, AcquaintanceSubphase, CRITERION_SUCCESSSES};
use crate::domain::{CardType, OrigaError};
use rstest::rstest;
use ulid::Ulid;

#[test]
fn empty_hand_is_rejected() {
    // Act
    let result = AcquaintanceHand::new(vec![]);

    // Assert
    let error = result.unwrap_err();
    assert!(matches!(error, OrigaError::InvalidAcquaintanceHand { .. }));
}

#[test]
fn phrase_card_in_hand_is_rejected() {
    // Arrange
    let id = Ulid::new();

    // Act
    let result = AcquaintanceHand::new(vec![(id, CardType::Phrase)]);

    // Assert: фразы вне режима знакомства
    let error = result.unwrap_err();
    let OrigaError::InvalidAcquaintanceHand { reason } = error else {
        panic!("expected InvalidAcquaintanceHand, got {error:?}");
    };
    assert!(reason.contains("phrase"));
}

#[test]
fn duplicate_cards_in_hand_are_rejected() {
    // Arrange
    let id = Ulid::new();

    // Act
    let result = AcquaintanceHand::new(vec![(id, CardType::Vocabulary), (id, CardType::Kanji)]);

    // Assert
    let error = result.unwrap_err();
    let OrigaError::InvalidAcquaintanceHand { reason } = error else {
        panic!("expected InvalidAcquaintanceHand, got {error:?}");
    };
    assert!(reason.contains("duplicate"));
}

#[test]
fn new_hand_preserves_given_presentation_order() {
    // Arrange: кандзи первым, его слова сразу за ним — группирует вызывающий
    let (kanji, word_a, word_b) = (Ulid::new(), Ulid::new(), Ulid::new());
    let cards = vec![
        (kanji, CardType::Kanji),
        (word_a, CardType::Vocabulary),
        (word_b, CardType::Vocabulary),
    ];

    // Act
    let hand = AcquaintanceHand::new(cards).unwrap();

    // Assert
    assert_eq!(hand.presentation_order(), vec![kanji, word_a, word_b]);
    assert_eq!(hand.len(), 3);
    assert_eq!(
        hand.entry(word_a).map(|entry| entry.card_type()),
        Some(CardType::Vocabulary)
    );
}

#[rstest]
#[case::with_words(vec![(Ulid::new(), CardType::Vocabulary)], Some(AcquaintanceSubphase::Forward))]
#[case::without_words(vec![(Ulid::new(), CardType::Grammar)], None)]
fn new_hand_starts_training_in_expected_subphase(
    #[case] cards: Vec<(Ulid, CardType)>,
    #[case] expected: Option<AcquaintanceSubphase>,
) {
    // Act
    let hand = AcquaintanceHand::new(cards).unwrap();

    // Assert
    assert_eq!(hand.subphase(), expected);
}

#[test]
fn fresh_entries_start_with_zero_progress_below_criterion() {
    // Arrange
    let id = Ulid::new();
    let hand = AcquaintanceHand::new(vec![(id, CardType::Vocabulary)]).unwrap();

    // Act
    let entry = hand.entry(id).unwrap();

    // Assert
    assert_eq!(entry.progress_in(Some(AcquaintanceSubphase::Forward)), 0);
    assert_eq!(CRITERION_SUCCESSSES, 3);
}

/// Репродукция живого баг-репорта (issue #415, rc3): рука из шести
/// счётных суффиксов и двух слов обязана пройти показ → тренировку
/// Forward → смену на РУ→ЯП (Reverse) → закрыться после Reverse-витка
/// слов. Симптом владельца: «застряло, не смог выйти из знакомства и
/// перейти к формату РУ→ЯП».
#[test]
fn mixed_counter_word_hand_completes_through_reverse() {
    use super::*;
    use crate::domain::CardType;
    use ulid::Ulid;

    let counters: Vec<Ulid> = (0..6).map(|_| Ulid::new()).collect();
    let words: Vec<Ulid> = (0..2).map(|_| Ulid::new()).collect();

    let mut entries: Vec<(Ulid, CardType, u8, u8)> = counters
        .iter()
        .map(|&id| (id, CardType::Counter, 0, 0))
        .collect();
    entries.extend(words.iter().map(|&id| (id, CardType::Vocabulary, 0, 0)));

    let mut hand = AcquaintanceHand::new_test(entries, Some(AcquaintanceSubphase::Forward));

    // Forward-виток: каждый счётчик и слово получают критерий успехов;
    // после каждого успешного ответа UI зовёт advance_subphase_if_words_done
    // (контракт after_answer) — воспроизводим честный флоу.
    for _ in 0..CRITERION_SUCCESSSES {
        for &id in counters.iter().chain(words.iter()) {
            let outcome = hand
                .record_answer(id, true)
                .expect("forward answer records");
            assert!(
                !matches!(outcome, AnswerOutcome::HandCompleted),
                "смена стороны должна предшествовать завершению руки"
            );
            if matches!(outcome, AnswerOutcome::Counted { .. }) {
                hand.advance_subphase_if_words_done();
            }
        }
    }

    // Последний успех слов обязан переключить подфазу на РУ→ЯП.
    assert_eq!(
        hand.subphase(),
        Some(AcquaintanceSubphase::Reverse),
        "все критерии Forward закрыты — подфаза обязана смениться на Reverse"
    );

    // Reverse-виток: только слова; закрывают обратный критерий.
    let mut completed = false;
    for _ in 0..CRITERION_SUCCESSSES {
        for &id in words.iter() {
            let outcome = hand
                .record_answer(id, true)
                .expect("reverse answer records");
            if matches!(outcome, AnswerOutcome::HandCompleted) {
                completed = true;
            }
        }
    }
    assert!(
        completed,
        "рука из 6 счётчиков + 2 слов обязана закрыться после Reverse-витка слов"
    );
}
