use crate::i18n::*;
use crate::ui_components::{AudioButtons, Tag, TagVariant};
use leptos::prelude::*;
use origa::domain::PartOfSpeech;

use super::card_type::CardType;
use super::pos_label::part_of_speech_label;
use super::quiz_card::QuizVariant;

fn quiz_variant_matches_card_type(quiz_variant: QuizVariant, card_type: CardType) -> bool {
    matches!(
        (quiz_variant, card_type),
        (QuizVariant::Grammar, CardType::Grammar)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_variant_matches_grammar_card_type() {
        assert!(quiz_variant_matches_card_type(
            QuizVariant::Grammar,
            CardType::Grammar
        ));
    }

    #[test]
    fn meaning_variant_never_matches() {
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Meaning,
            CardType::Grammar
        ));
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Meaning,
            CardType::Vocabulary
        ));
    }

    #[test]
    fn reading_variant_never_matches() {
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Reading,
            CardType::Grammar
        ));
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Reading,
            CardType::Kanji
        ));
    }

    #[test]
    fn grammar_variant_does_not_match_other_card_types() {
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Grammar,
            CardType::Vocabulary
        ));
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Grammar,
            CardType::Kanji
        ));
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Grammar,
            CardType::Phrase
        ));
    }
}

#[component]
pub fn QuizCardHeader(
    card_type: CardType,
    question_text: String,
    #[prop(optional)] quiz_variant: QuizVariant,
    #[prop(default = None)] part_of_speech: Option<PartOfSpeech>,
) -> impl IntoView {
    let i18n = use_i18n();
    let pos_label = StoredValue::new(part_of_speech.map(|p| part_of_speech_label(p, &i18n)));

    view! {
        <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-2">
                <Tag variant=Signal::derive(move || card_type.tag_variant())>
                    {card_type.label(&i18n)}
                </Tag>
                <Show when=move || pos_label.get_value().is_some()>
                    {move || {
                        pos_label
                            .get_value()
                            .map(|label| {
                                view! {
                                    // POS is secondary metadata, not a primary
                                    // category. DESIGN.md reserves coloured Tag
                                    // variants for distinguishing card TYPES and
                                    // assigns the muted Tertiary tier to secondary
                                    // metadata — keep Default here. Mirrors
                                    // `lesson_card_header.rs`.
                                    <Tag>
                                        {label}
                                    </Tag>
                                }
                            })
                    }}
                </Show>
                <Show when=move || !quiz_variant_matches_card_type(quiz_variant, card_type)>
                    {match quiz_variant {
                        QuizVariant::Meaning => view! {
                            <Tag variant=Signal::derive(move || TagVariant::Filled)>
                                {t!(i18n, lesson.quiz)}
                            </Tag>
                        }.into_any(),
                        QuizVariant::Reading => view! {
                            <Tag variant=Signal::derive(move || TagVariant::Filled)>
                                {t!(i18n, lesson.reading)}
                            </Tag>
                        }.into_any(),
                        QuizVariant::Grammar => view! {
                            <Tag variant=Signal::derive(move || TagVariant::Filled)>
                                {t!(i18n, lesson.grammar)}
                            </Tag>
                        }.into_any(),
                    }}
                </Show>
            </div>
            <Show when=move || card_type != CardType::Kanji>
                <AudioButtons
                    text=question_text.clone()
                    audio_path=None
                    class=Signal::derive(|| "".to_string())
                />
            </Show>
        </div>
    }
}
