use crate::components::layout::app_layout::{AppLayout, PageHeader};
use crate::services::user_service::UserService;
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

#[component]
pub fn Profile() -> impl IntoView {
    let user_service = use_context::<UserService>().expect("UserService not provided");

    let profile_resource = LocalResource::new({
        let service = user_service.clone();
        move || {
            let service = service.clone();
            async move {
                let user_id = ulid::Ulid::new();
                service.get_user_profile(user_id).await.ok()
            }
        }
    });

    let profile = Signal::derive(move || profile_resource.get().flatten());

    let (selected_level, set_selected_level) = signal(JapaneseLevel::N5);

    let handle_logout = move |_| {
        // TODO: Реализовать logout
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/");
        }
    };

    view! {
        <AppLayout active_tab="profile".to_string()>
            <PageHeader
                title=Signal::derive(|| "Профиль".to_string())
                subtitle="Настройки аккаунта".to_string()
            />

            <div class="section">
                <div class="profile-card">
                    <div class="profile-avatar">
                        <span class="avatar-placeholder">{"👤"}</span>
                    </div>
                    <div class="profile-info">
                        <h2 class="profile-name">
                            {move || profile.get().map(|p| p.username).unwrap_or_default()}
                        </h2>
                        <p class="profile-email">
                            {move || profile.get().map(|p| p.email).unwrap_or_default()}
                        </p>
                    </div>
                </div>
            </div>

            <div class="section">
                <h3 class="section-title">Целевой уровень JLPT</h3>
                <div class="jlpt-selector">
                    {[JapaneseLevel::N5, JapaneseLevel::N4, JapaneseLevel::N3, JapaneseLevel::N2, JapaneseLevel::N1]
                        .into_iter()
                        .map(|level| {
                            let level_clone = level;
                            view! {
                                <button
                                    class=move || {
                                        if selected_level.get() == level_clone {
                                            "jlpt-button active"
                                        } else {
                                            "jlpt-button"
                                        }
                                    }
                                    on:click=move |_| set_selected_level.set(level_clone)
                                >
                                    {level.to_string()}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </div>

            <div class="section">
                <h3 class="section-title">Интеграции</h3>
                <div class="integration-card">
                    <span class="integration-icon">{"🦉"}</span>
                    <div class="integration-info">
                        <h4>Duolingo</h4>
                        <p>Синхронизируйте прогресс с Duolingo</p>
                    </div>
                    <button class="button button-secondary">Подключить</button>
                </div>
            </div>

            <div class="section">
                <h3 class="section-title">Настройки</h3>
                <div class="settings-item">
                    <span>Язык интерфейса</span>
                    <select class="settings-select">
                        <option value="ru">Русский</option>
                        <option value="en">English</option>
                    </select>
                </div>
            </div>

            <div class="section">
                <button class="button button-danger full-width" on:click=handle_logout>
                    Выйти из аккаунта
                </button>
            </div>
        </AppLayout>
    }
}
