use super::LoginMode;
use super::auth_handlers::handle_register;
use super::email_input::EmailInput;
use super::error_message::ErrorMessage;
use super::password_input::PasswordInput;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;

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
