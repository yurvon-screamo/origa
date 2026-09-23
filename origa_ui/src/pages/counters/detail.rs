use super::super::shared::{
    CardStatus, DeleteRequest, create_delete_callback, create_mark_as_known_callback,
    format_answer_text,
};
use crate::i18n::use_i18n;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    CardActionBar, DeleteConfirmModal, FsrsMetrics, LoadingOverlay, Tag, Text, TextSize,
    TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use origa::domain::{Card as DomainCard, StudyCard};
use origa::traits::UserRepository;
use origa::use_cases::ToggleFavoriteUseCase;
use ulid::Ulid;

fn load_study_card(
    repository: HybridUserRepository,
    card_id: Ulid,
    result_signal: RwSignal<Option<StudyCard>>,
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
                is_loading.set(false);
            },
            Ok(None) => {
                if disposed.is_disposed() {
                    return;
                }
                tracing::warn!("CountersDetail: user not found");
                is_loading.set(false);
            },
            Err(e) => {
                if disposed.is_disposed() {
                    return;
                }
                tracing::error!("CountersDetail: get_current_user error: {e:?}");
                is_loading.set(false);
            },
        }
    });
}

#[component]
pub fn CountersDetail() -> impl IntoView {
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
        load_study_card(repo_for_effect.clone(), card_id, study_card, is_loading);
    });

    let is_favorite_signal: RwSignal<bool> = RwSignal::new(false);
    Effect::new(move |_| {
        if let Some(card) = study_card.get() {
            is_favorite_signal.set(card.is_favorite());
        }
    });

    let favorite_pending = RwSignal::new(false);
    let on_toggle_favorite = {
        let repo = repository.clone();
        let refresh = refresh_trigger;
        let pending = favorite_pending;
        Callback::new(move |card_id: Ulid| {
            is_favorite_signal.update(|f| *f = !*f);
            let repo = repo.clone();
            spawn_local(async move {
                pending.set(true);
                let use_case = ToggleFavoriteUseCase::new(&repo);
                if use_case.execute(card_id).await.is_ok() {
                    refresh.update(|v| *v += 1);
                } else {
                    is_favorite_signal.update(|f| *f = !*f);
                }
                pending.set(false);
            });
        })
    };
    let (on_mark_as_known, mark_known_pending) =
        create_mark_as_known_callback(repository.clone(), refresh_trigger);
    let toasts: RwSignal<Vec<crate::ui_components::ToastData>> = RwSignal::new(Vec::new());
    let (is_deleting, on_delete) =
        create_delete_callback(repository.clone(), toasts, refresh_trigger);

    let native_lang =
        Memo::new(move |_| crate::i18n::locale_to_native_language(&i18n.get_locale()));

    let is_delete_modal_open = RwSignal::new(false);
    let navigate = StoredValue::new(use_navigate());

    let not_found_text =
        Signal::derive(move || i18n.get_keys().counters().not_found().inner().to_string());
    let loading_text =
        Signal::derive(move || i18n.get_keys().common().loading().inner().to_string());

    let breadcrumbs_label = Signal::derive(move || {
        i18n.get_keys()
            .counters()
            .header()
            .inner()
            .to_string()
            .to_uppercase()
    });

    view! {
        <div class="counter-detail-container" data-testid="counters-detail">
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

                    let (suffix, readings) = match card.card() {
                        DomainCard::Counter(counter_card) => {
                            // Чтения — реестр (истина датасета), статус
                            // изученности — память связки карты юзера.
                            let rows: Vec<(String, String, bool)> =
                                origa::dictionary::counters::get_counter(counter_card.suffix())
                                    .map(|entry| {
                                        entry
                                            .readings()
                                            .iter()
                                            .map(|r| {
                                                (
                                                    match r.number() {
                                                        0 => "何".to_string(),
                                                        n => n.to_string(),
                                                    },
                                                    r.reading().to_string(),
                                                    counter_card
                                                        .binding_memory(r.number())
                                                        .is_some_and(|m| m.is_known_card()),
                                                )
                                            })
                                            .collect()
                                    })
                                    .unwrap_or_default();
                            (counter_card.suffix().to_string(), rows)
                        },
                        _ => return None,
                    };

                    let card_for_answer = card.clone();
                    let answer_text = Memo::new(move |_| {
                        let lang = native_lang.get();
                        format_answer_text(card_for_answer.card(), &lang)
                    });

                    let known_count = readings.iter().filter(|r| r.2).count();
                    let total = readings.len();
                    let bindings_label = Signal::derive(move || {
                        i18n.get_keys()
                            .counters()
                            .bindings_learned()
                            .inner()
                            .to_string()
                            .replacen("{}", &format!("{known_count}/{total}"), 1)
                    });

                    let readings_title = Signal::derive(move || {
                        i18n.get_keys().counters().readings_title().inner().to_string()
                    });

                    let card_id_for_delete = card_id;
                    let confirm_delete = Callback::new(move |_| {
                        on_delete.run(DeleteRequest {
                            card_id: card_id_for_delete,
                            on_success: Callback::new(move |_| {
                                is_delete_modal_open.set(false);
                                navigate.get_value()("/counters", Default::default());
                            }),
                        })
                    });

                    let card_id_for_known = card_id;
                    let card_id_for_fav = card_id;
                    let suffix_for_breadcrumbs = suffix.clone();

                    Some(view! {
                        // Breadcrumbs + Actions
                        <div class="counter-detail-top-bar">
                            <div class="counter-breadcrumbs">
                                <A href="/counters">{breadcrumbs_label}</A>
                                <span class="counter-breadcrumbs-separator">"/"</span>
                                <span class="counter-breadcrumbs-current">
                                    {suffix_for_breadcrumbs}
                                </span>
                            </div>
                            <FsrsMetrics
                                difficulty=memory.difficulty().map(|d| d.value())
                                stability=memory.stability().map(|s| s.value())
                                test_id=Signal::derive(|| "counters-detail-fsrs".to_string())
                            />
                            <CardActionBar
                                tag_variant=Signal::derive(move || status.tag_variant())
                                tag_label=Signal::derive(move || status.label(&i18n))
                                is_favorite=is_favorite_signal.into()
                                on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id_for_fav))
                                favorite_pending=favorite_pending
                                show_mark_as_known=Signal::derive(move || status != CardStatus::Learned)
                                on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(card_id_for_known))
                                mark_known_pending=mark_known_pending
                                on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                                test_id=Signal::derive(|| "counters-detail-actions".to_string())
                                show_tag=Signal::derive(|| false)
                            />
                        </div>

                        // Hero: знак + глосса + сводка связок
                        <div class="counter-detail-hero-card" data-testid="counters-detail-hero">
                            <div class="counter-detail-hero-header">
                                <div class="counter-detail-hero-char-box">
                                    <span class="counter-detail-hero-char">{suffix.clone()}</span>
                                </div>
                                <div class="counter-detail-hero-info">
                                    <div class="counter-detail-hero-meaning">
                                        {move || answer_text.get()}
                                    </div>
                                    <div class="counter-detail-hero-bindings" data-testid="counters-detail-bindings">
                                        {move || bindings_label.get()}
                                    </div>
                                </div>
                                <div class="counter-detail-hero-badge">
                                    <Tag variant=Signal::derive(move || status.tag_variant())>
                                        {move || status.label(&i18n)}
                                    </Tag>
                                </div>
                            </div>
                        </div>

                        // Таблица чтений с состоянием связок
                        <div class="counter-detail-section-card" style="margin-top:16px" data-testid="counters-detail-table">
                            <div class="counter-detail-section-title">{readings_title}</div>
                            <div class="counter-readings-table">
                                {readings
                                    .into_iter()
                                    .map(|(numeral, reading, known)| {
                                        view! {
                                            <div class=format!(
                                                "counter-readings-row{}",
                                                if known { " counter-readings-row--known" } else { "" },
                                            )>
                                                <span class="counter-readings-numeral">
                                                    {numeral}{"×"}{suffix.clone()}
                                                </span>
                                                <span class="counter-readings-reading">{reading}</span>
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>
                        </div>

                        <DeleteConfirmModal
                            test_id="counters-detail-delete-modal"
                            is_open=is_delete_modal_open
                            is_deleting=is_deleting.into()
                            on_confirm=confirm_delete
                            on_close=Callback::new(move |_| is_delete_modal_open.set(false))
                        />
                    }.into_any())
                }}
            </Show>
        </div>
    }
}
