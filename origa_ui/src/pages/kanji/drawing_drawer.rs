use crate::ui_components::{Drawer, KanjiDrawingPractice};
use leptos::prelude::*;

#[component]
pub fn DrawingDrawer(kanji: String, is_open: RwSignal<bool>) -> impl IntoView {
    let kanji_for_title = kanji.clone();
    let title = Signal::derive(move || format!("Прописи: {}", kanji_for_title));
    let kanji_stored = StoredValue::new(kanji);

    view! {
        <Drawer is_open=is_open title=title>
            <KanjiDrawingPractice kanji=kanji_stored.get_value() />
        </Drawer>
    }
}
