use leptos::prelude::*;

use super::{Button, ButtonVariant, Modal, Spinner, Text, TextSize, TypographyVariant};

#[component]
pub fn DeleteConfirmModal(
    #[prop(optional, into)] test_id: Signal<String>,
    is_open: RwSignal<bool>,
    is_deleting: Signal<bool>,
    on_confirm: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    let cancel_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            String::new()
        } else {
            format!("{}-cancel", val)
        }
    });

    let confirm_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            String::new()
        } else {
            format!("{}-confirm", val)
        }
    });
    view! {
        <Modal
            test_id=test_id
            is_open=is_open
            title=Signal::derive(|| "Удалить карточку?".to_string())
        >
            <div class="delete-confirm-modal">
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    "Карточка будет удалена без возможности восстановления."
                </Text>
                <div class="delete-confirm-actions">
                    <Button
                        test_id=cancel_test_id
                        variant=ButtonVariant::Ghost
                        disabled=is_deleting
                        on_click=Callback::new(move |_| on_close.run(()))
                    >
                        "Отмена"
                    </Button>
                    <Button
                        test_id=confirm_test_id
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
