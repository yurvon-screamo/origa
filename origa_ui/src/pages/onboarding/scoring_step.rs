use crate::i18n::*;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    AudioButtons, Button, ButtonVariant, Card, FuriganaText, MarkdownText, Text, TextSize,
    TypographyVariant, get_reading_from_text, is_speech_supported, speak_text,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::use_event_listener;
use origa::domain::{RateMode, Rating};
use origa::traits::UserRepository;
use origa::use_cases::MarkCardAsKnownUseCase;
use std::collections::HashSet;

use ulid::Ulid;

use super::scoring_helpers::{ScoringCard, build_scoring_cards};

#[component]
pub fn ScoringStep(
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional)] mark_all_trigger: RwSignal<u32>,
    scoring_completed: RwSignal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let cards: RwSignal<Vec<ScoringCard>> = RwSignal::new(Vec::new());
    let current_index: RwSignal<usize> = RwSignal::new(0);
    let is_loading = RwSignal::new(true);
    let is_rating = RwSignal::new(false);

    let known_count: RwSignal<usize> = RwSignal::new(0);
    let disposed = StoredValue::new(());

    let repo_for_load = repository.clone();
    let repo_for_know = repository.clone();
    let repo_for_mark_all = repository.clone();

    let i18n_for_load = i18n;
    Effect::new(move |_| {
        let repo = repo_for_load.clone();
        spawn_local(async move {
            let Ok(Some(user)) = repo.get_current_user().await else {
                is_loading.set(false);
                return;
            };

            if disposed.is_disposed() {
                return;
            }

            let lang = *user.native_language();
            let scoring_cards =
                build_scoring_cards(user.knowledge_set().study_cards(), &lang, i18n_for_load);

            if disposed.is_disposed() {
                return;
            }

            let total = scoring_cards.len();
            cards.set(scoring_cards);

            if total == 0 {
                scoring_completed.set(true);
            }

            is_loading.set(false);
        });
    });

    let total = Memo::new(move |_| cards.get().len());

    let on_dont_know = Callback::new(move |_: ()| {
        let idx = current_index.get_untracked();
        let t = total.get_untracked();
        if idx + 1 >= t {
            scoring_completed.set(true);
        } else {
            current_index.set(idx + 1);
        }
    });

    let on_know = Callback::new(move |_: ()| {
        let idx = current_index.get_untracked();
        let t = total.get_untracked();

        if let Some(scoring_card) = cards.get_untracked().get(idx) {
            let card_id = scoring_card.card_id;
            let repo = repo_for_know.clone();
            is_rating.set(true);

            spawn_local(async move {
                let use_case = MarkCardAsKnownUseCase::new(&repo);
                if use_case.execute(card_id).await.is_ok() {
                    known_count.update(|c| *c += 1);
                }

                if disposed.is_disposed() {
                    return;
                }

                is_rating.set(false);

                if idx + 1 >= t {
                    scoring_completed.set(true);
                } else {
                    current_index.set(idx + 1);
                }
            });
        }
    });

    let kb_on_dont_know = on_dont_know;
    let kb_on_know = on_know;
    let _ = use_event_listener(document(), leptos::ev::keydown, move |ev| {
        if is_loading.get() || is_rating.get() || scoring_completed.get() {
            return;
        }
        match ev.key().as_str() {
            "1" => kb_on_dont_know.run(()),
            "2" => kb_on_know.run(()),
            _ => {},
        }
    });

    let current_card: Signal<Option<ScoringCard>> =
        Signal::derive(move || cards.get().get(current_index.get()).cloned());

    Effect::new(move |_| {
        if is_loading.get() || scoring_completed.get() {
            return;
        }
        if let Some(card) = current_card.get() {
            if is_speech_supported() {
                let reading = get_reading_from_text(&card.question);
                let _ = speak_text(&reading, 1.0);
            }
        }
    });

    {
        let repo = repo_for_mark_all.clone();
        let mark_all_disposed = disposed;
        Effect::new(move |_| {
            let trigger_val = mark_all_trigger.get();
            if trigger_val == 0 {
                return;
            }
            if is_loading.get()
                || scoring_completed.get()
                || cards.get().is_empty()
                || is_rating.get_untracked()
            {
                return;
            }

            let remaining_ids: Vec<Ulid> = cards
                .get_untracked()
                .iter()
                .skip(current_index.get_untracked())
                .map(|c| c.card_id)
                .collect();

            if remaining_ids.is_empty() {
                return;
            }

            is_rating.set(true);

            let repo = repo.clone();
            spawn_local(async move {
                let Ok(Some(mut user)) = repo.get_current_user().await else {
                    is_rating.set(false);
                    return;
                };

                let mut success_count: usize = 0;
                for card_id in &remaining_ids {
                    if let Some(study_card) = user.knowledge_set().get_card(*card_id) {
                        if !study_card.memory().is_new() {
                            continue;
                        }
                    }

                    if user
                        .rate_card(*card_id, Rating::Easy, RateMode::StandardLesson)
                        .is_ok()
                    {
                        success_count += 1;
                    } else {
                        tracing::warn!("Failed to rate card {} in batch mark-all", card_id);
                    }
                }

                if mark_all_disposed.is_disposed() {
                    return;
                }

                if repo.save(&user).await.is_ok() {
                    known_count.update(|c| *c += success_count);
                    scoring_completed.set(true);
                } else {
                    tracing::error!("Failed to save user after batch mark-all-known");
                }

                is_rating.set(false);
            });
        });
    }

    view! {
        <div class="scoring-step" data-testid=test_id_val>
            <Show when=move || is_loading.get()>
                <div class="flex flex-col items-center py-8 gap-4">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {t!(i18n, onboarding.scoring.loading)}
                    </Text>
                </div>
            </Show>

            <Show when=move || !is_loading.get() && !scoring_completed.get()>
                <div>
                    <div class="text-center mb-4">
                        <Text
                            size=TextSize::Small
                            variant=TypographyVariant::Muted
                            test_id=Signal::derive(|| "scoring-step-hint".to_string())
                        >
                            {t!(i18n, onboarding.scoring.hint)}
                        </Text>
                    </div>

                    <div class="text-center mb-2">
                        <Text
                            size=TextSize::Default
                            variant=TypographyVariant::Muted
                            test_id=Signal::derive(|| "scoring-step-progress".to_string())
                        >
                            {move || {
                                let idx = current_index.get() + 1;
                                let t = total.get();
                                format!("{} / {}", idx, t)
                            }}
                        </Text>
                    </div>

                            {move || current_card.get().map(|card| {
                        view! {
                            <Card class=Signal::derive(|| "p-6".to_string())>
                                <div class="text-center">
                                    <div class="relative">
                                        <div class="text-center">
                                            <FuriganaText
                                                text={card.question.clone()}
                                                known_kanji=HashSet::new()
                                                test_id=Signal::derive(|| "scoring-step-question".to_string())
                                            />
                                        </div>
                                        <div class="absolute right-0 top-1/2 -translate-y-1/2">
                                            <AudioButtons
                                                text=card.question.clone()
                                                class=Signal::derive(|| "".to_string())
                                                test_id=Signal::derive(|| "scoring-step-audio".to_string())
                                            />
                                        </div>
                                    </div>

                                    <div class="mt-4">
                                        <MarkdownText
                                            content=Signal::derive(move || card.answer.clone())
                                            known_kanji=HashSet::new()
                                            test_id=Signal::derive(|| "scoring-step-answer".to_string())
                                        />
                                    </div>
                                </div>

                                <div class="grid grid-cols-2 gap-3 mt-6">
                                    <Button
                                        variant=ButtonVariant::Default
                                        disabled=Signal::derive(move || is_rating.get())
                                        on_click=Callback::new(move |_| on_dont_know.run(()))
                                        test_id=Signal::derive(|| "scoring-step-dont-know".to_string())
                                    >
                                        {t!(i18n, onboarding.scoring.dont_know)}
                                        <span class="hidden sm:inline text-xs ml-1">"[1]"</span>
                                    </Button>

                                    <Button
                                        variant=ButtonVariant::Olive
                                        disabled=Signal::derive(move || is_rating.get())
                                        on_click=Callback::new(move |_| on_know.run(()))
                                        test_id=Signal::derive(|| "scoring-step-know".to_string())
                                    >
                                        {t!(i18n, onboarding.scoring.know)}
                                        <span class="hidden sm:inline text-xs ml-1">"[2]"</span>
                                    </Button>
                                </div>
                            </Card>
                        }
                    })}
                </div>
            </Show>

            <Show when=move || scoring_completed.get()>
                <div class="text-center py-8">
                    <Text
                        size=TextSize::Large
                        variant=TypographyVariant::Primary
                        test_id=Signal::derive(|| "scoring-step-complete".to_string())
                    >
                        {t!(i18n, onboarding.scoring.great)}
                    </Text>
                    <div class="mt-2">
                        <Text
                            size=TextSize::Default
                            variant=TypographyVariant::Muted
                            test_id=Signal::derive(|| "scoring-step-complete-info".to_string())
                        >
                            {move || {
                                let known = known_count.get();
                                let t = total.get();
                                let locale = i18n.get_locale();
                                if t == 0 {
                                    td_string!(locale, onboarding.scoring.no_new_cards).to_string()
                                } else if known == 0 {
                                    td_string!(locale, onboarding.scoring.all_new).to_string()
                                } else {
                                    td_string!(locale, onboarding.scoring.you_know_count)
                                        .replace("{known}", &known.to_string())
                                        .replace("{total}", &t.to_string())
                                }
                            }}
                        </Text>
                    </div>
                    <div class="mt-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {t!(i18n, onboarding.scoring.press_finish)}
                        </Text>
                    </div>
                </div>
            </Show>
        </div>
    }
}
