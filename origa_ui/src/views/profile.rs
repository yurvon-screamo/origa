use leptos::*;
use leptos_router::*;
use thaw::*;
use crate::services::*;
use crate::components::*;

#[component]
pub fn Profile() -> impl IntoView {
    let loading = create_rw_signal(false);
    let settings_saved = create_rw_signal(false);
    let error_message = create_rw_signal(String::new());
    
    let llm_provider = create_rw_signal("openai".to_string());
    let api_key = create_rw_signal(String::new());
    let duolingo_jwt = create_rw_signal(String::new());

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
                    // Hide success message after 3 seconds
                    leptos::task::spawn_local(async {
                        leptos::task::sleep(std::time::Duration::from_secs(3)).await;
                        settings_saved.set(false);
                    });
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
                    <CardBody>
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
                                            style="width: 100%; margin-top: 8px;"
                                        >
                                            <SelectOption value="openai">"OpenAI"</SelectOption>
                                            <SelectOption value="gemini">"Google Gemini"</SelectOption>
                                            <SelectOption value="claude">"Anthropic Claude"</SelectOption>
                                        </Select>
                                    </div>
                                    
                                    <div style="margin-bottom: 16px;">
                                        <label for="api-key">"API Ключ"</label>
                                        <Input
                                            id="api-key"
                                            placeholder="Введите ваш API ключ"
                                            type="password"
                                            value=api_key
                                            on_change=move |v| api_key.set(v)
                                            style="width: 100%; margin-top: 8px;"
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
                                        <TextArea
                                            id="duolingo-jwt"
                                            placeholder="Вставьте ваш JWT токен из Duolingo"
                                            value=duolingo_jwt
                                            on_change=move |v| duolingo_jwt.set(v)
                                            style="width: 100%; margin-top: 8px;"
                                            rows=4
                                        />
                                    </div>
                                    <p style="font-size: 12px; color: var(--thaw-text-color-secondary);">
                                        "Как получить токен: Инструменты разработчика → Application → Local Storage → duolingo-current-user → jwt_token"
                                    </p>
                                </div>
                            </div>

                            <Show when=move || settings_saved.get()>
                                <div style="margin-top: 16px;">
                                    <Alert variant=AlertVariant::Success>
                                        "Настройки успешно сохранены!"
                                    </Alert>
                                </div>
                            </Show>

                            <Show when=move || !error_message.get().is_empty()>
                                <div style="margin-top: 16px;">
                                    <Alert variant=AlertVariant::Error>
                                        {error_message.get()}
                                    </Alert>
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
                    </CardBody>
                </Card>

                <Card style="margin-top: 16px;">
                    <CardHeader>
                        <h3>"Статистика"</h3>
                    </CardHeader>
                    <CardBody>
                        <Grid columns="repeat(2, 1fr)" gap="16px">
                            <Card>
                                <CardBody style="text-align: center;">
                                    <h3>"120"</h3>
                                    <p>"Карточек всего"</p>
                                </CardBody>
                            </Card>
                            <Card>
                                <CardBody style="text-align: center;">
                                    <h3>"45"</h3>
                                    <p>"Изучено"</p>
                                </CardBody>
                            </Card>
                            <Card>
                                <CardBody style="text-align: center;">
                                    <h3>"15"</h3>
                                    <p>"В изучении"</p>
                                </CardBody>
                            </Card>
                            <Card>
                                <CardBody style="text-align: center;">
                                    <h3>"5"</h3>
                                    <p>"Дней подряд"</p>
                                </CardBody>
                            </Card>
                        </Grid>
                    </CardBody>
                </Card>
            </div>
        </MobileLayout>
    }
}