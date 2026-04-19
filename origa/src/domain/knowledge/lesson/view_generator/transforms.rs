use crate::dictionary::grammar::get_rule_by_id;
use crate::domain::{Card, GrammarRuleCard, VocabularyCard};
use rand::{Rng, seq::SliceRandom};

use super::super::types::{GrammarInfo, LessonCardView};
use super::DEFAULT_LANG;

pub(crate) fn apply_reversed(card: &Card) -> LessonCardView {
    match card {
        Card::Vocabulary(vocab) => match vocab.revert(&DEFAULT_LANG) {
            Ok(reverted) => LessonCardView::Reversed(Card::Vocabulary(reverted)),
            Err(_) => LessonCardView::Normal(card.clone()),
        },
        Card::Kanji(_) | Card::Grammar(_) | Card::Phrase(_) => LessonCardView::Normal(card.clone()),
    }
}

pub(crate) fn apply_grammar_mutated<R: Rng>(
    card: &Card,
    known_grammars: &[GrammarRuleCard],
    rng: &mut R,
) -> LessonCardView {
    match card {
        Card::Vocabulary(vocab) => match select_applicable_grammar(vocab, known_grammars, rng) {
            Some(grammar_card) => {
                let rule = get_rule_by_id(grammar_card.rule_id());
                match rule {
                    Some(r) => match vocab.with_grammar_rule(r, &DEFAULT_LANG) {
                        Ok((mutated, grammar_description)) => {
                            let grammar_title = grammar_card
                                .title(&DEFAULT_LANG)
                                .map(|q| q.text().to_string())
                                .unwrap_or_else(|_| grammar_card.rule_id().to_string());
                            let grammar_info = GrammarInfo::new(
                                Some(grammar_card.rule_id().to_owned()),
                                grammar_title,
                                grammar_description,
                            );
                            LessonCardView::GrammarMutated {
                                card: Card::Vocabulary(mutated),
                                grammar_info,
                            }
                        },
                        Err(_) => LessonCardView::Normal(card.clone()),
                    },
                    None => LessonCardView::Normal(card.clone()),
                }
            },
            None => LessonCardView::Normal(card.clone()),
        },
        Card::Kanji(_) | Card::Grammar(_) | Card::Phrase(_) => LessonCardView::Normal(card.clone()),
    }
}

fn select_applicable_grammar<R: Rng>(
    vocab: &VocabularyCard,
    known_grammars: &[GrammarRuleCard],
    rng: &mut R,
) -> Option<GrammarRuleCard> {
    let word_part = vocab.part_of_speech().ok()?;

    let mut rules: Vec<_> = known_grammars
        .iter()
        .filter(|g| g.apply_to().contains(&word_part))
        .cloned()
        .collect();

    rules.shuffle(rng);
    rules.into_iter().next()
}
