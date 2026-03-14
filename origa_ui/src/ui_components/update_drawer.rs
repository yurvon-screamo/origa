use leptos::ev::MouseEvent;
use leptos::prelude::*;

use crate::ui_components::{
    Button, ButtonVariant, Card, Heading, HeadingLevel, ProgressBar, Text, TypographyVariant,
};

#[component]
pub fn UpdateDrawer(
    current_version: String,
    new_version: String,
    on_update: Callback<()>,
    download_progress: Signal<Option<f32>>,
) -> impl IntoView {
    let progress_value = RwSignal::new(0u32);

    Effect::new(move |_| {
        if let Some(progress) = download_progress.get() {
            progress_value.set((progress * 100.0) as u32);
        }
    });

    let is_downloading = move || download_progress.get().is_some();

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80">
            <Card class="w-full max-w-md mx-4 p-6 bg-[var(--bg-primary)]">
                <div class="space-y-5">
                    <Heading level=HeadingLevel::H3 variant=TypographyVariant::Primary>
                        "Доступно обновление"
                    </Heading>

                    <div class="flex items-center gap-3 font-mono text-sm">
                        <span class="text-[var(--fg-muted)]">{current_version}</span>
                        <span class="text-[var(--fg-muted)]">"→"</span>
                        <span class="text-[var(--accent-olive)] font-semibold">{new_version}</span>
                    </div>

                    <Text variant=TypographyVariant::Muted>
                        "Для продолжения работы необходимо обновить приложение."
                    </Text>

                    <Show
                        when=move || !is_downloading()
                        fallback=move || {
                            view! {
                                <ProgressBar
                                    value=progress_value
                                    max=100
                                    label="Загрузка..."
                                />
                            }
                        }
                    >
                        <Button
                            variant=ButtonVariant::Olive
                            class="w-full"
                            on_click=Callback::new(move |_: MouseEvent| {
                                on_update.run(());
                            })
                        >
                            "Обновить сейчас"
                        </Button>
                    </Show>
                </div>
            </Card>
        </div>
    }
}
