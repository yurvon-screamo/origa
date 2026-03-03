use super::LoginMode;
use super::error_message::ErrorMessage;
use crate::app::AuthContext;
use crate::ui_components::{Alert, AlertType, Button, ButtonVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;

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
