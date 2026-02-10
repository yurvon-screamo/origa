use crate::components::layout::app_layout::{AppLayout, PageHeader};
use crate::services::user_service::UserService;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{JapaneseLevel, NativeLanguage};

#[component]
pub fn Profile() -> impl IntoView {
    let user_service = use_context::<UserService>().expect("UserService not provided");

    let profile_resource = LocalResource::new({
        let service = user_service.clone();
        move || {
            let service = service.clone();
            async move { service.get_user_profile().await.ok() }
        }
    });

    let profile = Signal::derive(move || profile_resource.get().flatten());

    // Инициализировать selected_level из профиля
    let (selected_level, set_selected_level_signal) = signal(JapaneseLevel::N5);

    // Обновить selected_level когда профиль загрузится
    Effect::new(move |_| {
        if let Some(p) = profile.get() {
            set_selected_level_signal.set(p.current_level);
        }
    });

    // Обработчик изменения уровня JLPT с сохранением
    let handle_level_change = Callback::new({
        let user_service = user_service.clone();
        let set_level = set_selected_level_signal;
        move |level: JapaneseLevel| {
            set_level.set(level);
            let service = user_service.clone();
            spawn_local(async move {
                let _ = service.update_japanese_level(level).await;
            });
        }
    });

    // Инициализировать selected_language из профиля
    let (selected_language, set_selected_language) = signal("ru".to_string());

    // Обработчик изменения языка интерфейса
    let handle_language_change = {
        let user_service = user_service.clone();
        Callback::new(move |lang: String| {
            set_selected_language.set(lang.clone());
            let language = match lang.as_str() {
                "ru" => NativeLanguage::Russian,
                "en" => NativeLanguage::English,
                _ => NativeLanguage::Russian,
            };
            let service = user_service.clone();
            spawn_local(async move {
                let _ = service.update_native_language(language).await;
            });
        })
    };

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
                            let _handle_level = handle_level_change;
                            view! {
                                <button
                                    class=move || {
                                        if selected_level.get() == level_clone {
                                            "jlpt-button active"
                                        } else {
                                            "jlpt-button"
                                        }
                                    }
                                    on:click=move |_| handle_level_change.run(level_clone)
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
                    <select
                        class="settings-select"
                        prop:value=move || selected_language.get()
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            handle_language_change.run(value);
                        }
                    >
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
