use super::action_buttons::ActionButtons;
use super::add_kanji_modal_handlers::ModalHandlers;
use super::add_kanji_modal_state::ModalState;
use super::error_alert::ErrorAlert;
use super::kanji_list::KanjiList;
use super::level_selector::LevelSelector;
use super::selected_count::SelectedCount;
use crate::ui_components::{Modal, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

const JLPT_LEVELS: [JapaneseLevel; 5] = [
    JapaneseLevel::N5,
    JapaneseLevel::N4,
    JapaneseLevel::N3,
    JapaneseLevel::N2,
    JapaneseLevel::N1,
];

#[component]
pub fn AddKanjiModal(is_open: RwSignal<bool>) -> impl IntoView {
    let state = ModalState::new(is_open);
    let handlers = ModalHandlers::new(&state, is_open);

    Effect::new({
        let state = state.clone();
        move |_| {
            if is_open.get() {
                state.load_kanji();
            }
        }
    });

    view! {
        <Modal
            is_open=is_open
            title=Signal::derive(|| "Добавить кандзи".to_string())
        >
            <div class="space-y-4">
                <LevelSelector
                    levels=JLPT_LEVELS.to_vec()
                    selected_level=state.selected_level
                    on_select={let state = state.clone(); Callback::new(move |level| state.select_level(level))}
                />

                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(|| "mb-2".to_string())>
                        "Доступные кандзи"
                    </Text>
                    {move || {
                        let is_loading = state.is_loading_kanji.get();
                        let kanji_list = state.available_kanji.get();

                        if is_loading {
                            view! {
                                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                    "Загрузка..."
                                </Text>
                            }.into_any()
                        } else {
                            view! {
                                <KanjiList
                                    kanji_list=kanji_list
                                    selected_kanji=state.selected_kanji
                                />
                            }.into_any()
                        }
                    }}
                </div>

                <SelectedCount count=Signal::derive(move || state.selected_kanji.get().len()) />

                <ErrorAlert message=state.error_message />

                <ActionButtons
                    is_creating=state.is_creating
                    is_disabled=Signal::derive(move || state.selected_kanji.get().is_empty())
                    on_cancel=handlers.on_cancel
                    on_add=handlers.on_add
                />
            </div>
        </Modal>
    }
}
