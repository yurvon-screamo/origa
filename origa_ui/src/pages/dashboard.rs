use crate::components::cards::stat_card::{StatCard, StudyButton, StudyButtonType};
use crate::components::layout::app_layout::{AppLayout, PageHeader};
use crate::services::user_service::UserService;
use leptos::prelude::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    let user_service = use_context::<UserService>().expect("UserService not provided");

    let stats_resource = LocalResource::new({
        let service = user_service.clone();
        move || {
            let service = service.clone();
            async move { service.get_dashboard_stats().await.ok() }
        }
    });

    let stats = Signal::derive(move || stats_resource.get().flatten().unwrap_or_default());

    let total_cards = Signal::derive(move || stats.get().total_cards);
    let learned_cards = Signal::derive(move || stats.get().learned);
    let in_progress_cards = Signal::derive(move || stats.get().in_progress);
    let new_cards = Signal::derive(move || stats.get().new_cards);
    let difficult_cards = Signal::derive(move || stats.get().difficult);
    let lesson_count = Signal::derive(move || stats.get().lesson_count as u32);
    let fixation_count = Signal::derive(move || stats.get().fixation_count as u32);

    let profile_resource = LocalResource::new({
        let service = user_service.clone();
        move || {
            let service = service.clone();
            async move { service.get_user_profile().await.ok() }
        }
    });

    let username = Signal::derive(move || {
        profile_resource
            .get()
            .flatten()
            .map(|p| p.username)
            .unwrap_or_else(|| "Изучающий".to_string())
    });

    view! {
        <AppLayout active_tab="dashboard".to_string()>
            <PageHeader
                title=Signal::derive(move || format!("Привет, {}!", username.get()))
                subtitle="Готовы продолжить обучение?".to_string()
            />

            // Study Action Buttons
            <div class="section">
                <div class="grid grid-cols-2 gap-md">
                    <StudyButton
                        button_type=StudyButtonType::Lesson
                        count=lesson_count
                        on_click=Callback::new(move |_| {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href("/study");
                            }
                        })
                    />
                    <StudyButton
                        button_type=StudyButtonType::Fixation
                        count=fixation_count
                        on_click=Callback::new(move |_| {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href("/study?type=fixation");
                            }
                        })
                    />
                </div>
            </div>

            // Overview Statistics
            <div class="section">
                <div class="section-header">
                    <h2 class="section-title">Обзор</h2>
                    <p class="section-subtitle">Ваш прогресс в изучении</p>
                </div>

                <div class="grid grid-cols-2 gap-md">
                    <StatCard
                        title="Всего карточек".to_string()
                        value=Signal::derive(move || total_cards.get().to_string())
                        trend="+12".to_string()
                        show_history=true
                        on_history_click=Callback::new(|_| {})
                    />
                    <StatCard
                        title="Изучено".to_string()
                        value=Signal::derive(move || learned_cards.get().to_string())
                        trend="+5".to_string()
                        show_history=true
                        on_history_click=Callback::new(|_| {})
                    />
                    <StatCard
                        title="В процессе".to_string()
                        value=Signal::derive(move || in_progress_cards.get().to_string())
                        trend="-2".to_string()
                        show_history=true
                        on_history_click=Callback::new(|_| {})
                    />
                    <StatCard
                        title="Новые".to_string()
                        value=Signal::derive(move || new_cards.get().to_string())
                        trend="+8".to_string()
                        show_history=true
                        on_history_click=Callback::new(|_| {})
                    />
                    <StatCard
                        title="Сложные слова".to_string()
                        value=Signal::derive(move || difficult_cards.get().to_string())
                        trend="-3".to_string()
                        show_history=true
                        highlight=true
                        on_history_click=Callback::new(|_| {})
                    />
                </div>
            </div>

            // Quick Stats Summary (Mobile)
            <div class="section md:hidden">
                <div class="card">
                    <div class="card-content">
                        <div class="quick-stats">
                            <div class="quick-stat-item">
                                <span class="quick-stat-label">Сегодня</span>
                                <span class="quick-stat-value">15 карточек</span>
                            </div>
                            <div class="quick-stat-item">
                                <span class="quick-stat-label">На этой неделе</span>
                                <span class="quick-stat-value">87 карточек</span>
                            </div>
                            <div class="quick-stat-item">
                                <span class="quick-stat-label">Серия</span>
                                <span class="quick-stat-value">{"🔥 5 дней"}</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </AppLayout>
    }
}
