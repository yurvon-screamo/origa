use super::LoginMode;
use super::email_input::EmailInput;
use super::error_message::ErrorMessage;
use super::password_input::PasswordInput;
use crate::app::AuthContext;
use crate::ui_components::{Alert, AlertType, Button, ButtonVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use origa::application::UserRepository;
use origa::domain::{JapaneseLevel, NativeLanguage, User};

const MIN_PASSWORD_LENGTH: usize = 8;

fn validate_email(email: &str) -> Result<(), String> {
    let email = email.trim();
    if email.is_empty() {
        return Err("Введите email".to_string());
    }
    if !email.contains('@') || !email.contains('.') {
        return Err("Некорректный формат email".to_string());
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), String> {
    let password = password.trim();
    if password.is_empty() {
        return Err("Введите пароль".to_string());
    }
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "Пароль должен быть минимум {} символов",
            MIN_PASSWORD_LENGTH
        ));
    }
    Ok(())
}

fn validate_credentials(email: &str, password: &str) -> Result<(), String> {
    validate_email(email)?;
    validate_password(password)?;
    Ok(())
}

fn is_email_not_confirmed_error(error: &str) -> bool {
    error.contains("email_not_confirmed")
}

#[component]
pub fn LoginForm(
    email: RwSignal<String>,
    password: RwSignal<String>,
    error: RwSignal<Option<String>>,
    mode: RwSignal<LoginMode>,
) -> impl IntoView {
    let is_loading = RwSignal::new(false);

    let on_submit = Callback::new(move |()| {
        handle_login(email, password, error, mode, is_loading);
    });

    view! {
        <div class="space-y-5">
            <EmailInput value=email on_enter=on_submit />
            <PasswordInput value=password on_enter=on_submit />

            {move || error.get().map(|err| view! { <ErrorMessage message=err /> })}

            <div class="flex gap-3">
                <Button
                    variant=ButtonVariant::Olive
                    loading=Signal::derive(move || is_loading.get())
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| on_submit.run(()))
                >
                    "Войти"
                </Button>
            </div>

            <div class="text-center">
                <button
                    type="button"
                    class="text-sm text-[var(--fg-muted)] hover:text-[var(--fg)] transition-colors"
                    on:click=move |_| {
                        error.set(None);
                        mode.set(LoginMode::Register);
                    }
                >
                    "Нет аккаунта? Зарегистрироваться"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn RegisterForm(
    email: RwSignal<String>,
    password: RwSignal<String>,
    error: RwSignal<Option<String>>,
    mode: RwSignal<LoginMode>,
) -> impl IntoView {
    let is_loading = RwSignal::new(false);

    let on_submit = Callback::new(move |()| {
        handle_register(email, password, error, mode, is_loading);
    });

    view! {
        <div class="space-y-5">
            <EmailInput value=email on_enter=on_submit />
            <PasswordInput value=password on_enter=on_submit />

            {move || error.get().map(|err| view! { <ErrorMessage message=err /> })}

            <div class="flex gap-3">
                <Button
                    variant=ButtonVariant::Olive
                    loading=Signal::derive(move || is_loading.get())
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| on_submit.run(()))
                >
                    "Зарегистрироваться"
                </Button>
            </div>

            <div class="text-center">
                <button
                    type="button"
                    class="text-sm text-[var(--fg-muted)] hover:text-[var(--fg)] transition-colors"
                    on:click=move |_| {
                        error.set(None);
                        mode.set(LoginMode::Login);
                    }
                >
                    "Уже есть аккаунт? Войти"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn EmailConfirmationForm(
    email: RwSignal<String>,
    error: RwSignal<Option<String>>,
    mode: RwSignal<LoginMode>,
) -> impl IntoView {
    let resend_success = RwSignal::new(false);
    let is_loading = RwSignal::new(false);
    let ctx = use_context::<AuthContext>().expect("AuthContext not provided");

    view! {
        <div class="space-y-5">
            <Alert
                alert_type=AlertType::Info
                title="Регистрация"
                message=move || format!(
                    "Письмо для подтверждения отправлено на: {}. Проверьте почту и перейдите по ссылке в письме для завершения регистрации.",
                    email.get()
                )
            />

            {move || error.get().map(|err| view! { <ErrorMessage message=err /> })}

            {move || {
                if resend_success.get() {
                    view! {
                        <Alert
                            alert_type=AlertType::Success
                            title="Отправлено"
                            message="Письмо отправлено повторно!"
                        />
                    }.into_any()
                } else {
                    view! { <div class="hidden"></div> }.into_any()
                }
            }}

            <div class="flex flex-col gap-3">
                <Button
                    variant=ButtonVariant::Olive
                    loading=Signal::derive(move || is_loading.get())
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        let email_val = email.get();
                        let ctx = ctx.clone();
                        resend_success.set(false);
                        error.set(None);
                        is_loading.set(true);

                        spawn_local(async move {
                            match ctx.client.resend_confirmation_email(&email_val).await {
                                Ok(()) => {
                                    is_loading.set(false);
                                    resend_success.set(true);
                                }
                                Err(e) => {
                                    is_loading.set(false);
                                    error.set(Some(e));
                                }
                            }
                        });
                    })
                >
                    "Отправить письмо повторно"
                </Button>

                <button
                    type="button"
                    class="text-sm text-[var(--fg-muted)] hover:text-[var(--fg)] transition-colors text-center"
                    on:click=move |_| {
                        error.set(None);
                        resend_success.set(false);
                        mode.set(LoginMode::Login);
                    }
                >
                    "Вернуться к входу"
                </button>
            </div>
        </div>
    }
}

async fn get_or_create_profile(ctx: &AuthContext, email: &str) -> Result<User, String> {
    match ctx.repository.find_by_email(email).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => {
            let new_user = User::new(
                email.to_string(),
                JapaneseLevel::N5,
                NativeLanguage::Russian,
                None,
            );

            ctx.repository
                .save(&new_user)
                .await
                .map_err(|e| format!("Не удалось создать профиль: {}", e))?;

            ctx.repository
                .find_by_email(email)
                .await
                .map_err(|e| format!("Не удалось загрузить профиль: {}", e))?
                .ok_or_else(|| "Профиль не найден после создания".to_string())
        }
        Err(e) => Err(format!("Не удалось загрузить профиль: {}", e)),
    }
}

fn handle_login(
    email: RwSignal<String>,
    password: RwSignal<String>,
    error: RwSignal<Option<String>>,
    mode: RwSignal<LoginMode>,
    is_loading: RwSignal<bool>,
) {
    let email_val = email.get();
    let password_val = password.get();

    if let Err(e) = validate_credentials(&email_val, &password_val) {
        error.set(Some(e));
        return;
    }

    error.set(None);
    is_loading.set(true);

    let ctx = use_context::<AuthContext>().expect("AuthContext not provided");
    let navigate = use_navigate();

    spawn_local(async move {
        match ctx.client.login(&email_val, &password_val).await {
            Ok(_session) => match get_or_create_profile(&ctx, &email_val).await {
                Ok(user) => {
                    ctx.current_user.set(Some(user));
                    navigate("/home", Default::default());
                }
                Err(e) => {
                    is_loading.set(false);
                    error.set(Some(e));
                }
            },
            Err(e) => {
                is_loading.set(false);
                if is_email_not_confirmed_error(&e) {
                    error.set(None);
                    mode.set(LoginMode::EmailNotConfirmed);
                } else {
                    error.set(Some(e));
                }
            }
        }
    });
}

fn handle_register(
    email: RwSignal<String>,
    password: RwSignal<String>,
    error: RwSignal<Option<String>>,
    mode: RwSignal<LoginMode>,
    is_loading: RwSignal<bool>,
) {
    let email_val = email.get();
    let password_val = password.get();

    if let Err(e) = validate_credentials(&email_val, &password_val) {
        error.set(Some(e));
        return;
    }

    error.set(None);
    is_loading.set(true);

    let ctx = use_context::<AuthContext>().expect("AuthContext not provided");

    spawn_local(async move {
        match ctx.client.register(&email_val, &password_val).await {
            Ok(_user) => {
                is_loading.set(false);
                mode.set(LoginMode::EmailNotConfirmed);
            }
            Err(e) => {
                is_loading.set(false);
                if is_email_not_confirmed_error(&e) {
                    mode.set(LoginMode::EmailNotConfirmed);
                } else {
                    error.set(Some(e));
                }
            }
        }
    });
}
