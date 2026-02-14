use crate::repository::InMemoryUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::*;
use leptos_router::hooks::use_navigate;
use leptos_router::path;
use origa::domain::User;

#[component]
pub fn AppRoutes() -> impl IntoView {
    view! {
        <main class="min-h-screen paper-texture">
            <Routes fallback=|| view! { <Login/> }>
                <Route path=path!("/") view=Login />
                <Route path=path!("login") view=Login />
                <Route path=path!("home") view=Home />
            </Routes>
        </main>
    }
}

#[component]
fn Login() -> impl IntoView {
    let username = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let current_user = RwSignal::new(None::<User>);
    provide_context(current_user);

    view! {
        <div class="min-h-screen flex items-center justify-center px-4">
            <div class="max-w-md w-full">
                <div class="text-center mb-12">
                    <h1 class="font-serif text-4xl font-light tracking-tight mb-4">
                        "Origa"
                    </h1>
                    <p class="font-mono text-sm text-[var(--fg-muted)]">
                        "Изучение японского языка"
                    </p>
                </div>

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
                                    let name = username.get();
                                    if name.trim().is_empty() {
                                        error.set(Some("Введите имя пользователя".to_string()));
                                    } else {
                                        error.set(None);

                                        let repo = use_context::<InMemoryUserRepository>()
                                            .expect("UserRepository not provided");

                                        let current_user = current_user;
                                        spawn_local(async move {
                                            match repo.find_or_create_user(name) {
                                                Ok(user) => {
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
                        on:click=move |_| {
                            let name = username.get();
                            if name.trim().is_empty() {
                                error.set(Some("Введите имя пользователя".to_string()));
                            } else {
                                error.set(None);

                                let repo = use_context::<InMemoryUserRepository>()
                                    .expect("UserRepository not provided");

                                let current_user = current_user;
                                spawn_local(async move {
                                    match repo.find_or_create_user(name) {
                                        Ok(user) => {
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
                        class="w-full px-6 py-3 bg-[var(--accent-olive)] text-[var(--bg-primary)] font-mono text-xs tracking-widest uppercase hover:opacity-90 transition-opacity"
                    >
                        "Войти"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Home() -> impl IntoView {
    let current_user = use_context::<RwSignal<Option<User>>>();

    let user_display = {
        let current_user = current_user;
        move || {
            current_user.as_ref().and_then(|user| user.get()).map(|u| {
                view! {
                    <p class="font-mono text-sm text-[var(--fg-muted)]">
                        {format!("Пользователь: {}", u.username())}
                    </p>
                }
            })
        }
    };

    view! {
        <div class="min-h-screen">
            <div class="max-w-6xl mx-auto px-6 py-16">
                <header class="mb-12">
                    <div class="flex justify-between items-start">
                        <div>
                            <h1 class="font-serif text-4xl font-light tracking-tight mb-4">
                                "Добро пожаловать"
                            </h1>
                            {user_display}
                        </div>
                        <button
                            on:click=move |_| {
                                let navigate = leptos_router::hooks::use_navigate();
                                navigate("/login", Default::default());
                            }
                            class="px-4 py-2 bg-[var(--bg-primary)] border border-[var(--border-color)] font-mono text-xs tracking-widest uppercase hover:border-[var(--accent-olive)] transition-colors"
                        >
                            "Выход"
                        </button>
                    </div>
                </header>

                <div class="grid gap-8">
                    <div class="p-8 bg-[var(--bg-primary)] border border-[var(--border-color)]">
                        <h2 class="font-serif text-2xl font-light mb-4">"Обзор"</h2>
                        <p class="font-mono text-sm text-[var(--fg-muted)]">
                            "Здесь будет статистика вашего прогресса"
                        </p>
                    </div>

                    <div class="grid md:grid-cols-3 gap-6">
                        <div class="p-6 bg-[var(--bg-primary)] border border-[var(--border-color)]">
                            <h3 class="font-serif text-lg font-light mb-2">"Слова"</h3>
                            <p class="font-mono text-xs text-[var(--fg-muted)]">
                                "Библиотека слов"
                            </p>
                        </div>
                        <div class="p-6 bg-[var(--bg-primary)] border border-[var(--border-color)]">
                            <h3 class="font-serif text-lg font-light mb-2">"Кандзи"</h3>
                            <p class="font-mono text-xs text-[var(--fg-muted)]">
                                "Библиотека кандзи"
                            </p>
                        </div>
                        <div class="p-6 bg-[var(--bg-primary)] border border-[var(--border-color)]">
                            <h3 class="font-serif text-lg font-light mb-2">"Грамматика"</h3>
                            <p class="font-mono text-xs text-[var(--fg-muted)]">
                                "Грамматические конструкции"
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
