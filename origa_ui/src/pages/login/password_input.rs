use crate::ui_components::{Input, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn PasswordInput(
    value: RwSignal<String>,
    #[prop(optional, into)] autocomplete: Signal<String>,
    #[prop(optional, into)] id: Signal<String>,
    #[prop(optional, into)] name: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let show_password = RwSignal::new(false);

    let toggle_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            String::new()
        } else {
            format!("{}-toggle", val)
        }
    });

    let autocomplete_val = move || {
        let a = autocomplete.get();
        if a.is_empty() {
            "current-password".to_string()
        } else {
            a
        }
    };

    view! {
        <div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true class="block mb-2">
                "Пароль"
            </Text>
            <div class="relative group">
                <Input
                    value=value
                    input_type=move || if show_password.get() { "text" } else { "password" }
                    autocomplete=Signal::derive(autocomplete_val)
                    id=id
                    name=name
                    test_id=test_id
                    on_change=Callback::new(move |ev: leptos::ev::Event| {
                        value.set(event_target_value(&ev));
                    })
                />
                <button
                    type="button"
                    class="absolute right-3 top-1/2 -translate-y-1/2 p-1 text-[var(--fg-muted)] hover:text-[var(--fg-black)] transition-colors"
                    data-testid=move || {
                        let val = toggle_test_id.get();
                        if val.is_empty() { None } else { Some(val) }
                    }
                    on:click=move |_| show_password.set(!show_password.get())
                >
                    <PasswordVisibilityIcon show=show_password />
                </button>
            </div>
        </div>
    }
}

#[component]
fn PasswordVisibilityIcon(show: RwSignal<bool>) -> impl IntoView {
    view! {
        {move || if show.get() {
            view! {
                <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" viewBox="0 0 24 24">
                    <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" />
                    <line x1="1" y1="1" x2="23" y2="23" />
                </svg>
            }.into_any()
        } else {
            view! {
                <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" viewBox="0 0 24 24">
                    <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                    <circle cx="12" cy="12" r="3" />
                </svg>
            }.into_any()
        }}
    }
}
