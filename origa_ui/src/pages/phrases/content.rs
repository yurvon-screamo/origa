use std::sync::Arc;

use super::super::shared::{
    BackfillState, CardListExtras, CardListViewConfig, CardsLoadedCallback, LazyDetailHooks,
    ListGrouping, ListPage, ListUiStore, card_list_view, create_card_list_context,
    create_search_backfill_effect, create_visible_slice_loader,
};
use super::lazy_details::{load_and_refresh, missing_details, phrase_ids_of};
use super::phrase_card_item::PhraseCardItem;
use crate::i18n::{td_string, use_i18n};
use crate::repository::HybridUserRepository;
use leptos::prelude::*;
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
    // The search box lives in the session store (same slot `card_list_view`
    // reads), so a query survives navigating away and back. The backfill
    // effect below observes it through this signal.
    let search = use_context::<ListUiStore>()
        .expect("ListUiStore not provided")
        .slot(ListPage::Phrases)
        .search;
    let search_backfill_loading: RwSignal<bool> = RwSignal::new(false);
    let backfill_done: RwSignal<bool> = RwSignal::new(false);

    let on_cards_loaded: CardsLoadedCallback = Arc::new(move |cards: &[StudyCard]| {
        all_phrase_ids.set(phrase_ids_of(cards));
    });

    let hooks = LazyDetailHooks {
        ids_of: Arc::new(phrase_ids_of),
        missing_of: Arc::new(missing_details),
        load_batch: Arc::new(move |ids| {
            Box::pin(async move { load_and_refresh(ids, refresh_trigger).await })
        }),
    };

    // Visible-slice lazy load: runs on first render, "load more" and
    // filter/search switches (shared machinery, see lazy_card_details).
    let on_visible_cards = create_visible_slice_loader(search_backfill_loading, hooks.clone());

    // Search semantics parity with the pre-lazy behavior: matching runs
    // over the loaded details instantly, while the REST of the user's
    // phrases load once per session. Deviation from the planned
    // chunk-by-chunk streaming (#540-В1): the backfill is ONE parallel
    // batch — search results jump to complete when it lands (the browser
    // serializes the fetches over its connection pool anyway), and the
    // indicator stays visible for the whole load.
    create_search_backfill_effect(
        BackfillState {
            search,
            all_ids: all_phrase_ids,
            done: backfill_done,
            loading: search_backfill_loading,
        },
        "phrases",
        hooks,
    );

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

    let config = CardListViewConfig {
        test_id_prefix: "phrases",
        empty_message,
        grid_classes: Some(
            "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-4 items-start",
        ),
    };
    let list = card_list_view(
        ctx,
        ListPage::Phrases,
        ListGrouping::Flat,
        config,
        CardListExtras {
            search: Some(search),
            on_visible_cards: Some(on_visible_cards),
            search_match: None,
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
