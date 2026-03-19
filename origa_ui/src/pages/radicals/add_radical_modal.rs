use super::action_buttons::ActionButtons;
use super::add_radical_modal_handlers::ModalHandlers;
use super::add_radical_modal_state::ModalState;
use super::error_alert::ErrorAlert;
use super::radical_list::RadicalList;
use super::selected_count::SelectedCount;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Button, ButtonSize, ButtonVariant, Drawer, Spinner, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;

#[component]
pub fn AddRadicalModal(is_open: RwSignal<bool>, refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let _repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let state = ModalState::new(is_open, refresh_trigger);
    let handlers = ModalHandlers::new(&state, is_open);

    Effect::new({
        let state = state.clone();
        move |_| {
            if is_open.get() {
                state.load_radicals();
            }
        }
    });

    view! {
        <Drawer
            is_open=is_open
            title=Signal::derive(|| "Добавить радикалы".to_string())
        >
            <div class="space-y-4">
                <div>
                    <div class="flex items-center justify-between mb-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Доступные радикалы"
                        </Text>
                        <Button
                            variant=Signal::derive(|| ButtonVariant::Ghost)
                            size=Signal::derive(|| ButtonSize::Small)
                            on_click=Callback::new({
                                let state = state.clone();
                                move |_| state.select_all()
                            })
                        >
                            "Выделить все"
                        </Button>
                    </div>
                    {move || {
                        let is_loading = state.is_loading_radicals.get();
                        let radical_list = state.available_radicals.get();

                        if is_loading {
                            view! {
                                <div class="flex flex-col items-center py-4 gap-3">
                                    <Spinner />
                                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                        "Поиск радикалов..."
                                    </Text>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <RadicalList
                                    radical_list=radical_list
                                    selected_radicals=state.selected_radicals
                                    known_radicals=std::collections::HashSet::new()
                                />
                            }.into_any()
                        }
                    }}
                </div>

                <SelectedCount count=Signal::derive(move || state.selected_radicals.get().len()) />

                <ErrorAlert message=state.error_message />

                <ActionButtons
                    is_creating=state.is_creating
                    is_disabled=Signal::derive(move || state.selected_radicals.get().is_empty())
                    on_cancel=handlers.on_cancel
                    on_add=handlers.on_add
                />
            </div>
        </Drawer>
    }
}
