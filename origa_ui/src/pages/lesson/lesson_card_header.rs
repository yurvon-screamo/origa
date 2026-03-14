use crate::ui_components::{AudioButtons, Tag};
use leptos::prelude::*;
use origa::domain::GrammarInfo;

use super::card_type::CardType;
use super::grammar_info_badge::GrammarInfoBadge;

#[component]
pub fn LessonCardHeader(
    card_type: CardType,
    question_text: String,
    grammar_info: Option<GrammarInfo>,
    show_answer: bool,
) -> impl IntoView {
    let grammar_info = StoredValue::new(grammar_info);
    view! {
        <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-2">
                <Tag variant=Signal::derive(move || card_type.tag_variant())>
                    {card_type.label()}
                </Tag>
                <Show when=move || show_answer && grammar_info.get_value().is_some()>
                    {move || {
                        grammar_info
                            .get_value()
                            .map(|info| {
                                view! {
                                    <GrammarInfoBadge
                                        title=info.title().to_string()
                                        description=info.description().to_string()
                                    />
                                }
                            })
                    }}
                </Show>
            </div>
            <Show when=move || card_type != CardType::Kanji>
                <AudioButtons text=question_text.clone() class=Signal::derive(|| "".to_string())/>
            </Show>
        </div>
    }
}
