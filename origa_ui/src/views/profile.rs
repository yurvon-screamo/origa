use crate::components::*;
use crate::services::*;
use leptos::control_flow::Show;
use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::*;

#[component]
pub fn Profile() -> impl IntoView {
    let loading = RwSignal::new(false);
    let settings_saved = RwSignal::new(false);
    let error_message = RwSignal::new(String::new());

    let llm_provider = RwSignal::new("openai".to_string());
    let api_key = RwSignal::new(String::new());
    let duolingo_jwt = RwSignal::new(String::new());

    let handle_save_settings = move |_| {
        loading.set(true);
        error_message.set(String::new());

        let settings = UserSettings {
            llm_provider: llm_provider.get(),
            api_key: api_key.get(),
            duolingo_jwt: duolingo_jwt.get(),
        };

        spawn_local(async move {
            match update_settings(settings).await {
                Ok(_) => {
                    settings_saved.set(true);
                    loading.set(false);
                }
                Err(err) => {
                    error_message.set(err);
                    loading.set(false);
                }
            }
        });
    };

    view! {
        <MobileLayout>
            <div class="profile-page">
                <Card>
                    <CardHeader>
                        <h2>"Настройки"</h2>
                        <p>"Управление вашим профилем и настройками"</p>
                    </CardHeader>
                    <div class="settings-card-body">
                        <div class="settings-form">
                            <div class="settings-section">
                                <h3>"Профиль пользователя"</h3>
                                <div style="margin-top: 12px;">
                                    <p>"Имя пользователя: Test User"</p>
                                    <p>"Статус: Активный"</p>
                                    <p>"Регистрация: Сегодня"</p>
                                </div>
                            </div>

                            <Divider />

                            <div class="settings-section">
                                <h3>"Настройки LLM"</h3>
                                <div style="margin-top: 12px;">
                                    <div style="margin-bottom: 16px;">
                                        <label for="llm-provider">"Провайдер LLM"</label>
                                        <Select
                                            id="llm-provider"
                                            value=llm_provider
                                        >
                                            <option value="openai">"OpenAI"</option>
                                            <option value="gemini">"Google Gemini"</option>
                                            <option value="claude">"Anthropic Claude"</option>
                                        </Select>
                                    </div>

                                    <div style="margin-bottom: 16px;">
                                        <label for="api-key">"API Ключ"</label>
                                        <Input
                                            id="api-key"
                                            placeholder="Введите ваш API ключ"
                                            value=api_key
                                        />
                                    </div>
                                </div>
                            </div>

                            <Divider />

                            <div class="settings-section">
                                <h3>"Интеграция с Duolingo"</h3>
                                <div style="margin-top: 12px;">
                                    <div style="margin-bottom: 16px;">
                                        <label for="duolingo-jwt">"Duolingo JWT Token"</label>
                                        <Textarea
                                            id="duolingo-jwt"
                                            placeholder="Вставьте ваш JWT токен из Duolingo"
                                            value=duolingo_jwt
                                        />
                                    </div>
                                    <p style="font-size: 12px; color: var(--thaw-text-color-secondary);">
                                        "Как получить токен: Инструменты разработчика → Application → Local Storage → duolingo-current-user → jwt_token"
                                    </p>
                                </div>
                            </div>

                            <Show when=move || settings_saved.get()>
                                <div style="margin-top: 16px;">
                                    "Настройки успешно сохранены!"
                                </div>
                            </Show>

                            <Show when=move || !error_message.get().is_empty()>
                                <div style="margin-top: 16px;">
                                    {error_message.get()}
                                </div>
                            </Show>

                            <div class="settings-actions" style="margin-top: 24px;">
                                <Button
                                    appearance=ButtonAppearance::Primary
                                    on_click=handle_save_settings
                                    loading=loading.get()
                                >
                                    "Сохранить настройки"
                                </Button>
                            </div>
                        </div>
                    </div>
                </Card>

                <Card>
                    <CardHeader>
                        <h3>"Статистика"</h3>
                    </CardHeader>
                    <div class="stats-card-body">
                        <Grid cols=2 x_gap=16 y_gap=16>
                            <Card>
                                <div style="text-align: center;">
                                    <h3>"120"</h3>
                                    <p>"Карточек всего"</p>
                                </div>
                            </Card>
                            <Card>
                                <div style="text-align: center;">
                                    <h3>"45"</h3>
                                    <p>"Изучено"</p>
                                </div>
                            </Card>
                            <Card>
                                <div style="text-align: center;">
                                    <h3>"15"</h3>
                                    <p>"В изучении"</p>
                                </div>
                            </Card>
                            <Card>
                                <div style="text-align: center;">
                                    <h3>"5"</h3>
                                    <p>"Дней подряд"</p>
                                </div>
                            </Card>
                        </Grid>
                    </div>
                </Card>
            </div>
        </MobileLayout>
    }
}
