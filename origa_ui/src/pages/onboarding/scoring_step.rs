use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Button, ButtonVariant, Card, DisplayText, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::use_event_listener;
use origa::domain::{Card as DomainCard, NativeLanguage};
use origa::traits::UserRepository;
use origa::use_cases::MarkCardAsKnownUseCase;
use ulid::Ulid;

#[derive(Clone)]
struct ScoringCard {
    card_id: Ulid,
    question: String,
    answer: String,
}

fn extract_card_data(card: &DomainCard, lang: &NativeLanguage) -> (String, String) {
    match card {
        DomainCard::Vocabulary(v) => (
            v.word().text().to_string(),
            v.answer(lang)
                .ok()
                .map(|a| a.text().to_string())
                .unwrap_or_else(|| "(нет перевода)".to_string()),
        ),
        DomainCard::Kanji(k) => (
            k.kanji().text().to_string(),
            k.description()
                .ok()
                .map(|a| a.text().to_string())
                .unwrap_or_else(|| "(нет перевода)".to_string()),
        ),
        DomainCard::Grammar(g) => (
            g.title(lang)
                .ok()
                .map(|q| q.text().to_string())
                .unwrap_or_default(),
            g.description(lang)
                .ok()
                .map(|a| a.text().to_string())
                .unwrap_or_else(|| "(нет перевода)".to_string()),
        ),
    }
}

fn build_scoring_cards(
    study_cards: &std::collections::HashMap<Ulid, origa::domain::StudyCard>,
    lang: &NativeLanguage,
) -> Vec<ScoringCard> {
    study_cards
        .values()
        .filter(|sc| sc.memory().is_new())
        .map(|sc| {
            let (question, answer) = extract_card_data(sc.card(), lang);
            ScoringCard {
                card_id: *sc.card_id(),
                question,
                answer,
            }
        })
        .collect()
}

#[component]
pub fn ScoringStep(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
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
    let is_completed = RwSignal::new(false);
    let known_count: RwSignal<usize> = RwSignal::new(0);
    let disposed = StoredValue::new(());

    let repo_for_load = repository.clone();
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
            let scoring_cards = build_scoring_cards(user.knowledge_set().study_cards(), &lang);

            if disposed.is_disposed() {
                return;
            }

            let total = scoring_cards.len();
            cards.set(scoring_cards);

            if total == 0 {
                is_completed.set(true);
            }

            is_loading.set(false);
        });
    });

    let total = Memo::new(move |_| cards.get().len());

    let on_dont_know = Callback::new(move |_: ()| {
        let idx = current_index.get_untracked();
        let t = total.get_untracked();
        if idx + 1 >= t {
            is_completed.set(true);
        } else {
            current_index.set(idx + 1);
        }
    });

    let on_know = Callback::new(move |_: ()| {
        let idx = current_index.get_untracked();
        let t = total.get_untracked();

        if let Some(scoring_card) = cards.get_untracked().get(idx) {
            let card_id = scoring_card.card_id;
            let repo = repository.clone();
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
                    is_completed.set(true);
                } else {
                    current_index.set(idx + 1);
                }
            });
        }
    });

    let kb_on_dont_know = on_dont_know;
    let kb_on_know = on_know;
    let _ = use_event_listener(document(), leptos::ev::keydown, move |ev| {
        if is_loading.get() || is_rating.get() || is_completed.get() {
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

    view! {
        <div class="scoring-step" data-testid=test_id_val>
            <Show when=move || is_loading.get()>
                <div class="flex flex-col items-center py-8 gap-4">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        "Загрузка карточек..."
                    </Text>
                </div>
            </Show>

            <Show when=move || !is_loading.get() && !is_completed.get()>
                <div>
                    <div class="text-center mb-4">
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
                                    <DisplayText
                                        test_id=Signal::derive(|| "scoring-step-question".to_string())
                                    >
                                        {card.question.clone()}
                                    </DisplayText>

                                    <div class="mt-4">
                                        <Text
                                            size=TextSize::Default
                                            variant=TypographyVariant::Muted
                                            test_id=Signal::derive(|| "scoring-step-answer".to_string())
                                        >
                                            {card.answer.clone()}
                                        </Text>
                                    </div>
                                </div>

                                <div class="grid grid-cols-2 gap-3 mt-6">
                                    <Button
                                        variant=ButtonVariant::Default
                                        disabled=Signal::derive(move || is_rating.get())
                                        on_click=Callback::new(move |_| on_dont_know.run(()))
                                        test_id=Signal::derive(|| "scoring-step-dont-know".to_string())
                                    >
                                        "Не знаю"
                                        <span class="hidden sm:inline text-xs ml-1">"[1]"</span>
                                    </Button>

                                    <Button
                                        variant=ButtonVariant::Olive
                                        disabled=Signal::derive(move || is_rating.get())
                                        on_click=Callback::new(move |_| on_know.run(()))
                                        test_id=Signal::derive(|| "scoring-step-know".to_string())
                                    >
                                        "Знаю"
                                        <span class="hidden sm:inline text-xs ml-1">"[2]"</span>
                                    </Button>
                                </div>
                            </Card>
                        }
                    })}

                    <div class="text-center mt-4">
                        <Text
                            size=TextSize::Small
                            variant=TypographyVariant::Muted
                            test_id=Signal::derive(|| "scoring-step-hint".to_string())
                        >
                            "Отметьте карточки, которые вы уже знаете"
                        </Text>
                    </div>
                </div>
            </Show>

            <Show when=move || is_completed.get()>
                <div class="text-center py-8">
                    <Text
                        size=TextSize::Large
                        variant=TypographyVariant::Primary
                        test_id=Signal::derive(|| "scoring-step-complete".to_string())
                    >
                        "Отлично!"
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
                                if t == 0 {
                                    "Нет новых карточек для оценки".to_string()
                                } else if known == 0 {
                                    "Все карточки новые — пора учить!".to_string()
                                } else {
                                    format!("Вы знаете {} из {} карточек", known, t)
                                }
                            }}
                        </Text>
                    </div>
                    <div class="mt-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Нажмите «Завершить» чтобы начать обучение"
                        </Text>
                    </div>
                </div>
            </Show>
        </div>
    }
}
