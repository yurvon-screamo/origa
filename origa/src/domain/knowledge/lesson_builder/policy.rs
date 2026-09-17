use super::*;

/// Политика включения незнакомых карт в урок (docs/acquaintance-mode.md §9.3,
/// срез S3): режим знакомства требует, чтобы незнакомые карты не попадали в
/// ревью ни через один путь — впрыск, избранное, padding или companions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewCardPolicy {
    /// Историческое поведение: новые карты впрыскиваются в урок.
    Inject,
    /// Режим знакомства: незнакомые карты исключены из урока полностью.
    Exclude,
}

/// Должна ли карта быть исключена из урока по политике. Фразы освобождены:
/// они живут по собственным пайплайнам anchored/tail.
pub(crate) fn excluded_by_new_card_policy(
    card_type: CardType,
    is_new: bool,
    policy: NewCardPolicy,
) -> bool {
    policy == NewCardPolicy::Exclude && is_new && card_type != CardType::Phrase
}

#[cfg(test)]
mod policy_predicate_tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(CardType::Vocabulary, true, NewCardPolicy::Exclude, true)]
    #[case(CardType::Kanji, true, NewCardPolicy::Exclude, true)]
    #[case(CardType::Grammar, true, NewCardPolicy::Exclude, true)]
    #[case(CardType::Phrase, true, NewCardPolicy::Exclude, false)]
    #[case(CardType::Vocabulary, true, NewCardPolicy::Inject, false)]
    #[case(CardType::Vocabulary, false, NewCardPolicy::Exclude, false)]
    fn exclusion_follows_policy_type_and_novelty(
        #[case] card_type: CardType,
        #[case] is_new: bool,
        #[case] policy: NewCardPolicy,
        #[case] expected: bool,
    ) {
        assert_eq!(
            excluded_by_new_card_policy(card_type, is_new, policy),
            expected
        );
    }
}
