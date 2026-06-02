use std::collections::HashSet;

use super::super::shared::{
    CardStatus, DeleteRequest, create_delete_callback, create_mark_as_known_callback,
};
use super::grammar_detail_hero_card::GrammarDetailHeroCard;
use super::grammar_detail_mobile::GrammarMobileOverview;
use super::grammar_practice_modal::GrammarPracticeModal;
use crate::i18n::use_i18n;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    CardActionBar, CardHistoryModal, DeleteConfirmModal, FsrsMetrics, FsrsMetricsMode,
    FuriganaText, LoadingOverlay, MarkdownText, TabItem, Tabs, Tag, Text, TextSize,
    TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use origa::dictionary::grammar::{GrammarRule, get_rule_by_id};
use origa::domain::{Card as DomainCard, StudyCard, User};
use origa::traits::UserRepository;
use origa::use_cases::ToggleFavoriteUseCase;
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
                tracing::warn!("GrammarDetail: user not found");
                is_loading.set(false);
            },
            Err(e) => {
                if disposed.is_disposed() {
                    return;
                }
                tracing::error!("GrammarDetail: get_current_user error: {:?}", e);
                is_loading.set(false);
            },
        }
    });
}

fn extract_grammar_rule(study_card: &StudyCard) -> Option<&'static GrammarRule> {
    match study_card.card() {
        DomainCard::Grammar(grammar) => get_rule_by_id(grammar.rule_id()),
        _ => None,
    }
}

#[component]
pub fn GrammarDetail() -> impl IntoView {
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

    let repo_for_effect = repository.clone();
    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let Some(card_id) = card_id_result.get() else {
            is_loading.set(false);
            return;
        };
        load_study_card(
            repo_for_effect.clone(),
            card_id,
            study_card,
            current_user,
            is_loading,
        );
    });

    let is_favorite_signal: RwSignal<bool> = RwSignal::new(false);
    Effect::new(move |_| {
        if let Some(card) = study_card.get() {
            is_favorite_signal.set(card.is_favorite());
        }
    });

    let on_toggle_favorite = {
        let repo = repository.clone();
        let current_user_fav = current_user;
        let refresh = refresh_trigger;
        Callback::new(move |card_id: Ulid| {
            is_favorite_signal.update(|f| *f = !*f);
            let repo = repo.clone();
            spawn_local(async move {
                let use_case = ToggleFavoriteUseCase::new(&repo);
                if use_case.execute(card_id).await.is_ok() {
                    current_user_fav.update(|u| {
                        if let Some(user) = u {
                            let _ = user.toggle_favorite(card_id);
                        }
                    });
                    refresh.update(|v| *v += 1);
                } else {
                    is_favorite_signal.update(|f| *f = !*f);
                }
            });
        })
    };

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
    let is_practice_open = RwSignal::new(false);
    let navigate = StoredValue::new(use_navigate());
    let active_tab: RwSignal<String> = RwSignal::new("overview".to_string());

    let not_found_text = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .not_found()
            .inner()
            .to_string()
    });
    let loading_text =
        Signal::derive(move || i18n.get_keys().common().loading().inner().to_string());

    let breadcrumbs_label = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .header()
            .inner()
            .to_string()
            .to_uppercase()
    });

    let tab_items = Signal::derive(move || {
        vec![
            TabItem {
                id: "overview".to_string(),
                label: i18n
                    .get_keys()
                    .grammar_page()
                    .tab_overview()
                    .inner()
                    .to_string(),
            },
            TabItem {
                id: "practice".to_string(),
                label: i18n
                    .get_keys()
                    .grammar_page()
                    .tab_practice()
                    .inner()
                    .to_string(),
            },
            TabItem {
                id: "stats".to_string(),
                label: i18n
                    .get_keys()
                    .grammar_page()
                    .tab_stats()
                    .inner()
                    .to_string(),
            },
        ]
    });

    let practice_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .practice()
            .inner()
            .to_string()
    });
    let practice_label = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .practice_start()
            .inner()
            .to_string()
    });
    let explanation_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .explanation()
            .inner()
            .to_string()
    });
    let how_to_form_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .how_to_form()
            .inner()
            .to_string()
    });
    let examples_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .examples()
            .inner()
            .to_string()
    });
    let nuances_title =
        Signal::derive(move || i18n.get_keys().grammar_page().nuances().inner().to_string());
    let pro_tip_title =
        Signal::derive(move || i18n.get_keys().grammar_page().pro_tip().inner().to_string());
    let related_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .related_patterns()
            .inner()
            .to_string()
    });

    view! {
        <div class="grammar-detail-container">
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
                    let memory = card.memory().clone();

                    let status = CardStatus::from_study_card(&card);
                    let grammar_rule = extract_grammar_rule(&card);

                    let title_text = match card.card() {
                        DomainCard::Grammar(grammar) => {
                            let lang = native_lang.get();
                            grammar.title(&lang).ok().map(|t| t.text().to_string()).unwrap_or_default()
                        },
                        _ => "?".to_string(),
                    };

                    let short_description = Memo::new(move |_| {
                        grammar_rule
                            .map(|r| r.content(&native_lang.get()).short_description().to_string())
                            .unwrap_or_default()
                    });

                    let explanation = Memo::new(move |_| {
                        grammar_rule.map(|r| r.content(&native_lang.get()).explanation().to_string())
                    });
                    let how_to_form = Memo::new(move |_| {
                        grammar_rule.map(|r| r.content(&native_lang.get()).how_to_form().to_string())
                    });
                    let examples = Memo::new(move |_| {
                        grammar_rule.map(|r| r.content(&native_lang.get()).examples().to_string())
                    });
                    let nuances = Memo::new(move |_| {
                        grammar_rule.map(|r| r.content(&native_lang.get()).nuances().to_string())
                    });
                    let pro_tip = Memo::new(move |_| {
                        grammar_rule.map(|r| r.content(&native_lang.get()).pro_tip().to_string())
                    });
                    let related_patterns = Memo::new(move |_| {
                        grammar_rule.and_then(|r| {
                            r.content(&native_lang.get()).related_patterns().map(|s| s.to_string())
                        })
                    });
                    let has_quiz = grammar_rule.map(|r| r.has_format_map()).unwrap_or(false);

                    let next_review = memory
                        .next_review_date()
                        .map(|d| d.format("%d.%m.%Y").to_string())
                        .unwrap_or("-".to_string());

                    let memory_history: StoredValue<origa::domain::MemoryHistory> =
                        StoredValue::new(memory.clone());
                    let title_stored: StoredValue<String> = StoredValue::new(title_text.clone());
                    let known_kanji_stored: StoredValue<HashSet<char>> =
                        StoredValue::new(known_kanji.get());

                    let card_id_for_delete = card_id;
                    let confirm_delete = Callback::new(move |_| {
                        on_delete.run(DeleteRequest {
                            card_id: card_id_for_delete,
                            on_success: Callback::new(move |_| {
                                is_delete_modal_open.set(false);
                                navigate.get_value()("/grammar", Default::default());
                            }),
                        })
                    });

                    let card_id_for_fav = card_id;
                    let card_id_for_known = card_id;
                    let active_tab_cell = active_tab;
                    let breadcrumbs_label_val = breadcrumbs_label;

                    Some(view! {
                        <div class="grammar-detail-top-bar">
                            <div class="grammar-breadcrumbs">
                                <A href="/grammar">{breadcrumbs_label_val}</A>
                                <span class="grammar-breadcrumbs-separator">"/"</span>
                                <span class="grammar-breadcrumbs-current">
                                    {title_stored.get_value()}
                                </span>
                            </div>
                            <FsrsMetrics
                                difficulty=memory.difficulty().map(|d| d.value())
                                stability=memory.stability().map(|s| s.value())
                                test_id=Signal::derive(|| "grammar-detail-fsrs".to_string())
                            />
                            <CardActionBar
                                tag_variant=Signal::derive(move || status.tag_variant())
                                tag_label=Signal::derive(move || status.label(&i18n))
                                is_favorite=is_favorite_signal.into()
                                on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id_for_fav))
                                show_mark_as_known=Signal::derive(move || status != CardStatus::Learned)
                                on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(card_id_for_known))
                                on_history=Callback::new(move |_| is_history_open.set(true))
                                on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                                test_id=Signal::derive(|| "grammar-detail-actions".to_string())
                                show_tag=Signal::derive(|| false)
                            />
                            <Show when=move || current_user.get().is_some() && grammar_rule.is_some()>
                                <button
                                    class=move || if has_quiz {
                                        "btn btn-olive text-sm cursor-pointer"
                                    } else {
                                        "btn btn-olive text-sm opacity-50 cursor-not-allowed"
                                    }
                                    disabled=!has_quiz
                                    data-testid="grammar-detail-practice-btn"
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        if has_quiz {
                                            is_practice_open.set(true);
                                        }
                                    }
                                >
                                    {practice_label}
                                </button>
                            </Show>
                        </div>

                        // Desktop layout
                        <div class="grammar-detail-grid grammar-detail-desktop">
                            <div class="grammar-detail-left-col">
                                <GrammarDetailHeroCard
                                    title_stored=title_stored
                                    short_description=short_description
                                    tag_variant=Signal::derive(move || status.tag_variant())
                                    tag_label=Signal::derive(move || status.label(&i18n))
                                    known_kanji=known_kanji_stored.get_value()
                                />

                                <Show when=move || explanation.get().is_some_and(|s| !s.is_empty())>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{explanation_title}</div>
                                        <MarkdownText
                                            content=Signal::derive(move || explanation.get().unwrap_or_default())
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </div>
                                </Show>

                                <Show when=move || examples.get().is_some_and(|s| !s.is_empty())>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{examples_title}</div>
                                        <MarkdownText
                                            content=Signal::derive(move || examples.get().unwrap_or_default())
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </div>
                                </Show>
                            </div>

                            <div class="grammar-detail-right-col">
                                <Show when=move || how_to_form.get().is_some_and(|s| !s.is_empty())>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{how_to_form_title}</div>
                                        <MarkdownText
                                            content=Signal::derive(move || how_to_form.get().unwrap_or_default())
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </div>
                                </Show>

                                <Show when=move || nuances.get().is_some_and(|s| !s.is_empty())>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{nuances_title}</div>
                                        <MarkdownText
                                            content=Signal::derive(move || nuances.get().unwrap_or_default())
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </div>
                                </Show>

                                <Show when=move || pro_tip.get().is_some_and(|s| !s.is_empty())>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{pro_tip_title}</div>
                                        <MarkdownText
                                            content=Signal::derive(move || pro_tip.get().unwrap_or_default())
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </div>
                                </Show>

                                <Show when=move || related_patterns.get().is_some_and(|s| !s.is_empty())>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{related_title}</div>
                                        <MarkdownText
                                            content=Signal::derive(move || related_patterns.get().unwrap_or_default())
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </div>
                                </Show>
                            </div>
                        </div>

                        // Mobile layout
                        <div class="grammar-detail-mobile">
                            <div class="grammar-detail-hero-card" style="margin-bottom:16px">
                                <div class="grammar-detail-hero-header">
                                    <div class="grammar-detail-hero-form" style="font-size:28px">
                                        <FuriganaText
                                            text=title_stored.get_value()
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </div>
                                    <Show when=move || !short_description.get().is_empty()>
                                        <div class="grammar-detail-hero-meaning">{short_description}</div>
                                    </Show>
                                    <div class="grammar-detail-hero-badge">
                                        <Tag variant=Signal::derive(move || status.tag_variant())>
                                            {move || status.label(&i18n)}
                                        </Tag>
                                    </div>
                                </div>
                                <div class="grammar-detail-hero-actions">
                                    <FsrsMetrics
                                        difficulty=memory.difficulty().map(|d| d.value())
                                        stability=memory.stability().map(|s| s.value())
                                        test_id=Signal::derive(|| "grammar-detail-fsrs-mobile".to_string())
                                    />
                                    <CardActionBar
                                        tag_variant=Signal::derive(move || status.tag_variant())
                                        tag_label=Signal::derive(move || status.label(&i18n))
                                        is_favorite=is_favorite_signal.into()
                                        on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id_for_fav))
                                        show_mark_as_known=Signal::derive(move || status != CardStatus::Learned)
                                        on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(card_id_for_known))
                                        on_history=Callback::new(move |_| is_history_open.set(true))
                                        on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                                        test_id=Signal::derive(|| "grammar-detail-actions-mobile".to_string())
                                        show_tag=Signal::derive(|| false)
                                    />
                                </div>
                            </div>

                            <div class="grammar-detail-section">
                                <Tabs
                                    tabs=tab_items
                                    active=active_tab_cell
                                    test_id=Signal::derive(|| "grammar-detail-tabs".to_string())
                                    class="tabs--underline".to_string()
                                />
                            </div>

                            <Show when=move || active_tab_cell.get() == "overview">
                                <GrammarMobileOverview
                                    explanation=explanation
                                    how_to_form=how_to_form
                                    examples=examples
                                    nuances=nuances
                                    pro_tip=pro_tip
                                    related_patterns=related_patterns
                                    explanation_title=explanation_title
                                    how_to_form_title=how_to_form_title
                                    examples_title=examples_title
                                    nuances_title=nuances_title
                                    pro_tip_title=pro_tip_title
                                    related_title=related_title
                                    known_kanji=known_kanji_stored.get_value()
                                />
                            </Show>

                            <Show when=move || active_tab_cell.get() == "practice">
                                <div class="grammar-detail-section">
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{practice_title}</div>
                                        <Show when=move || current_user.get().is_some() && grammar_rule.is_some()>
                                            <button
                                                class=move || if has_quiz {
                                                    "btn btn-olive text-sm cursor-pointer".to_string()
                                                } else {
                                                    "btn btn-olive text-sm opacity-50 cursor-not-allowed".to_string()
                                                }
                                                disabled=!has_quiz
                                                data-testid="grammar-detail-practice-btn-mobile"
                                                on:click=move |ev| {
                                                    ev.stop_propagation();
                                                    if has_quiz {
                                                        is_practice_open.set(true);
                                                    }
                                                }
                                            >
                                                {practice_label}
                                            </button>
                                        </Show>
                                    </div>
                                </div>
                            </Show>

                            <Show when=move || active_tab_cell.get() == "stats">
                                <div class="grammar-detail-section">
                                    <FsrsMetrics
                                        difficulty=memory.difficulty().map(|d| d.value())
                                        stability=memory.stability().map(|s| s.value())
                                        next_review_date=next_review.clone()
                                        mode=FsrsMetricsMode::Expanded
                                        test_id=Signal::derive(|| "grammar-detail-fsrs-expanded".to_string())
                                    />
                                </div>
                            </Show>
                        </div>

                        <CardHistoryModal
                            is_open=Signal::derive(move || is_history_open.get())
                            memory=memory_history.get_value()
                            on_close=Callback::new(move |_| is_history_open.set(false))
                        />
                        <DeleteConfirmModal
                            test_id="grammar-detail-delete-modal"
                            is_open=is_delete_modal_open
                            is_deleting=is_deleting.into()
                            on_confirm=confirm_delete
                            on_close=Callback::new(move |_| is_delete_modal_open.set(false))
                        />
                        <Show when=move || is_practice_open.get() && grammar_rule.is_some() && current_user.get().is_some()>
                            {move || {
                                let rule = grammar_rule?;
                                let user = current_user.get()?;
                                Some(view! {
                                    <GrammarPracticeModal
                                        rule=rule
                                        native_language=native_lang
                                        user=user
                                        is_open=Signal::derive(move || is_practice_open.get())
                                        on_close=Callback::new(move |_| is_practice_open.set(false))
                                    />
                                }.into_any())
                            }}
                        </Show>
                    }.into_any())
                }}
            </Show>
        </div>
    }
}
