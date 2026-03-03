use crate::ui_components::{AudioButtons, Tag};
use leptos::prelude::*;

use super::card_type::CardType;

#[component]
pub fn LessonCardHeader(card_type: CardType, question_text: String) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between mb-4">
            <Tag variant=Signal::derive(move || card_type.tag_variant())>
                {card_type.label()}
            </Tag>
            <Show when=move || card_type != CardType::Kanji>
                <AudioButtons
                    text=question_text.clone()
                    class=Signal::derive(|| "".to_string())
                />
            </Show>
        </div>
    }
}
