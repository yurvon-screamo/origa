use leptos::ev::MouseEvent;
use leptos::prelude::*;

use crate::ui_components::{
    Button, ButtonVariant, Card, Heading, HeadingLevel, ProgressBar, Text, TypographyVariant,
};

#[component]
pub fn UpdateDrawer(
    #[prop(optional, into)] test_id: Signal<String>,
    current_version: String,
    new_version: String,
    on_update: Callback<()>,
    download_progress: Signal<Option<f32>>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let update_btn_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            String::new()
        } else {
            format!("{}-update", val)
        }
    });

    let progress_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            String::new()
        } else {
            format!("{}-progress", val)
        }
    });

    let progress_value = RwSignal::new(0u32);

    Effect::new(move |_| {
        if let Some(progress) = download_progress.get() {
            progress_value.set((progress * 100.0) as u32);
        }
    });

    let is_downloading = move || download_progress.get().is_some();

    view! {
        <div
            class="update-drawer-backdrop"
            data-testid=test_id_val
        >
            <Card class="update-drawer-content">
                <div class="update-drawer-body">
                    <Heading level=HeadingLevel::H3 variant=TypographyVariant::Primary>
                        "Доступно обновление"
                    </Heading>

                    <div class="flex items-center gap-3 font-mono text-sm">
                        <span class="text-[var(--fg-muted)]">{current_version}</span>
                        <span class="text-[var(--fg-muted)]">"→"</span>
                        <span class="font-semibold text-[var(--accent-olive)]">{new_version}</span>
                    </div>

                    <Text variant=TypographyVariant::Muted>
                        "Для продолжения работы необходимо обновить приложение."
                    </Text>

                    <Show
                        when=move || !is_downloading()
                        fallback=move || {
                            view! {
                                <div data-testid=progress_test_id>
                                    <ProgressBar
                                        value=progress_value
                                        max=100
                                        label="Загрузка..."
                                    />
                                </div>
                            }
                        }
                    >
                        <Button
                            test_id=update_btn_test_id
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
