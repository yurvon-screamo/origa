use crate::i18n::{td_string, use_i18n};
use crate::ui_components::{Button, ButtonSize, ButtonVariant};
use leptos::prelude::*;

/// Каркас панели-инсайта (#749): вопрос сверху, одна фраза-вердикт,
/// раскрываемые доказательства. Котобара-стиль: панель отвечает на один
/// вопрос юзера и не требует «декодирования» цифр.
#[component]
pub(crate) fn InsightPanel(
    /// Локализованный вопрос панели.
    #[prop(into)]
    question: Signal<String>,
    /// Локализованная фраза-вердикт (или нейтральный empty-state).
    #[prop(into)]
    verdict: Signal<String>,
    /// Строки доказательств; пустой список — кнопки раскрытия нет.
    #[prop(into)]
    proofs: Signal<Vec<String>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let proofs_open = RwSignal::new(false);

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let proofs_test_id = Signal::derive(move || format!("{}-proofs", test_id.get()));

    view! {
        <div class="card card-shadow p-6" data-testid=test_id_val>
            <div class="flex flex-col gap-3">
                <p class="text-xs text-[var(--fg-muted)] uppercase tracking-widest">
                    {move || question.get()}
                </p>

                <p class="heading-h3 text-[var(--fg-black)]">
                    {move || verdict.get()}
                </p>

                <Show when=move || !proofs.get().is_empty()>
                    <div>
                        <Button
                            variant=ButtonVariant::Ghost
                            size=ButtonSize::Small
                            test_id=proofs_test_id
                            on_click=Callback::new(move |_| proofs_open.update(|open| *open = !*open))
                        >
                            {move || {
                                let locale = i18n.get_locale();
                                if proofs_open.get() {
                                    td_string!(locale, insights.proofs_hide)
                                } else {
                                    td_string!(locale, insights.proofs_show)
                                }
                            }}
                        </Button>

                        <Show when=move || proofs_open.get()>
                            <ul class="mt-3 flex flex-col gap-1 font-mono text-sm text-[var(--fg-muted)]">
                                <For each=move || proofs.get() key=|line| line.clone() let(line)>
                                    <li>{line}</li>
                                </For>
                            </ul>
                        </Show>
                    </div>
                </Show>
            </div>
        </div>
    }
}
