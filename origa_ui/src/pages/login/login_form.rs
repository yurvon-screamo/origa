use super::auth_handlers::handle_login;
use super::email_input::EmailInput;
use super::error_message::ErrorMessage;
use super::oauth_buttons::OAuthButtons;
use super::password_input::PasswordInput;
use super::LoginMode;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;

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

            <div class="relative my-4">
                <div class="absolute inset-0 flex items-center">
                    <div class="w-full border-t border-[var(--border-color)]"></div>
                </div>
            </div>

            <OAuthButtons />

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
