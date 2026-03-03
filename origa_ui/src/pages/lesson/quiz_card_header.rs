use crate::ui_components::{AudioButtons, Tag, TagVariant};
use leptos::prelude::*;

use super::card_type::CardType;

#[component]
pub fn QuizCardHeader(card_type: CardType, question_text: String) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-2">
                <Tag variant=Signal::derive(move || card_type.tag_variant())>
                    {card_type.label()}
                </Tag>
                <Tag variant=Signal::derive(move || TagVariant::Filled)>
                    "Тест"
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
