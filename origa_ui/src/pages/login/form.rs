use crate::repository::InMemoryUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use origa::domain::User;

#[component]
pub fn LoginForm(username: RwSignal<String>, error: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div>
                <label class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-2">
                    "Имя пользователя"
                </label>
                <input
                    type="text"
                    placeholder="Введите имя"
                    value=username
                    class="w-full px-4 py-3 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-none font-mono text-sm focus:outline-none focus:border-[var(--accent-olive)]"
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
                        <div class="p-4 bg-red-50 border border-red-200 text-red-700 font-mono text-xs">
                            {err}
                        </div>
                    }
                })
            }}

            <button
                on:click=move |_| handle_login(username, error)
                class="w-full px-6 py-3 bg-[var(--accent-olive)] text-[var(--bg-primary)] font-mono text-xs tracking-widest uppercase hover:opacity-90 transition-opacity"
            >
                "Войти"
            </button>
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

        spawn_local(async move {
            match repo.find_or_create_user(name) {
                Ok(user) => {
                    let current_user = use_context::<RwSignal<Option<User>>>()
                        .expect("current_user context not provided");
                    current_user.set(Some(user));
                    let navigate = use_navigate();
                    navigate("/home", Default::default());
                }
                Err(e) => {
                    error.set(Some(format!("Ошибка: {}", e)));
                }
            }
        });
    }
}
