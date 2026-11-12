use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage};
use std::collections::HashSet;

use super::answer_display::extract_card_answer;
use super::card_type::CardType;
use super::grammar_details_expand::GrammarDetailsExpand;
use super::kanji_card_data::extract_kanji_card_data;
use super::kanji_card_details::KanjiCardDetails;

/// The full-card expand on a quiz result: from the result the learner
/// unfolds the complete kanji card (readings, meaning, radicals,
/// writing, examples — the canonical [`KanjiCardDetails`]) or the
/// complete grammar rule ([`GrammarDetailsExpand`]).
///
/// Type-driven dispatch: cards without a separate full document
/// (vocabulary, phrase, counter, example) render nothing — their answer
/// is already revealed on the result side. Mount point is the card BODY
/// on the result side (content lives in the card, the verdict stays in
/// the feedback strip — direction B doctrine). A submitted result never
/// mutates while visible, so card data is extracted once at mount.
///
/// Kanji composition (plan §2): canonical mount as-is — readings and
/// the «Значение» row are always visible, the internal
/// `kanji-details-toggle` unfolds radicals/writing/examples; no outer
/// toggle. The «Значение» row carries the SAME `card.answer` text the
/// result previously echoed through a dedicated name box — the box is
/// superseded, the duplicate is avoided by construction. An empty name
/// (dictionary lookup failure, warn already logged) hides the whole
/// expand: the #822 guard against an empty «Значение» box transfers to
/// the details block.
#[component]
pub fn QuizFullCardExpand(
    card: DomainCard,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let expand_root = move || {
        let val = test_id.get();
        (!val.is_empty()).then_some(val)
    };
    let kanji_data = extract_kanji_card_data(&card, native_language);

    if let Some(data) = kanji_data {
        let name = extract_card_answer(&card, &native_language, &CardType::Kanji).text;
        if name.is_empty() {
            return view! { <div data-testid=expand_root /> }.into_any();
        }
        let symbol = data.symbol;
        return view! {
            <div data-testid=expand_root>
                <KanjiCardDetails
                    kanji=symbol
                    name=name
                    radicals=data.radicals
                    example_words=data.example_words
                    on_readings=data.on_readings
                    kun_readings=data.kun_readings
                    known_kanji
                    native_language
                />
            </div>
        }
        .into_any();
    }

    if let DomainCard::Grammar(grammar) = &card {
        let rule_id = *grammar.rule_id();
        let is_expanded = RwSignal::new(false);
        return view! {
            <div data-testid=expand_root>
                <GrammarDetailsExpand
                    rule_id
                    is_expanded
                    known_kanji=known_kanji.get_untracked()
                    native_language
                />
            </div>
        }
        .into_any();
    }

    view! { <div data-testid=expand_root /> }.into_any()
}
