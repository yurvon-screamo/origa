use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Input, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;

use super::password_input::PasswordInput;
use super::validation;

#[component]
pub fn EmailPasswordForm(
    #[prop(optional, into)] test_id: Signal<String>,
    on_submit: Callback<(String, String)>,
) -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let handle_submit = move || {
        let email_val = email.get();
        let password_val = password.get();

        if let Err(e) = validation::validate_credentials(&email_val, &password_val) {
            error.set(Some(e));
            return;
        }

        error.set(None);
        on_submit.run((email_val, password_val));
    };

    let on_submit_form = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        handle_submit();
    };

    let form_test_id = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    let error_test_id = Signal::derive(move || {
        let base = test_id.get();
        if base.is_empty() {
            "login-error".to_string()
        } else {
            format!("{}-error", base)
        }
    });

    view! {
        <form class="space-y-4" on:submit=on_submit_form data-testid=form_test_id>
            <Show when=move || error.get().is_some()>
                <Alert
                    test_id=error_test_id
                    alert_type=Signal::derive(|| AlertType::Error)
                    message=Signal::derive(move || error.get().unwrap_or_default())
                />
            </Show>

            <div>
                <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true class="block mb-2">
                    "Email"
                </Text>
                <Input
                    value=email
                    input_type=Signal::derive(|| "email".to_string())
                    autocomplete=Signal::derive(|| "email".to_string())
                    id=Signal::derive(|| "email".to_string())
                    name=Signal::derive(|| "email".to_string())
                    placeholder=Signal::derive(|| "example@mail.com".to_string())
                    test_id=Signal::derive(|| "email-input".to_string())
                />
            </div>

            <PasswordInput
                value=password
                id=Signal::derive(|| "password".to_string())
                name=Signal::derive(|| "password".to_string())
                test_id=Signal::derive(|| "password-input".to_string())
            />

            <Button
                variant=Signal::derive(|| ButtonVariant::Olive)
                loading=loading
                disabled=loading
                button_type=Signal::derive(|| "submit".to_string())
                class=Signal::derive(|| "w-full".to_string())
                test_id=Signal::derive(|| "login-submit".to_string())
            >
                "Войти"
            </Button>
        </form>
    }
}
