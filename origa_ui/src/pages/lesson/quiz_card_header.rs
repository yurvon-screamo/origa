use crate::i18n::*;
use crate::ui_components::{AudioButtons, Tag, TagVariant};
use leptos::prelude::*;

use super::card_type::CardType;

#[component]
pub fn QuizCardHeader(
    card_type: CardType,
    question_text: String,
    #[prop(optional)] quiz_variant: super::quiz_card::QuizVariant,
) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-2">
                <Tag variant=Signal::derive(move || card_type.tag_variant())>
                    {card_type.label(&i18n)}
                </Tag>
                {match quiz_variant {
                    super::quiz_card::QuizVariant::Meaning => view! {
                        <Tag variant=Signal::derive(move || TagVariant::Filled)>
                            {t!(i18n, lesson.quiz)}
                        </Tag>
                    }.into_any(),
                    super::quiz_card::QuizVariant::Reading => view! {
                        <Tag variant=Signal::derive(move || TagVariant::Filled)>
                            {t!(i18n, lesson.reading)}
                        </Tag>
                    }.into_any(),
                    super::quiz_card::QuizVariant::Grammar => view! {
                        <Tag variant=Signal::derive(move || TagVariant::Filled)>
                            {t!(i18n, lesson.grammar)}
                        </Tag>
                    }.into_any(),
                }}
            </div>
            <Show when=move || card_type != CardType::Kanji>
                <AudioButtons
                    text=question_text.clone()
                    audio_src=None
                    class=Signal::derive(|| "".to_string())
                />
            </Show>
        </div>
    }
}
