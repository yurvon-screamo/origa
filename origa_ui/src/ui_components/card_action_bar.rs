use crate::i18n::use_i18n;
use crate::pages::shared::MarkAsKnownButton;
use crate::ui_components::{DeleteButton, FavoriteButton, HistoryButton, Tag, TagVariant, Tooltip};
use leptos::prelude::*;

#[component]
pub fn CardActionBar(
    #[prop(into)] tag_variant: Signal<TagVariant>,
    #[prop(into)] tag_label: Signal<String>,
    #[prop(optional)] is_favorite: Signal<bool>,
    #[prop(optional, into)] on_toggle_favorite: Option<Callback<()>>,
    #[prop(optional, into)] on_mark_as_known: Option<Callback<()>>,
    #[prop(optional)] show_mark_as_known: Signal<bool>,
    #[prop(optional, into)] on_history: Option<Callback<()>>,
    #[prop(optional, into)] on_delete: Option<Callback<()>>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional, into)] show_tag: Signal<bool>,
) -> impl IntoView {
    let favorite_test_id = Signal::derive(move || format!("{}-favorite-btn", test_id.get()));
    let mark_known_test_id = Signal::derive(move || format!("{}-mark-known-btn", test_id.get()));
    let history_test_id = Signal::derive(move || format!("{}-history-btn", test_id.get()));
    let delete_test_id = Signal::derive(move || format!("{}-delete-btn", test_id.get()));

    let mark_known_visible = move || show_mark_as_known.get();
    let i18n = use_i18n();

    view! {
        <div class="card-action-bar" data-testid=move || test_id.get() on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()>
            <Show when=move || show_tag.get()>
                <div class="card-action-status">
                    <Tag variant=tag_variant>{tag_label}</Tag>
                </div>
            </Show>
            <div class="card-action-toolbar" role="toolbar" aria-label="Card actions">
                {move || {
                    match on_toggle_favorite {
                        Some(cb) => view! {
                            <Tooltip text=Signal::derive(move || crate::i18n::td_string!(i18n.get_locale(), shared.favorite))>
                                <FavoriteButton
                                    is_favorite=is_favorite
                                    on_click=cb
                                    test_id=favorite_test_id
                                />
                            </Tooltip>
                        }
                            .into_any(),
                        None => ().into_any(),
                    }
                }}
                {move || {
                    match on_mark_as_known {
                        Some(cb) => view! {
                        <span
                            style:display=move || if mark_known_visible() { "inline-flex" } else { "none" }
                            aria-hidden=move || if mark_known_visible() { "false" } else { "true" }
                        >
                            <Tooltip text=Signal::derive(move || crate::i18n::td_string!(i18n.get_locale(), shared.mark_as_known))>
                                <MarkAsKnownButton on_click=cb test_id=mark_known_test_id />
                            </Tooltip>
                        </span>
                    }
                            .into_any(),
                        None => ().into_any(),
                    }
                }}
                {move || {
                    match on_history {
                        Some(cb) => view! {
                            <Tooltip text=Signal::derive(move || crate::i18n::td_string!(i18n.get_locale(), ui.card_history))>
                                <HistoryButton on_click=cb test_id=history_test_id />
                            </Tooltip>
                        }
                            .into_any(),
                        None => ().into_any(),
                    }
                }}
                {move || {
                    match on_delete {
                        Some(cb) => view! {
                            <Tooltip text=Signal::derive(move || crate::i18n::td_string!(i18n.get_locale(), common.delete))>
                                <DeleteButton on_click=cb test_id=delete_test_id />
                            </Tooltip>
                        }
                            .into_any(),
                        None => ().into_any(),
                    }
                }}
            </div>
        </div>
    }
}
