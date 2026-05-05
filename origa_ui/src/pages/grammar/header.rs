use super::add_grammar_modal::AddGrammarModal;
use crate::i18n::use_i18n;
use crate::ui_components::{Button, ButtonVariant, PageHeader};
use leptos::prelude::*;

#[component]
pub fn GrammarHeader(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let is_modal_open = RwSignal::new(false);

    view! {
        <PageHeader
            back_path="/home".to_string()
            back_label=Signal::derive(move || i18n.get_keys().common().back().inner().to_string())
            title=Signal::derive(move || i18n.get_keys().grammar_page().header().inner().to_string())
            test_id="grammar"
        >
            <Button
                variant=ButtonVariant::Olive
                test_id="grammar-add-btn"
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    is_modal_open.set(true);
                })
            >
                "+"
            </Button>
        </PageHeader>

        <AddGrammarModal is_open=is_modal_open refresh_trigger=refresh_trigger />
    }
}
