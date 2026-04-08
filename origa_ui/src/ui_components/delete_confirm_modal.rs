use crate::i18n::{t, use_i18n};
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
    let i18n = use_i18n();
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
            title=Signal::derive(move || crate::i18n::use_i18n().get_keys().ui().delete_card().inner().to_string())
        >
            <div class="delete-confirm-modal">
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    {t!(i18n, ui.delete_card_message)}
                </Text>
                <div class="delete-confirm-actions">
                    <Button
                        test_id=cancel_test_id
                        variant=ButtonVariant::Ghost
                        disabled=is_deleting
                        on_click=Callback::new(move |_| on_close.run(()))
                    >
                        {t!(i18n, common.cancel)}
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
                            t!(i18n, common.delete).into_any()
                        }}
                    </Button>
                </div>
            </div>
        </Modal>
    }
}
