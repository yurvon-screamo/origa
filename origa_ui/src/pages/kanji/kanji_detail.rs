use std::collections::HashSet;

use super::super::shared::{
    CardStatus, DeleteRequest, create_delete_callback, create_mark_as_known_callback,
    create_toggle_favorite_callback,
};
use crate::i18n::use_i18n;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    CardActionBar, CardHistoryModal, DeleteConfirmModal, KanjiDrawingPractice, KanjiViewMode,
    KanjiWritingSection, LoadingOverlay, MarkdownText, PageHeader, TabItem, Tabs, Text, TextSize,
    TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_params_map};
use origa::domain::{Card as DomainCard, StudyCard, User};
use origa::traits::UserRepository;
use ulid::Ulid;

fn load_study_card(
    repository: HybridUserRepository,
    card_id: Ulid,
    result_signal: RwSignal<Option<StudyCard>>,
    current_user: RwSignal<Option<User>>,
    is_loading: RwSignal<bool>,
) {
    let disposed = StoredValue::new(());
    spawn_local(async move {
        match repository.get_current_user().await {
            Ok(Some(user)) => {
                if disposed.is_disposed() {
                    return;
                }
                let found = user
                    .knowledge_set()
                    .study_cards()
                    .iter()
                    .find(|(id, _)| **id == card_id)
                    .map(|(_, card)| card.clone());
                result_signal.set(found);
                current_user.set(Some(user));
                is_loading.set(false);
            },
            Ok(None) => {
                if disposed.is_disposed() {
                    return;
                }
                tracing::warn!("KanjiDetail: user not found");
                is_loading.set(false);
            },
            Err(e) => {
                if disposed.is_disposed() {
                    return;
                }
                tracing::error!("KanjiDetail: get_current_user error: {:?}", e);
                is_loading.set(false);
            },
        }
    });
}

#[component]
pub fn KanjiDetail() -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let params = use_params_map();
    let card_id_result: Memo<Option<Ulid>> = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<Ulid>().ok())
    });

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let study_card: RwSignal<Option<StudyCard>> = RwSignal::new(None);
    let is_loading = RwSignal::new(true);
    let refresh_trigger = RwSignal::new(0u32);

    let repo_for_init = repository.clone();
    Effect::new(move |_| {
        let Some(card_id) = card_id_result.get() else {
            is_loading.set(false);
            return;
        };
        load_study_card(
            repo_for_init.clone(),
            card_id,
            study_card,
            current_user,
            is_loading,
        );
    });

    let repo_for_refresh = repository.clone();
    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let Some(card_id) = card_id_result.get() else {
            return;
        };
        load_study_card(
            repo_for_refresh.clone(),
            card_id,
            study_card,
            current_user,
            is_loading,
        );
    });

    let on_toggle_favorite =
        create_toggle_favorite_callback(repository.clone(), current_user, refresh_trigger);
    let on_mark_as_known = create_mark_as_known_callback(repository.clone(), refresh_trigger);
    let toasts: RwSignal<Vec<crate::ui_components::ToastData>> = RwSignal::new(Vec::new());
    let (is_deleting, on_delete) =
        create_delete_callback(repository.clone(), toasts, refresh_trigger);

    let native_lang =
        Memo::new(move |_| crate::i18n::locale_to_native_language(&i18n.get_locale()));

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let is_delete_modal_open = RwSignal::new(false);
    let is_history_open = RwSignal::new(false);
    let navigate = StoredValue::new(use_navigate());

    let back_label = Signal::derive(move || i18n.get_keys().home().kanji().inner().to_string());

    let not_found_text =
        Signal::derive(move || i18n.get_keys().kanji_page().not_found().inner().to_string());

    let loading_text =
        Signal::derive(move || i18n.get_keys().common().loading().inner().to_string());

    let active_tab: RwSignal<String> = RwSignal::new("overview".to_string());
    let tab_items = Signal::derive(|| {
        vec![
            TabItem {
                id: "overview".to_string(),
                label: "Overview".to_string(),
            },
            TabItem {
                id: "writing".to_string(),
                label: "Writing".to_string(),
            },
            TabItem {
                id: "stats".to_string(),
                label: "Stats".to_string(),
            },
        ]
    });

    view! {
        <PageHeader
            back_path=Signal::derive(|| "/kanji".to_string())
            back_label=back_label
            test_id=Signal::derive(|| "kanji-detail".to_string())
        />

        <Show when=move || is_loading.get()>
            <LoadingOverlay message=loading_text />
        </Show>

        <Show when=move || !is_loading.get() && study_card.get().is_none()>
            <div class="flex items-center justify-center py-16">
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    {not_found_text}
                </Text>
            </div>
        </Show>

        <Show when=move || study_card.get().is_some()>
            {move || {
                let card = study_card.get()?;
                let card_id = *card.card_id();
                let is_favorite = card.is_favorite();
                let memory = card.memory().clone();

                let status = CardStatus::from_study_card(&card);

                let (kanji_char, radicals) = match card.card() {
                    DomainCard::Kanji(kanji_card) => (
                        kanji_card.kanji().text().to_string(),
                        kanji_card.radicals_chars().into_iter().collect::<String>(),
                    ),
                    _ => ("?".to_string(), String::new()),
                };

                let card_for_desc = card.clone();
                let description = Memo::new(move |_| {
                    let lang = native_lang.get();
                    match card_for_desc.card() {
                        DomainCard::Kanji(kanji_card) => kanji_card
                            .description(&lang)
                            .ok()
                            .map(|d| d.text().to_string())
                            .unwrap_or_default(),
                        _ => String::new(),
                    }
                });

                let card_for_answer = card.clone();
                let answer_text = Memo::new(move |_| {
                    let lang = native_lang.get();
                    card_for_answer
                        .card()
                        .answer(&lang)
                        .map(|a| a.text().to_string())
                        .unwrap_or_default()
                });

                let card_for_examples = card.clone();
                let example_words = Memo::new(move |_| {
                    let lang = native_lang.get();
                    match card_for_examples.card() {
                        DomainCard::Kanji(kanji_card) => kanji_card
                            .example_words(&lang)
                            .iter()
                            .map(|w| (w.word().to_string(), w.meaning().to_string()))
                            .collect::<Vec<_>>(),
                        _ => Vec::new(),
                    }
                });

                let has_radicals = !radicals.is_empty();
                let radicals_stored: StoredValue<String> = StoredValue::new(radicals.clone());
                let has_examples = Memo::new(move |_| !example_words.get().is_empty());

                let difficulty = memory
                    .difficulty()
                    .map(|d| format!("{:.1}", d.value()))
                    .unwrap_or("-".to_string());
                let stability = memory
                    .stability()
                    .map(|s| format!("{:.1}", s.value()))
                    .unwrap_or("-".to_string());
                let next_review = memory
                    .next_review_date()
                    .map(|d| d.format("%d.%m.%Y").to_string())
                    .unwrap_or("-".to_string());

                let memory_history: StoredValue<origa::domain::MemoryHistory> =
                    StoredValue::new(memory.clone());

                let card_id_for_delete = card_id;
                let confirm_delete = Callback::new(move |_| {
                    on_delete.run(DeleteRequest {
                        card_id: card_id_for_delete,
                        on_success: Callback::new(move |_| {
                            is_delete_modal_open.set(false);
                            navigate.get_value()("/kanji", Default::default());
                        }),
                    })
                });

                let kanji_stored: StoredValue<String> = StoredValue::new(kanji_char.clone());
                let card_id_for_known = card_id;
                let card_id_for_fav = card_id;

                let radicals_heading = Signal::derive(move || {
                    i18n.get_keys()
                        .shared()
                        .radicals_label()
                        .inner()
                        .to_string()
                        .replacen("{}", "", 1)
                });

                let examples_heading = Signal::derive(move || {
                    i18n.get_keys()
                        .shared()
                        .examples_label()
                        .inner()
                        .to_string()
                        .replacen("{}", "", 1)
                });

                let stats_text = Signal::derive(move || {
                    i18n.get_keys()
                        .shared()
                        .card_info()
                        .inner()
                        .to_string()
                        .replacen("{}", &next_review, 1)
                        .replacen("{}", &difficulty, 1)
                        .replacen("{}", &stability, 1)
                });

                let active_tab_cell = active_tab;

                Some(view! {
                    // Desktop layout (current 2-column)
                    <div class="kanji-detail-layout kanji-detail-desktop">
                        <div class="kanji-detail-left">
                            <KanjiDetailHero
                                kanji_stored=kanji_stored
                                answer_text=answer_text
                                has_radicals=has_radicals
                                radicals_stored=radicals_stored
                            />

                            <CardActionBar
                                tag_variant=Signal::derive(move || status.tag_variant())
                                tag_label=Signal::derive(move || status.label(&i18n))
                                is_favorite=Signal::derive(move || is_favorite)
                                on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id_for_fav))
                                show_mark_as_known=Signal::derive(move || status != CardStatus::Learned)
                                on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(card_id_for_known))
                                on_history=Callback::new(move |_| is_history_open.set(true))
                                on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                                test_id=Signal::derive(|| "kanji-detail-actions".to_string())
                            />

                            <div class="kanji-detail-section">
                                <KanjiWritingSection
                                    kanji=kanji_stored.get_value()
                                    mode=KanjiViewMode::Frames
                                />
                            </div>

                            <div class="kanji-detail-section">
                                <KanjiDrawingPractice kanji=kanji_stored.get_value() />
                            </div>
                        </div>

                        <KanjiDetailRightColumn
                            description=description
                            has_radicals=has_radicals
                            radicals_stored=radicals_stored
                            radicals_heading=radicals_heading
                            has_examples=has_examples
                            examples_heading=examples_heading
                            example_words=example_words
                            stats_text=stats_text
                            known_kanji=known_kanji.get()
                        />
                    </div>

                    // Mobile layout (hero + tabs)
                    <div class="kanji-detail-mobile">
                        <KanjiDetailHero
                            kanji_stored=kanji_stored
                            answer_text=answer_text
                            has_radicals=has_radicals
                            radicals_stored=radicals_stored
                        />

                        <CardActionBar
                            tag_variant=Signal::derive(move || status.tag_variant())
                            tag_label=Signal::derive(move || status.label(&i18n))
                            is_favorite=Signal::derive(move || is_favorite)
                            on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id_for_fav))
                            show_mark_as_known=Signal::derive(move || status != CardStatus::Learned)
                            on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(card_id_for_known))
                            on_history=Callback::new(move |_| is_history_open.set(true))
                            on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                            test_id=Signal::derive(|| "kanji-detail-actions-mobile".to_string())
                        />

                        <div class="kanji-detail-section">
                            <Tabs
                                tabs=tab_items
                                active=active_tab_cell
                                test_id=Signal::derive(|| "kanji-detail-tabs".to_string())
                                class="tabs--underline".to_string()
                            />
                        </div>

                        <Show when=move || active_tab_cell.get() == "overview">
                            <KanjiDetailRightColumn
                                description=description
                                has_radicals=has_radicals
                                radicals_stored=radicals_stored
                                radicals_heading=radicals_heading
                                has_examples=has_examples
                                examples_heading=examples_heading
                                example_words=example_words
                                stats_text=stats_text
                                known_kanji=known_kanji.get()
                            />
                        </Show>

                        <Show when=move || active_tab_cell.get() == "writing">
                            <div class="kanji-detail-section">
                                <KanjiWritingSection
                                    kanji=kanji_stored.get_value()
                                    mode=KanjiViewMode::Frames
                                />
                            </div>

                            <div class="kanji-detail-section">
                                <KanjiDrawingPractice kanji=kanji_stored.get_value() />
                            </div>
                        </Show>

                        <Show when=move || active_tab_cell.get() == "stats">
                            <div class="kanji-detail-section">
                                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                    {stats_text}
                                </Text>
                            </div>
                        </Show>
                    </div>

                    <CardHistoryModal
                        is_open=Signal::derive(move || is_history_open.get())
                        memory=memory_history.get_value()
                        on_close=Callback::new(move |_| is_history_open.set(false))
                    />
                    <DeleteConfirmModal
                        test_id="kanji-detail-delete-modal"
                        is_open=is_delete_modal_open
                        is_deleting=is_deleting.into()
                        on_confirm=confirm_delete
                        on_close=Callback::new(move |_| is_delete_modal_open.set(false))
                    />
                }.into_any())
            }}
        </Show>
    }
}

#[component]
fn KanjiDetailHero(
    kanji_stored: StoredValue<String>,
    answer_text: Memo<String>,
    has_radicals: bool,
    radicals_stored: StoredValue<String>,
) -> impl IntoView {
    view! {
        <div class="kanji-detail-hero">
            <div class="kanji-detail-hero-kanji">
                {kanji_stored.get_value()}
            </div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                {answer_text}
            </Text>
            <Show when=move || has_radicals>
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {radicals_stored.get_value()}
                </Text>
            </Show>
        </div>
    }
}

#[component]
fn KanjiDetailRightColumn(
    description: Memo<String>,
    has_radicals: bool,
    radicals_stored: StoredValue<String>,
    radicals_heading: Signal<String>,
    has_examples: Memo<bool>,
    examples_heading: Signal<String>,
    example_words: Memo<Vec<(String, String)>>,
    stats_text: Signal<String>,
    known_kanji: HashSet<char>,
) -> impl IntoView {
    view! {
        <div class="kanji-detail-right">
            <Show when=move || !description.get().is_empty()>
                <div class="kanji-detail-section">
                    <MarkdownText
                        content=Signal::derive(move || description.get())
                        known_kanji=known_kanji.clone()
                    />
                </div>
            </Show>

            <Show when=move || has_radicals>
                <div class="kanji-detail-section">
                    <div class="kanji-detail-section-title">
                        {radicals_heading}
                    </div>
                    <Text size=TextSize::Default variant=TypographyVariant::Primary>
                        {radicals_stored.get_value()}
                    </Text>
                </div>
            </Show>

            <Show when=move || has_examples.get()>
                <div class="kanji-detail-section">
                    <div class="kanji-detail-section-title">
                        {examples_heading}
                    </div>
                    <div class="space-y-1">
                        <For
                            each=move || example_words.get()
                            key=|(word, _)| word.clone()
                            children=move |(word, meaning): (String, String)| {
                                view! {
                                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                        {format!("{} — {}", word, meaning)}
                                    </Text>
                                }
                            }
                        />
                    </div>
                </div>
            </Show>

            <div class="kanji-detail-section">
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {stats_text}
                </Text>
            </div>
        </div>
    }
}
