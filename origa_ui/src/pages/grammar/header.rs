use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn GrammarHeader(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();
    let is_modal_open = RwSignal::new(false);

    view! {
        <div class="flex flex-wrap justify-between items-center gap-4 mb-6">
            <Heading level=HeadingLevel::H1 test_id="grammar-title">
                {t!(i18n, grammar_page.header)}
            </Heading>
            <div class="flex items-center gap-2 sm:gap-4">
                <Button
                    variant=ButtonVariant::Ghost
                    test_id="grammar-back-btn"
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        navigate("/home", Default::default());
                    })
                >
                    {t!(i18n, common.back)}
                </Button>
                <Button
                    variant=ButtonVariant::Olive
                    test_id="grammar-add-btn"
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        is_modal_open.set(true);
                    })
                >
                    "+"
                </Button>
            </div>
        </div>

        <AddGrammarModal is_open=is_modal_open refresh_trigger=refresh_trigger />
    }
}

use super::add_grammar_modal::AddGrammarModal;
