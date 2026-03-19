use super::super::shared::DeleteRequest;
use crate::ui_components::MarkdownText;
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
    is_deleting: Signal<bool>,
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
        <div class="p-4 border border-[var(--border-dark)] bg-[var(--bg-paper)] rounded-lg hover:border-[var(--accent-olive)] transition-all">
            <div class="flex justify-between items-start mb-2">
                <span class="text-3xl font-serif">{question}</span>
                <div class="flex gap-1">
                    <button
                        class="text-[var(--text-muted)] hover:text-[var(--accent-olive)] transition-colors"
                        on:click=move |_| on_toggle_favorite.run(card_id)
                    >
                        {if study_card.is_favorite() { "★" } else { "☆" }}
                    </button>
                    <button
                        class="text-[var(--text-muted)] hover:text-red-500 transition-colors disabled:opacity-50"
                        disabled=is_deleting
                        on:click=move |_| {
                            on_delete.run(DeleteRequest {
                                card_id,
                                on_success: Callback::new(move |_| ()),
                            });
                        }
                    >
                        "×"
                    </button>
                </div>
            </div>
            <MarkdownText content=Signal::derive(move || answer.clone()) known_kanji=known_kanji/>
        </div>
    }
}
