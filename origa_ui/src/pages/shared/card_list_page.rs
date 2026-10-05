use std::collections::HashSet;
use std::sync::Arc;

use super::{
    CardCounts, CardStatus, DEFAULT_VISIBLE_COUNT, DeleteRequest, Filter, FilterBtn, FirstRunGuard,
    GroupedGrid, JlptCounts, JlptFilter, JlptFilterBtn, LevelIndex, ListGrouping, ListPage,
    ListUiStore, LoadMoreButton, create_delete_callback, create_mark_as_known_callback,
    create_toggle_favorite_callback, jlpt_level_idx, order_cards_by_group,
    restore_scroll_when_ready, track_scroll_while_authenticated,
};
use crate::i18n::use_i18n;
use crate::loaders::get_jlpt_content;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Input, LoadingOverlay, Text, TextSize, ToastContainer, ToastData, TypographyVariant,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, CardAnswer, JapaneseLevel, NativeLanguage, StudyCard, User};
use origa::traits::UserRepository;
use std::collections::HashMap;

pub type CardsLoadedCallback = Arc<dyn Fn(&[StudyCard]) + Send + Sync>;

/// Callback for the visible (rendered) card slice — the lazy data-load
/// trigger of #540-В1.
pub type VisibleCardsCallback = Arc<dyn Fn(&[StudyCard]) + Send + Sync>;

/// Page-specific hooks into the shared list view (#540-В1).
///
/// The phrases page is data-lazy: unlike the other pages it must NOT load
/// every phrase's details up front, so it (a) injects its own search
/// signal to observe queries and (b) subscribes to the currently visible
/// card slice to load details for what is actually on screen. Every other
/// page passes the default (no overrides) and behaves exactly as before.
#[derive(Default)]
pub struct CardListExtras {
    /// Use this signal as the list's search box state instead of a local
    /// one — lets the page observe queries (deferred full load).
    pub search: Option<RwSignal<String>>,
    /// Called whenever the visible (filtered + paginated) card slice
    /// changes — the lazy data-load trigger.
    pub on_visible_cards: Option<VisibleCardsCallback>,
}

#[derive(Clone)]
pub struct CardListContext {
    #[expect(
        dead_code,
        reason = "read via closure capture in toggle_favorite_callback"
    )]
    pub current_user: RwSignal<Option<User>>,
    pub native_lang: Memo<NativeLanguage>,
    pub known_kanji: Memo<HashSet<char>>,
    pub on_toggle_favorite: Callback<ulid::Ulid>,
    pub on_mark_as_known: Callback<ulid::Ulid>,
    pub on_delete: Callback<DeleteRequest>,
    pub is_deleting: Signal<bool>,
    pub toasts: RwSignal<Vec<ToastData>>,
    pub all_cards: RwSignal<Vec<StudyCard>>,
    pub is_loading: RwSignal<bool>,
}

pub fn create_card_list_context(
    repository: HybridUserRepository,
    refresh_trigger: RwSignal<u32>,
    card_type_filter: fn(&Card) -> bool,
    on_cards_loaded: Option<CardsLoadedCallback>,
) -> CardListContext {
    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let is_loading = RwSignal::new(true);
    let all_cards: RwSignal<Vec<StudyCard>> = RwSignal::new(Vec::new());
    let repo_for_effect = repository.clone();

    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let repo = repo_for_effect.clone();
        let on_loaded = on_cards_loaded.clone();
        spawn_local(async move {
            match repo.get_current_user().await {
                Ok(Some(user)) => {
                    let disposed = StoredValue::new(());
                    if disposed.is_disposed() {
                        return;
                    }
                    let cards: Vec<StudyCard> = user
                        .knowledge_set()
                        .study_cards()
                        .iter()
                        .filter(|(_, card)| card_type_filter(card.card()))
                        .map(|(_, card)| card.clone())
                        .collect();

                    if let Some(on_loaded) = on_loaded.as_ref() {
                        on_loaded(&cards);
                    }

                    if disposed.is_disposed() {
                        return;
                    }
                    all_cards.set(cards);
                    current_user.set(Some(user));
                    is_loading.set(false);
                },
                Ok(None) => {
                    tracing::warn!("CardListPage: user not found");
                    is_loading.set(false);
                },
                Err(e) => {
                    tracing::error!("CardListPage: get_current_user error: {:?}", e);
                    is_loading.set(false);
                },
            }
        });
    });

    let i18n = use_i18n();
    let native_lang =
        Memo::new(move |_| crate::i18n::locale_to_native_language(&i18n.get_locale()));

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let (on_toggle_favorite, _favorite_pending) =
        create_toggle_favorite_callback(repository.clone(), current_user, refresh_trigger);

    let (on_mark_as_known, _mark_known_pending) =
        create_mark_as_known_callback(repository.clone(), refresh_trigger);

    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let (is_deleting, on_delete) =
        create_delete_callback(repository.clone(), toasts, refresh_trigger);

    CardListContext {
        current_user,
        native_lang,
        known_kanji,
        on_toggle_favorite,
        on_mark_as_known,
        on_delete,
        is_deleting: is_deleting.into(),
        toasts,
        all_cards,
        is_loading,
    }
}

/// Presentation config of a list view: the test-id prefix the page's
/// hooks are keyed by, the localized empty-state message and an optional
/// grid class override (flat pages pass their grid, grouped pages may
/// rely on the default).
pub struct CardListViewConfig {
    pub test_id_prefix: &'static str,
    pub empty_message: Signal<String>,
    pub grid_classes: Option<&'static str>,
}

pub fn card_list_view<F>(
    ctx: CardListContext,
    page: ListPage,
    grouping: ListGrouping,
    config: CardListViewConfig,
    extras: CardListExtras,
    render_card: F,
) -> AnyView
where
    F: Fn(StudyCard) -> AnyView + Clone + Send + Sync + 'static,
{
    let i18n = use_i18n();
    let is_loading = ctx.is_loading;
    let all_cards = ctx.all_cards;
    let native_lang = ctx.native_lang;
    let toasts = ctx.toasts;
    let CardListViewConfig {
        test_id_prefix,
        empty_message,
        grid_classes,
    } = config;

    // Filter/pagination signals come from the session store: they survive
    // navigating to a detail page and back, so the list reopens exactly as
    // the user left it. `extras.search` stays as the page-provided override
    // (phrases injects its own signal to observe queries).
    let list_ui = use_context::<ListUiStore>().expect("ListUiStore not provided");
    let slot = list_ui.slot(page);
    let search = extras.search.unwrap_or(slot.search);
    let filter = slot.status;
    let jlpt_filter = slot.jlpt;
    let visible_count = slot.visible_count;

    // Build the card_id -> JLPT level lookup once when grouping is enabled.
    // Computed only when `all_cards` changes — NOT on every filter/search tick.
    // For `ListGrouping::Flat` we skip the work entirely (empty map).
    let level_index: Memo<LevelIndex> = match grouping {
        ListGrouping::Flat => Memo::new(move |_| HashMap::new()),
        ListGrouping::ByJlptLevel { card_type } => Memo::new(move |_| {
            let content = get_jlpt_content();
            all_cards
                .get()
                .iter()
                .map(|card| {
                    (
                        *card.card_id(),
                        content.find_level(&card.card().content_key(), card_type),
                    )
                })
                .collect()
        }),
    };

    let filtered_cards = Memo::new(move |_| {
        let query = search.get().to_lowercase();
        let current_filter = filter.get();
        let current_jlpt = jlpt_filter.get();
        let index = level_index.get();
        let lang = native_lang.get();

        let mut cards: Vec<_> = all_cards
            .get()
            .into_iter()
            .filter(|card| {
                let matches_search = query.is_empty() || {
                    let card_inner = card.card();
                    let question = card_inner.question(&lang);

                    let matches_question = question
                        .ok()
                        .is_some_and(|q| q.text().to_lowercase().contains(&query));

                    let matches_answer = match card_inner.answer(&lang).ok() {
                        Some(CardAnswer::Vocabulary {
                            translations,
                            description,
                        }) => {
                            translations
                                .iter()
                                .any(|t| t.to_lowercase().contains(&query))
                                || description
                                    .as_ref()
                                    .is_some_and(|d| d.to_lowercase().contains(&query))
                        },
                        Some(CardAnswer::Text(s)) => s.to_lowercase().contains(&query),
                        Some(CardAnswer::GrammarNuances { .. }) => false,
                        None => false,
                    };

                    matches_question || matches_answer
                };
                let matches_filter =
                    current_filter.matches(CardStatus::from_study_card(card), card.is_favorite());
                // JLPT axis — AND with status + search. For Flat pages
                // `level_index` is empty, so `index.get(...)` is None and
                // `JlptFilter::All` (default) still matches via the All arm.
                let card_level = index.get(card.card_id()).and_then(|l| *l);
                let matches_jlpt = current_jlpt.matches(card_level);
                matches_search && matches_filter && matches_jlpt
            })
            .collect();

        match grouping {
            ListGrouping::ByJlptLevel { .. } => {
                // `index` was captured once above; reuse it for sort instead
                // of calling `level_index.get()` again inside this branch.
                cards = order_cards_by_group(&cards, &index);
            },
            // Flat pages render in stable card-id order; grouped pages
            // are ordered by their group axis instead (both arms kept
            // their previous unconditional behavior — every caller passed
            // the same flag).
            ListGrouping::Flat => cards.sort_by_key(|c| *c.card_id()),
        }
        cards
    });

    let jlpt_counts = Memo::new(move |_| {
        let index = level_index.get();
        let mut counts = JlptCounts::default();
        for card in all_cards.get().iter() {
            match index.get(card.card_id()).and_then(|l| *l) {
                Some(level) => counts.by_level[jlpt_level_idx(level)] += 1,
                None => counts.other += 1,
            }
            counts.total += 1;
        }
        counts
    });

    // A filter/search change restarts the rendered slice from the first
    // page — but not on mount: the effect always runs once on creation,
    // and that first run would clobber the visible_count restored from
    // the store right after remounting the page.
    let visible_count_reset = FirstRunGuard::new();
    Effect::new(move |_| {
        let _ = search.get();
        let _ = filter.get();
        let _ = jlpt_filter.get();
        if visible_count_reset.should_run() {
            visible_count.set(DEFAULT_VISIBLE_COUNT);
        }
    });

    let visible_cards = Memo::new(move |_| {
        filtered_cards
            .get()
            .into_iter()
            .take(visible_count.get())
            .collect::<Vec<_>>()
    });

    // Scroll persistence: track the position as the user scrolls (saving on
    // unmount is unreliable — the browser clamps the scroll the moment the
    // old page's DOM is detached), restore it once per mount after the
    // rendered slice is on screen. The restored visible_count above makes
    // the document tall enough for the saved position to land on.
    track_scroll_while_authenticated(&slot);
    restore_scroll_when_ready(&slot, is_loading, move || !visible_cards.get().is_empty());

    // Lazy-load hook (#540-В1): notify the page whenever the rendered
    // slice changes (first render, "load more", filter/search switches).
    // Runs after the memo settled; the page's callback decides what to
    // fetch. Unsubscribed pages pay nothing.
    if let Some(on_visible_cards) = extras.on_visible_cards {
        Effect::new(move |_| {
            let cards = visible_cards.get();
            on_visible_cards(&cards);
        });
    }

    let counts = Memo::new(move |_| {
        let cards = all_cards.get();
        cards.iter().fold(CardCounts::default(), |mut acc, card| {
            acc.total += 1;
            if card.is_favorite() {
                acc.favorite += 1;
            }
            match CardStatus::from_study_card(card) {
                CardStatus::New => acc.new += 1,
                CardStatus::Hard => acc.hard += 1,
                CardStatus::InProgress => acc.in_progress += 1,
                CardStatus::Learned => acc.learned += 1,
            }
            acc
        })
    });

    let grid_id = Signal::derive(move || format!("{test_id_prefix}-grid"));
    let empty_id = Signal::derive(move || format!("{test_id_prefix}-empty-state"));
    let search_id = Signal::derive(move || format!("{test_id_prefix}-search-input"));
    let load_more_id = Signal::derive(move || format!("{test_id_prefix}-load-more-btn"));
    let render_card = StoredValue::new(render_card);

    let flat_grid_classes = grid_classes.unwrap_or(
        "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4 items-start",
    );

    view! {
        <div class="space-y-4">
            <Show when=move || is_loading.get()>
                <LoadingOverlay message=Signal::derive(move || i18n.get_keys().common().loading().inner().to_string()) />
            </Show>
            <Show when=move || !is_loading.get()>
                <Input
                    value=search
                    placeholder=Signal::derive(move || i18n.get_keys().common().search().inner().to_string())
                    test_id=search_id
                />

                <div class="flex flex-wrap gap-2">
                    <FilterBtn filter=Filter::All count=move || counts.get().total active=filter test_id=format!("{test_id_prefix}-filter-all") />
                    <FilterBtn filter=Filter::New count=move || counts.get().new active=filter test_id=format!("{test_id_prefix}-filter-new") />
                    <FilterBtn filter=Filter::Hard count=move || counts.get().hard active=filter test_id=format!("{test_id_prefix}-filter-hard") />
                    <FilterBtn filter=Filter::InProgress count=move || counts.get().in_progress active=filter test_id=format!("{test_id_prefix}-filter-in-progress") />
                    <FilterBtn filter=Filter::Learned count=move || counts.get().learned active=filter test_id=format!("{test_id_prefix}-filter-learned") />
                    <FilterBtn filter=Filter::Favorite count=move || counts.get().favorite active=filter test_id=format!("{test_id_prefix}-filter-favorite") />
                </div>

                {matches!(grouping, ListGrouping::ByJlptLevel { .. }).then(|| {
                    view! {
                        <div class="flex flex-wrap gap-2 mt-2">
                            <JlptFilterBtn
                                filter=JlptFilter::All
                                count=move || jlpt_counts.get().total
                                active=jlpt_filter
                                test_id=format!("{test_id_prefix}-filter-jlpt-all")
                            />
                            <For
                                each=|| JapaneseLevel::ALL.to_vec()
                                key=|level: &JapaneseLevel| *level
                                children=move |level| {
                                    let counts_for_btn = jlpt_counts;
                                    view! {
                                        <JlptFilterBtn
                                            filter=JlptFilter::Level(level)
                                            count=move || counts_for_btn.get().level_count(level)
                                            active=jlpt_filter
                                            test_id=format!(
                                                "{test_id_prefix}-filter-jlpt-{}",
                                                level.code().to_lowercase()
                                            )
                                        />
                                    }
                                }
                            />
                            // "No level" is an impossible state for
                            // JLPT-indexed content (every kanji/grammar
                            // card has a level); the chip renders only
                            // when levelless cards actually exist.
                            <Show when=move || { jlpt_counts.get().other > 0 }>
                                <JlptFilterBtn
                                    filter=JlptFilter::Other
                                    count=move || jlpt_counts.get().other
                                    active=jlpt_filter
                                    test_id=format!("{test_id_prefix}-filter-jlpt-other")
                                />
                            </Show>
                        </div>
                    }
                })}

                {match grouping {
                    ListGrouping::Flat => view! {
                        <div class=flat_grid_classes data-testid=move || grid_id.get()>
                            {move || {
                                let cards = visible_cards.get();
                                if cards.is_empty() {
                                    Either::Left(view! {
                                        <div class="col-span-full" data-testid=move || empty_id.get()>
                                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                                {empty_message.get()}
                                            </Text>
                                        </div>
                                    })
                                } else {
                                    Either::Right(view! {
                                        <For
                                            each=move || visible_cards.get()
                                            key=|card| format!("{}-{}", card.card_id(), card.is_favorite())
                                            children=move |card| {
                                                let render = render_card.with_value(|r| r.clone());
                                                render(card)
                                            }
                                        />
                                    })
                                }
                            }}
                        </div>
                    }
                    .into_any(),
                    ListGrouping::ByJlptLevel { .. } => view! {
                        <div data-testid=move || grid_id.get() class="space-y-8">
                            {move || {
                                let cards = visible_cards.get();
                                if cards.is_empty() {
                                    Either::Left(view! {
                                        <div data-testid=move || empty_id.get()>
                                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                                {empty_message.get()}
                                            </Text>
                                        </div>
                                    })
                                } else {
                                    let render = render_card.with_value(|r| r.clone());
                                    Either::Right(view! {
                                        <GroupedGrid
                                            cards=visible_cards
                                            level_index=level_index
                                            grid_classes=flat_grid_classes
                                            test_id_prefix=test_id_prefix
                                            render_card=render
                                        />
                                    })
                                }
                            }}
                        </div>
                    }
                    .into_any(),
                }}

                <LoadMoreButton
                    visible_count=visible_count
                    total=Signal::derive(move || filtered_cards.get().len())
                    test_id=load_more_id
                />
                <ToastContainer toasts=toasts duration_ms=5000 />
            </Show>
        </div>
    }
    .into_any()
}
