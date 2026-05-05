use super::add_words_preview_modal::AddWordsPreviewModal;
use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, PageHeader};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn WordsHeader(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let navigate_sets = use_navigate();
    let is_modal_open = RwSignal::new(false);

    view! {
        <PageHeader
            back_path="/home".to_string()
            back_label=Signal::derive(move || i18n.get_keys().common().back().inner().to_string())
            title=Signal::derive(move || i18n.get_keys().words().header().inner().to_string())
            test_id="words"
        >
            <Button
                variant=ButtonVariant::Ghost
                test_id="words-sets-btn"
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    navigate_sets("/sets", Default::default());
                })
            >
                {t!(i18n, words.decks)}
            </Button>
            <Button
                variant=ButtonVariant::Olive
                test_id="words-add-btn"
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    is_modal_open.set(true);
                })
            >
                "+"
            </Button>
        </PageHeader>

        <AddWordsPreviewModal is_open=is_modal_open refresh_trigger=refresh_trigger />
    }
}
