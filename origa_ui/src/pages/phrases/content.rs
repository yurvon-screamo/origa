use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::super::shared::{
    CardListExtras, CardsLoadedCallback, ListGrouping, card_list_view, create_card_list_context,
};
use super::lazy_details::{load_and_refresh, missing_details, phrase_ids_of};
use super::phrase_card_item::PhraseCardItem;
use crate::i18n::{td_string, use_i18n};
use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, StudyCard};
use ulid::Ulid;

#[component]
pub fn PhrasesContent(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    // #540-В1: the page is data-lazy. The full id list is only recorded
    // (for the search-driven backfill); details load per visible slice.
    let all_phrase_ids: RwSignal<Vec<Ulid>> = RwSignal::new(Vec::new());
    let search: RwSignal<String> = RwSignal::new(String::new());
    let search_backfill_loading: RwSignal<bool> = RwSignal::new(false);
    let backfill_done: RwSignal<bool> = RwSignal::new(false);

    let on_cards_loaded: CardsLoadedCallback = Arc::new(move |cards: &[StudyCard]| {
        all_phrase_ids.set(phrase_ids_of(cards));
    });

    // Visible-slice lazy load: runs on first render, "load more" and
    // filter/search switches. While a load is in flight, later triggers
    // only mark a rerun — the loop picks the freshest visible ids after
    // the current batch lands (no lost tail, no duplicate batches).
    let visible_ids: RwSignal<Vec<Ulid>> = RwSignal::new(Vec::new());
    let visible_load_running = Arc::new(AtomicBool::new(false));
    let visible_load_rerun = Arc::new(AtomicBool::new(false));
    let on_visible_cards = {
        let refresh = refresh_trigger;
        let running = visible_load_running.clone();
        let rerun = visible_load_rerun.clone();
        let backfill_running = search_backfill_loading;
        Arc::new(move |cards: &[StudyCard]| {
            // The search backfill covers EVERY phrase — a visible-slice
            // load racing it would only collect Errs on the in-flight
            // chunks. Skip; the backfill's refresh re-renders the cards.
            if backfill_running.get_untracked() {
                return;
            }
            let ids = phrase_ids_of(cards);
            visible_ids.set(ids.clone());
            if missing_details(&ids).is_empty() {
                return;
            }
            if running.swap(true, Ordering::AcqRel) {
                rerun.store(true, Ordering::Release);
                return;
            }
            let running = running.clone();
            let rerun = rerun.clone();
            spawn_local(async move {
                loop {
                    let ids = visible_ids.get_untracked();
                    if !ids.is_empty() {
                        load_and_refresh(ids, refresh).await;
                    }
                    if !rerun.swap(false, Ordering::AcqRel) {
                        break;
                    }
                }
                running.store(false, Ordering::Release);
            });
        })
    };

    // Search semantics parity with the pre-lazy behavior: matching runs
    // over the loaded details instantly, while the REST of the user's
    // phrases stream in once per session — results grow as chunks land,
    // and the indicator tells the user the search is not final yet.
    {
        let all_ids = all_phrase_ids;
        let backfill_done_sig = backfill_done;
        let loading_sig = search_backfill_loading;
        let refresh = refresh_trigger;
        let search_sig = search;
        Effect::new(move |_| {
            let query = search_sig.get();
            if query.trim().is_empty()
                || backfill_done_sig.get_untracked()
                || loading_sig.get_untracked()
            {
                return;
            }
            let missing = missing_details(&all_ids.get_untracked());
            if missing.is_empty() {
                backfill_done_sig.set(true);
                return;
            }
            loading_sig.set(true);
            spawn_local(async move {
                tracing::info!(
                    phrases = missing.len(),
                    "Backfilling all phrase details for search"
                );
                load_and_refresh(missing, refresh).await;
                loading_sig.set(false);
                backfill_done_sig.set(true);
            });
        });
    }

    let ctx = create_card_list_context(
        repository,
        refresh_trigger,
        |card| matches!(card, Card::Phrase(_)),
        Some(on_cards_loaded),
    );

    let ctx_for_render = ctx.clone();
    let empty_message =
        Signal::derive(move || td_string!(i18n.get_locale(), phrases.not_found).to_string());
    let partial_note =
        move || td_string!(i18n.get_locale(), phrases.search_in_progress).to_string();

    let list = card_list_view(
        ctx,
        ListGrouping::Flat,
        "phrases",
        empty_message,
        Some(
            "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-4 items-start",
        ),
        CardListExtras {
            search: Some(search),
            on_visible_cards: Some(on_visible_cards),
        },
        move |card| {
            let ctx = ctx_for_render.clone();
            let card_id = *card.card_id();
            view! {
                <PhraseCardItem
                    study_card=card
                    native_language=ctx.native_lang
                    known_kanji=ctx.known_kanji.get()
                    on_toggle_favorite=ctx.on_toggle_favorite
                    on_mark_as_known=Callback::new(move |_| ctx.on_mark_as_known.run(card_id))
                    on_delete=ctx.on_delete
                    is_deleting=ctx.is_deleting
                    phrase_data_trigger=refresh_trigger
                />
            }
            .into_any()
        },
    );

    view! {
        <div class="space-y-4">
            <Show when=move || {
                !search.get().trim().is_empty() && search_backfill_loading.get()
            }>
                <div
                    class="text-sm text-muted-foreground animate-pulse"
                    data-testid="phrases-search-partial"
                >
                    {partial_note}
                </div>
            </Show>
            {list}
        </div>
    }
    .into_any()
}
