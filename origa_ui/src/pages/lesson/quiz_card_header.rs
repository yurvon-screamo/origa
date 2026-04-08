use crate::i18n::*;
use crate::ui_components::{AudioButtons, Tag, TagVariant};
use leptos::prelude::*;

use super::card_type::CardType;

#[component]
pub fn QuizCardHeader(card_type: CardType, question_text: String) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-2">
                <Tag variant=Signal::derive(move || card_type.tag_variant())>
                    {card_type.label(&i18n)}
                </Tag>
                <Tag variant=Signal::derive(move || TagVariant::Filled)>
                    {t!(i18n, lesson.quiz)}
                </Tag>
            </div>
            <Show when=move || card_type != CardType::Kanji>
                <AudioButtons
                    text=question_text.clone()
                    class=Signal::derive(|| "".to_string())
                />
            </Show>
        </div>
    }
}
