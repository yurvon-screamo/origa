use leptos::prelude::*;

use super::{Button, ButtonVariant, Modal, Spinner, Text, TextSize, TypographyVariant};

#[component]
pub fn DeleteConfirmModal(
    is_open: RwSignal<bool>,
    is_deleting: Signal<bool>,
    on_confirm: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <Modal
            is_open=is_open
            title=Signal::derive(|| "Удалить карточку?".to_string())
        >
            <div class="space-y-4">
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    "Карточка будет удалена без возможности восстановления."
                </Text>
                <div class="flex gap-2 justify-end">
                    <Button
                        variant=ButtonVariant::Ghost
                        disabled=is_deleting
                        on_click=Callback::new(move |_| on_close.run(()))
                    >
                        "Отмена"
                    </Button>
                    <Button
                        variant=ButtonVariant::Filled
                        disabled=is_deleting
                        on_click=Callback::new(move |_| {
                            on_confirm.run(());
                        })
                    >
                        {move || if is_deleting.get() {
                            view! { <Spinner /> }.into_any()
                        } else {
                            "Удалить".into_any()
                        }}
                    </Button>
                </div>
            </div>
        </Modal>
    }
}
