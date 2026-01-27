use leptos::prelude::*;
use crate::components::layout::app_layout::{AppLayout, PageHeader};
use crate::components::cards::stat_card::{StatCard, StudyButton, StudyButtonType};

#[component]
pub fn Dashboard() -> impl IntoView {
    // Mock data - will be replaced with real data from use cases
    let (total_cards, set_total_cards) = create_signal(156);
    let (learned_cards, set_learned_cards) = create_signal(89);
    let (in_progress_cards, set_in_progress_cards) = create_signal(34);
    let (new_cards, set_new_cards) = create_signal(33);
    let (difficult_cards, set_difficult_cards) = create_signal(12);
    
    let (lesson_count, set_lesson_count) = create_signal(12);
    let (fixation_count, set_fixation_count) = create_signal(8);
    
    view! {
        <AppLayout active_tab="dashboard".to_string()>
            <PageHeader 
                title="Привет, изучающий!" 
                subtitle="Готовы продолжить обучение?" />
            
            // Study Action Buttons
            <div class="section">
                <div class="grid grid-cols-2 gap-md">
                    <StudyButton 
                        button_type=StudyButtonType::Lesson
                        count=lesson_count()
                        on_click=Callback::new(|_| {
                            // Navigate to study session
                            leptos_router::use_navigate()("/study", Default::default());
                        }) />
                    <StudyButton 
                        button_type=StudyButtonType::Fixation
                        count=fixation_count()
                        on_click=Callback::new(|_| {
                            // Navigate to fixation session
                            leptos_router::use_navigate()("/study?type=fixation", Default::default());
                        }) />
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
                        title="Всего карточек" 
                        value=total_cards().to_string()
                        trend="+12"
                        show_history=true
                        on_history_click=Callback::new(|_| {
                            // Show total cards history
                        }) />
                    <StatCard 
                        title="Изучено" 
                        value=learned_cards().to_string()
                        trend="+5"
                        show_history=true
                        on_history_click=Callback::new(|_| {
                            // Show learned cards history
                        }) />
                    <StatCard 
                        title="В процессе" 
                        value=in_progress_cards().to_string()
                        trend="-2"
                        show_history=true
                        on_history_click=Callback::new(|_| {
                            // Show in progress cards history
                        }) />
                    <StatCard 
                        title="Новые" 
                        value=new_cards().to_string()
                        trend="+8"
                        show_history=true
                        on_history_click=Callback::new(|_| {
                            // Show new cards history
                        }) />
                    <StatCard 
                        title="Сложные слова" 
                        value=difficult_cards().to_string()
                        trend="-3"
                        show_history=true
                        highlight=true
                        on_history_click=Callback::new(|_| {
                            // Show difficult words history
                        }) />
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
                                <span class="quick-stat-value">🔥 5 дней</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </AppLayout>
    }
}