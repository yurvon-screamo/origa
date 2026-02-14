use crate::repository::InMemoryUserRepository;
use crate::ui_components::{Button, ButtonVariant, Input, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use origa::domain::User;

#[component]
pub fn LoginForm(username: RwSignal<String>, error: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        <div class="space-y-5">
            <div>
                <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true class="block mb-2">
                    "Имя пользователя"
                </Text>
                <Input
                    value=username
                    placeholder="Введите имя"
                    on_change=Callback::new(move |ev: leptos::ev::Event| {
                        username.set(event_target_value(&ev));
                    })
                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                        if ev.key() == "Enter" {
                            handle_login(username, error);
                        }
                    }
                />
            </div>

            {move || {
                error.get().map(|err| {
                    view! {
                        <div class="p-3 bg-red-950/20 border border-red-900/30 text-red-400 font-mono text-xs">
                            {err}
                        </div>
                    }
                })
            }}

            <Button
                variant=ButtonVariant::Olive
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    handle_login(username, error);
                })
            >
                "Войти"
            </Button>
        </div>
    }
}

fn handle_login(username: RwSignal<String>, error: RwSignal<Option<String>>) {
    let name = username.get();
    if name.trim().is_empty() {
        error.set(Some("Введите имя пользователя".to_string()));
    } else {
        error.set(None);

        let repo = use_context::<InMemoryUserRepository>().expect("UserRepository not provided");
        let current_user =
            use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
        let navigate = use_navigate();

        spawn_local(async move {
            match repo.find_or_create_user(name) {
                Ok(user) => {
                    current_user.set(Some(user));
                    navigate("/home", Default::default());
                }
                Err(e) => {
                    error.set(Some(format!("Ошибка: {}", e)));
                }
            }
        });
    }
}
