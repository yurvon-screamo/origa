use leptos::prelude::*;
use leptos_router::components::*;

pub fn AppRoutes() -> impl IntoView {
    view! {
        <main class="min-h-screen paper-texture">
            <Routes>
                <Route path="/" view=Login />
                <Route path="/login" view=Login />
                <Route path="/home" view=Home />
            </Routes>
        </main>
    }
}

#[component]
fn Login() -> impl IntoView {
    let username = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);

    let handle_login = move |_| {
        let name = username.get();
        if name.trim().is_empty() {
            error.set(Some("Введите имя пользователя".to_string()));
        } else {
            error.set(None);
            let navigate = leptos_router::hooks::use_navigate();
            navigate("/home", Default::default());
        }
    };

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
                                    handle_login(ev);
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
                        on:click=handle_login
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
    view! {
        <div class="min-h-screen">
            <div class="max-w-6xl mx-auto px-6 py-16">
                <header class="mb-12">
                    <h1 class="font-serif text-4xl font-light tracking-tight mb-4">
                        "Добро пожаловать"
                    </h1>
                    <p class="font-mono text-sm text-[var(--fg-muted)]">
                        "Главная страница приложения"
                    </p>
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
