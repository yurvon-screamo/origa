use std::sync::Arc;

use super::super::shared::{
    BackfillState, CardListExtras, CardListViewConfig, CardsLoadedCallback, LazyDetailHooks,
    ListGrouping, ListPage, ListUiStore, SearchMatcher, card_list_view, create_card_list_context,
    create_search_backfill_effect, create_visible_slice_loader,
};
use super::example_card_item::ExampleCardItem;
use super::lazy_details::{load_and_refresh, missing_details, sentence_ids_of};
use crate::i18n::{td_string, use_i18n};
use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use origa::dictionary::example::get_cached_example_detail;
use origa::domain::{Card, NativeLanguage, StudyCard};

#[component]
pub fn ExamplesContent(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    // Data-lazy like the phrases page (#540-В1): the full sentence-id list
    // is only recorded (for the search-driven backfill); details load per
    // visible slice.
    let all_sentence_ids: RwSignal<Vec<u32>> = RwSignal::new(Vec::new());
    // The search box lives in the session store (same slot `card_list_view`
    // reads), so a query survives navigating away and back.
    let search = use_context::<ListUiStore>()
        .expect("ListUiStore not provided")
        .slot(ListPage::Examples)
        .search;
    let search_backfill_loading: RwSignal<bool> = RwSignal::new(false);
    let backfill_done: RwSignal<bool> = RwSignal::new(false);

    let on_cards_loaded: CardsLoadedCallback = Arc::new(move |cards: &[StudyCard]| {
        all_sentence_ids.set(sentence_ids_of(cards));
    });

    let hooks = LazyDetailHooks {
        ids_of: Arc::new(sentence_ids_of),
        missing_of: Arc::new(missing_details),
        load_batch: Arc::new(move |ids| {
            Box::pin(async move { load_and_refresh(ids, refresh_trigger).await })
        }),
    };

    let on_visible_cards = create_visible_slice_loader(search_backfill_loading, hooks.clone());

    create_search_backfill_effect(
        BackfillState {
            search,
            all_ids: all_sentence_ids,
            done: backfill_done,
            loading: search_backfill_loading,
        },
        "examples",
        hooks,
    );

    let ctx = create_card_list_context(
        repository,
        refresh_trigger,
        |card| matches!(card, Card::Example(_)),
        Some(on_cards_loaded),
    );

    let ctx_for_render = ctx.clone();
    let empty_message =
        Signal::derive(move || td_string!(i18n.get_locale(), examples.not_found).to_string());
    let partial_note =
        move || td_string!(i18n.get_locale(), examples.search_in_progress).to_string();

    // The domain Card for an example carries only the word surface (the
    // sentence is CDN data), so search matches the word plus the lazily
    // loaded sentence text/translation through the details cache.
    let search_match: SearchMatcher = Arc::new(
        |card: &StudyCard, query: &str, lang: NativeLanguage| match card.card() {
            Card::Example(ec) => {
                if ec.word().to_lowercase().contains(query) {
                    return true;
                }
                get_cached_example_detail(ec.sentence_id()).is_some_and(|detail| {
                    detail.text.to_lowercase().contains(query)
                        || detail
                            .translation(&lang)
                            .is_some_and(|t| t.to_lowercase().contains(query))
                })
            },
            _ => false,
        },
    );

    let config = CardListViewConfig {
        test_id_prefix: "examples",
        empty_message,
        grid_classes: Some(
            "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-4 items-start",
        ),
    };
    let list = card_list_view(
        ctx,
        ListPage::Examples,
        ListGrouping::Flat,
        config,
        CardListExtras {
            search: Some(search),
            on_visible_cards: Some(on_visible_cards),
            search_match: Some(search_match),
        },
        move |card| {
            let ctx = ctx_for_render.clone();
            view! {
                <ExampleCardItem
                    study_card=card
                    native_language=ctx.native_lang
                    actions=ctx.actions()
                    details_trigger=refresh_trigger
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
                    data-testid="examples-search-partial"
                >
                    {partial_note}
                </div>
            </Show>
            {list}
        </div>
    }
    .into_any()
}
