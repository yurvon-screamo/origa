use super::super::shared::DeleteRequest;
use crate::ui_components::{Card, DeleteButton, FavoriteButton, MarkdownText};
use leptos::prelude::*;
use origa::domain::{NativeLanguage, StudyCard};
use std::collections::HashSet;
use ulid::Ulid;

#[component]
pub fn RadicalItem(
    study_card: StudyCard,
    native_language: NativeLanguage,
    on_toggle_favorite: Callback<Ulid>,
    on_delete: Callback<DeleteRequest>,
    #[allow(unused_variables)] is_deleting: Signal<bool>,
) -> impl IntoView {
    let card_id = *study_card.card_id();

    let question = study_card
        .card()
        .question(&native_language)
        .ok()
        .map(|q| q.text().to_string())
        .unwrap_or_default();

    let answer = study_card
        .card()
        .answer(&native_language)
        .ok()
        .map(|a| a.text().to_string())
        .unwrap_or_default();

    let known_kanji: HashSet<String> = HashSet::new();

    view! {
        <Card class="p-4">
            <div class="flex justify-between items-start mb-2">
                <span class="text-3xl font-serif">{question}</span>
                <div class="flex gap-1">
                    <FavoriteButton
                        is_favorite=Signal::derive(move || study_card.is_favorite())
                        on_click=Callback::new(move |_| on_toggle_favorite.run(card_id))
                    />
                    <DeleteButton
                        on_click=Callback::new(move |_| {
                            on_delete.run(DeleteRequest {
                                card_id,
                                on_success: Callback::new(move |_| ()),
                            });
                        })
                    />
                </div>
            </div>
            <MarkdownText content=Signal::derive(move || answer.clone()) known_kanji=known_kanji/>
        </Card>
    }
}
