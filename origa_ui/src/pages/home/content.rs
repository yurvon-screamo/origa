use leptos::prelude::*;

#[component]
pub fn HomeContent() -> impl IntoView {
    view! {
        <main class="flex-1">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div class="bg-[var(--bg-secondary)] border border-[var(--border-color)] p-6">
                        <h2 class="font-mono text-xs tracking-widest text-[var(--fg-muted)] uppercase mb-4">
                            "Канжи"
                        </h2>
                        <p class="font-serif text-2xl font-light mb-2">
                            "1,245"
                        </p>
                        <p class="font-mono text-xs text-[var(--fg-muted)]">
                            "изученных символов"
                        </p>
                    </div>

                    <div class="bg-[var(--bg-secondary)] border border-[var(--border-color)] p-6">
                        <h2 class="font-mono text-xs tracking-widest text-[var(--fg-muted)] uppercase mb-4">
                            "Слова"
                        </h2>
                        <p class="font-serif text-2xl font-light mb-2">
                            "3,821"
                        </p>
                        <p class="font-mono text-xs text-[var(--fg-muted)]">
                            "в словаре"
                        </p>
                    </div>

                    <div class="bg-[var(--bg-secondary)] border border-[var(--border-color)] p-6">
                        <h2 class="font-mono text-xs tracking-widest text-[var(--fg-muted)] uppercase mb-4">
                            "Уровень"
                        </h2>
                        <p class="font-serif text-2xl font-light mb-2">
                            "N5"
                        </p>
                        <p class="font-mono text-xs text-[var(--fg-muted)]">
                            "текущий прогресс"
                        </p>
                    </div>
                </div>

                <div class="mt-12">
                    <h2 class="font-mono text-xs tracking-widest text-[var(--fg-muted)] uppercase mb-6">
                        "Сегодня"
                    </h2>
                    <div class="bg-[var(--bg-secondary)] border border-[var(--border-color)] p-6">
                        <p class="font-mono text-sm text-[var(--fg-muted)]">
                            "Начните изучение японского языка"
                        </p>
                    </div>
                </div>
            </div>
        </main>
    }
}
