use leptos::prelude::*;
use crate::components::interactive::next_button::NextButton;
use crate::components::interactive::next_button::SkipButton;

#[component]
pub fn StudyNavigation(
    #[prop(into, optional)] show_next: Option<bool>,
    #[prop(into, optional)] show_skip: Option<bool>,
    #[prop(into, optional)] next_disabled: Option<bool>,
    #[prop(into, optional)] skip_disabled: Option<bool>,
    #[prop(into, optional)] on_next: Option<Callback<()>>,
    #[prop(into, optional)] on_skip: Option<Callback<()>>,
    #[prop(into, optional)] on_audio_toggle: Option<Callback<()>>,
    #[prop(into, optional)] audio_enabled: Option<bool>,
) -> impl IntoView {
    let show_next_btn = show_next.unwrap_or(true);
    let show_skip_btn = show_skip.unwrap_or(true);
    let next_btn_disabled = next_disabled.unwrap_or(false);
    let skip_btn_disabled = skip_disabled.unwrap_or(false);
    let audio_btn_enabled = audio_enabled.unwrap_or(true);
    
    let handle_next = on_next.unwrap_or(Callback::new(|_| {}));
    let handle_skip = on_skip.unwrap_or(Callback::new(|_| {}));
    let handle_audio_toggle = on_audio_toggle.unwrap_or(Callback::new(|_| {}));
    
    view! {
        <div class="study-navigation">
            <div class="nav-left">
                <Show when=show_skip_btn>
                    <SkipButton 
                        label="Пропустить"
                        disabled=skip_btn_disabled
                        on_click=handle_skip />
                </Show>
            </div>
            
            <div class="nav-center">
                <Show when=audio_btn_enabled>
                    <button 
                        class="audio-toggle"
                        on:click=handle_audio_toggle
                        aria-label="Переключить аудио"
                    >
                        <span class="audio-icon">
                            {move || if audio_btn_enabled() { "🔊" } else { "🔇" }}
                        </span>
                        <span class="audio-text">Аудио</span>
                    </button>
                </Show>
            </div>
            
            <div class="nav-right">
                <Show when=show_next_btn>
                    <NextButton 
                        label="Далее"
                        disabled=next_btn_disabled
                        on_click=handle_next />
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn SessionProgress(
    #[prop(into, optional)] current: Option<u32>,
    #[prop(into, optional)] total: Option<u32>,
    #[prop(into, optional)] show_streak_info: Option<bool>,
) -> impl IntoView {
    let current_val = current.unwrap_or(0);
    let total_val = total.unwrap_or(1);
    let show_streak = show_streak_info.unwrap_or(false);
    
    let progress_percent = if total_val > 0 {
        (current_val as f32 / total_val as f32) * 100.0
    } else {
        0.0
    };
    
    view! {
        <div class="session-progress">
            <div class="progress-main">
                <div class="progress-text">
                    <span class="progress-label">Прогресс урока</span>
                    <span class="progress-value">{current_val} / {total_val}</span>
                </div>
                
                <div class="progress-bar-container">
                    <ProgressBar 
                        percentage=Signal::derive(move || Some(progress_percent))
                        show_percentage=true />
                </div>
                
                <Show when=show_streak>
                    <div class="streak-indicator">
                        <span class="streak-icon">🔥</span>
                        <span class="streak-text">Серия: {current_val} дней</span>
                    </div>
                </Show>
            </div>
            
            <Show when=show_streak>
                <div class="streak-rewards">
                    <div class="rewards-grid">
                        <div class="reward-item">
                            <span class="reward-day">5 дней</span>
                            <div class="reward-bonus">+2x очки</div>
                        </div>
                        <div class="reward-item">
                            <span class="reward-day">10 дней</span>
                            <div class="reward-bonus">+3x очки</div>
                        </div>
                        <div class="reward-item">
                            <span class="reward-day">20 дней</span>
                            <div class="reward-bonus">+5x очки</div>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn StudySettings(
    #[prop(into, optional)] audio_enabled: Option<bool>,
    #[prop(into, optional)] auto_advance: Option<bool>,
    #[prop(into, optional)] show_answers: Option<bool>,
    #[prop(into, optional)] on_audio_toggle: Option<Callback<()>>,
    #[prop(into, optional)] on_auto_advance_toggle: Option<Callback<()>>,
    #[prop(into, optional)] on_show_answers_toggle: Option<Callback<()>>,
) -> impl IntoView {
    let audio_enabled_val = audio_enabled.unwrap_or(true);
    let auto_advance_val = auto_advance.unwrap_or(false);
    let show_answers_val = show_answers.unwrap_or(false);
    
    let handle_audio_toggle = on_audio_toggle.unwrap_or(Callback::new(|_| {}));
    let handle_auto_advance_toggle = on_auto_advance_toggle.unwrap_or(Callback::new(|_| {}));
    let handle_show_answers_toggle = on_show_answers_toggle.unwrap_or(Callback::new(|_| {}));
    
    view! {
        <div class="study-settings">
            <div class="settings-title">
                <h3>Настройки урока</h3>
            </div>
            
            <div class="settings-list">
                <div class="setting-item">
                    <div class="setting-info">
                        <span class="setting-label">Аудио</span>
                        <span class="setting-description">Произношение слов</span>
                    </div>
                    <div class="setting-control">
                        <label class="toggle-switch">
                            <input 
                                type="checkbox"
                                checked=audio_enabled_val
                                on:click=handle_audio_toggle
                            />
                            <span class="toggle-slider"></span>
                            <span class="toggle-bg"></span>
                        </label>
                    </div>
                </div>
                
                <div class="setting-item">
                    <div class="setting-info">
                        <span class="setting-label">Автопереход</span>
                        <span class="setting-description">Автоматически следующая карточка</span>
                    </div>
                    <div class="setting-control">
                        <label class="toggle-switch">
                            <input 
                                type="checkbox"
                                checked=auto_advance_val
                                on:click=handle_auto_advance_toggle
                            />
                            <span class="toggle-slider"></span>
                            <span class="toggle-bg"></span>
                        </label>
                    </div>
                </div>
                
                <div class="setting-item">
                    <div class="setting-info">
                        <span class="setting-label">Показать ответы</span>
                        <span class="setting-description">Отображение переводов сразу</span>
                    </div>
                    <div class="setting-control">
                        <label class="toggle-switch">
                            <input 
                                type="checkbox"
                                checked=show_answers_val
                                on:click=handle_show_answers_toggle
                            />
                            <span class="toggle-slider"></span>
                            <span class="toggle-bg"></span>
                        </label>
                    </div>
                </div>
            </div>
            
            <div class="settings-actions">
                <button class="settings-button">Сбросить настройки</button>
            </div>
        </div>
    }
}