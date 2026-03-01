use super::{HistoryModal, StatCard, StatMetric};
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Button, ButtonVariant, Card, Skeleton, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use origa::application::GetUserInfoUseCase;
use origa::domain::{DailyHistoryItem, User};

#[derive(Clone, Default)]
struct HomeStats {
    total_cards: usize,
    learned: usize,
    in_progress: usize,
    new: usize,
    high_difficulty: usize,
    weekly_delta: usize,
}

fn format_number(n: usize) -> String {
    if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

fn calculate_stats(history: &[DailyHistoryItem]) -> HomeStats {
    if history.is_empty() {
        return HomeStats::default();
    }

    let last = history.last().unwrap();
    let weekly_delta = if history.len() >= 2 {
        let prev = &history[history.len() - 2];
        last.total_words().saturating_sub(prev.total_words())
    } else {
        0
    };

    HomeStats {
        total_cards: last.total_words(),
        learned: last.known_words(),
        in_progress: last.in_progress_words(),
        new: last.new_words(),
        high_difficulty: last.high_difficulty_words(),
        weekly_delta,
    }
}

#[component]
pub fn HomeContent() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let stats = RwSignal::new(None::<HomeStats>);
    let history = RwSignal::new(Vec::<DailyHistoryItem>::new());
    let history_open = RwSignal::new(false);
    let selected_metric = RwSignal::new(StatMetric::TotalCards);
    let is_loading = RwSignal::new(true);

    Effect::new(move |_| {
        let user = current_user.get();
        if let Some(user) = user {
            let user_id = user.id();
            let repo = repository.clone();
            spawn_local(async move {
                let use_case = GetUserInfoUseCase::new(&repo);
                match use_case.execute(user_id).await {
                    Ok(profile) => {
                        history.set(profile.lesson_history.clone());
                        stats.set(Some(calculate_stats(&profile.lesson_history)));
                        is_loading.set(false);
                    }
                    Err(_) => {
                        stats.set(Some(HomeStats::default()));
                        is_loading.set(false);
                    }
                }
            });
        }
    });

    let total_cards =
        Signal::derive(move || format_number(stats.get().map(|s| s.total_cards).unwrap_or(0)));
    let learned =
        Signal::derive(move || format_number(stats.get().map(|s| s.learned).unwrap_or(0)));
    let in_progress =
        Signal::derive(move || format_number(stats.get().map(|s| s.in_progress).unwrap_or(0)));
    let new_cards = Signal::derive(move || format_number(stats.get().map(|s| s.new).unwrap_or(0)));
    let high_difficulty =
        Signal::derive(move || format_number(stats.get().map(|s| s.high_difficulty).unwrap_or(0)));

    let weekly_delta_text = Signal::derive(move || {
        stats
            .get()
            .filter(|s| s.weekly_delta > 0)
            .map(|s| format!("+{}", s.weekly_delta))
            .unwrap_or_default()
    });

    let open_history = move |metric: StatMetric| {
        Callback::new(move |_: ()| {
            selected_metric.set(metric);
            history_open.set(true);
        })
    };

    let close_history = Callback::new(move |_: ()| {
        history_open.set(false);
    });

    view! {
        <main class="flex-1">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true class="mb-6">
                    "Статистика"
                </Text>

                <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-6">
                    <Show
                        when=move || !is_loading.get()
                        fallback=move || {
                            view! {
                                {(0..6).map(|_| view! {
                                    <Card class=Signal::derive(|| "p-6".to_string())>
                                        <Skeleton
                                            width=Signal::derive(|| Some("60%".to_string()))
                                            height=Signal::derive(|| Some("12px".to_string()))
                                            class=Signal::derive(|| "mb-4".to_string())
                                        />
                                        <Skeleton
                                            width=Signal::derive(|| Some("50%".to_string()))
                                            height=Signal::derive(|| Some("32px".to_string()))
                                            class=Signal::derive(|| "mb-2".to_string())
                                        />
                                        <Skeleton
                                            width=Signal::derive(|| Some("70%".to_string()))
                                            height=Signal::derive(|| Some("12px".to_string()))
                                            class=Signal::derive(|| "mb-4".to_string())
                                        />
                                        <Skeleton
                                            width=Signal::derive(|| Some("80px".to_string()))
                                            height=Signal::derive(|| Some("36px".to_string()))
                                            class=Signal::derive(String::new)
                                        />
                                    </Card>
                                }).collect::<Vec<_>>()}
                            }
                        }
                    >
                        <Card class=Signal::derive(|| "p-6 flex flex-col justify-between".to_string())>
                            <Text size=TextSize::Small variant=TypographyVariant::Muted class="mb-3">
                                "Обучение"
                            </Text>
                            <div class="flex flex-col gap-2">
                                <A href="/lesson">
                                    <Button variant=Signal::derive(|| ButtonVariant::Filled) class="w-full">
                                        "Урок"
                                    </Button>
                                </A>
                                <A href="/lesson?mode=fixation">
                                    <Button variant=Signal::derive(|| ButtonVariant::Olive) class="w-full">
                                        "Сложные"
                                    </Button>
                                </A>
                            </div>
                        </Card>

                        <StatCard
                            title=Signal::derive(|| "Всего карточек".to_string())
                            value=total_cards
                            subtitle=Signal::derive(|| "в базе".to_string())
                            delta=weekly_delta_text
                            on_history=open_history(StatMetric::TotalCards)
                        />

                        <StatCard
                            title=Signal::derive(|| "Изучено".to_string())
                            value=learned
                            subtitle=Signal::derive(|| "карточек".to_string())
                            on_history=open_history(StatMetric::Learned)
                        />

                        <StatCard
                            title=Signal::derive(|| "В процессе".to_string())
                            value=in_progress
                            subtitle=Signal::derive(|| "изучения".to_string())
                            on_history=open_history(StatMetric::InProgress)
                        />

                        <StatCard
                            title=Signal::derive(|| "Новые".to_string())
                            value=new_cards
                            subtitle=Signal::derive(|| "карточек".to_string())
                            on_history=open_history(StatMetric::New)
                        />

                        <StatCard
                            title=Signal::derive(|| "Сложные слова".to_string())
                            value=high_difficulty
                            subtitle=Signal::derive(|| "требуют внимания".to_string())
                            on_history=open_history(StatMetric::HighDifficulty)
                        />
                    </Show>
                </div>
            </div>

            <HistoryModal
                is_open=Signal::derive(move || history_open.get())
                metric=Signal::derive(move || selected_metric.get())
                history=Signal::derive(move || history.get())
                on_close=close_history
            />
        </main>
    }
}
