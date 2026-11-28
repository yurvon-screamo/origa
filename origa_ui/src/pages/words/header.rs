use super::add_words_preview_modal::AddWordsPreviewModal;
use crate::i18n::{Locale, t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, PageHeader};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// The words action strip keeps the deliberate two-row grid from #932 for
/// every locale. Only EN/VI need the compact density: their tab labels are
/// wider than the 342px card content at 390px (uwuwu#413).
pub(crate) fn words_actions_class(locale: Locale) -> String {
    let compact = match locale {
        Locale::en | Locale::vi => " words-actions-compact",
        Locale::ru | Locale::ko => "",
    };
    format!("max-md:basis-full max-md:justify-between{compact}")
}

#[component]
pub fn WordsHeader(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let navigate_sets = use_navigate();
    let navigate_phrases = use_navigate();
    let navigate_examples = use_navigate();
    let is_modal_open = RwSignal::new(false);

    view! {
        <PageHeader
            back_path="".to_string()
            back_label=Signal::derive(move || i18n.get_keys().common().back().inner().to_string())
            title=Signal::derive(move || i18n.get_keys().words().header().inner().to_string())
            test_id="words"
            // Below 768px the four actions leave the title's row and take
            // their own full-width row as a tab strip (deliberate grid);
            // above 768px the header stays one line. Pages with fewer
            // actions keep the natural wrap of #823.
            actions_class=Signal::derive(move || words_actions_class(i18n.get_locale()))
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
                variant=ButtonVariant::Ghost
                test_id="words-phrases-btn"
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    navigate_phrases("/phrases", Default::default());
                })
            >
                {t!(i18n, words.phrases)}
            </Button>
            <Button
                variant=ButtonVariant::Ghost
                test_id="words-examples-btn"
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    navigate_examples("/examples", Default::default());
                })
            >
                {t!(i18n, words.examples)}
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
