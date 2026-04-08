use crate::i18n::use_i18n;
use crate::ui_components::{Drawer, KanjiDrawingPractice};
use leptos::prelude::*;

#[component]
pub fn DrawingDrawer(kanji: String, is_open: RwSignal<bool>) -> impl IntoView {
    let i18n = use_i18n();
    let kanji_for_title = kanji.clone();
    let title = Signal::derive(move || {
        i18n.get_keys()
            .kanji_page()
            .worksheets()
            .inner()
            .to_string()
            .replacen("{}", &kanji_for_title, 1)
    });
    let kanji_stored = StoredValue::new(kanji);

    view! {
        <Drawer is_open=is_open title=title>
            <KanjiDrawingPractice kanji=kanji_stored.get_value() />
        </Drawer>
    }
}
