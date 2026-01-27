use crate::components::cards::stat_card::{StatCard, StudyButton, StudyButtonType};
use crate::components::layout::app_layout::{AppLayout, PageHeader};
use leptos::prelude::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    // Mock data - will be replaced with real data from use cases
    let (total_cards, _set_total_cards) = signal(156);
    let (learned_cards, _set_learned_cards) = signal(89);
    let (in_progress_cards, _set_in_progress_cards) = signal(34);
    let (new_cards, _set_new_cards) = signal(33);
    let (difficult_cards, _set_difficult_cards) = signal(12);

    let (lesson_count, _set_lesson_count) = signal(12);
    let (fixation_count, _set_fixation_count) = signal(8);

    view! {
        <AppLayout active_tab="dashboard".to_string()>
            <PageHeader
                title="Привет, изучающий!".to_string()
                subtitle="Готовы продолжить обучение?".to_string()
            />

            // Study Action Buttons
            <div class="section">
                <div class="grid grid-cols-2 gap-md">
                    <StudyButton
                        button_type=StudyButtonType::Lesson
                        count=lesson_count.get()
                        on_click=Callback::new(move |_| {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href("/study");
                            }
                        })
                    />
                    <StudyButton
                        button_type=StudyButtonType::Fixation
                        count=fixation_count.get()
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
                        value=total_cards.get().to_string()
                        trend="+12".to_string()
                        show_history=true
                        on_history_click=Callback::new(|_| {})
                    />
                    <StatCard
                        title="Изучено".to_string()
                        value=learned_cards.get().to_string()
                        trend="+5".to_string()
                        show_history=true
                        on_history_click=Callback::new(|_| {})
                    />
                    <StatCard
                        title="В процессе".to_string()
                        value=in_progress_cards.get().to_string()
                        trend="-2".to_string()
                        show_history=true
                        on_history_click=Callback::new(|_| {})
                    />
                    <StatCard
                        title="Новые".to_string()
                        value=new_cards.get().to_string()
                        trend="+8".to_string()
                        show_history=true
                        on_history_click=Callback::new(|_| {})
                    />
                    <StatCard
                        title="Сложные слова".to_string()
                        value=difficult_cards.get().to_string()
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
