use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::WellKnownSets;

use super::set_card::SetCard;
use super::types::{SetInfo, SetType};

#[component]
pub fn SetsTypeGroup(
    set_type: SetType,
    sets_for_level: Memo<Vec<SetInfo>>,
    importing: RwSignal<Option<WellKnownSets>>,
    on_import: Callback<WellKnownSets>,
) -> impl IntoView {
    let sets_for_type = Memo::new(move |_| {
        sets_for_level
            .get()
            .into_iter()
            .filter(|s| s.set_type == set_type)
            .collect::<Vec<_>>()
    });

    view! {
        <Show when=move || !sets_for_type.get().is_empty()>
            <div class="mb-4">
                <Text
                    size=TextSize::Small
                    variant=TypographyVariant::Muted
                    class="mb-2"
                >
                    {set_type.label()}
                </Text>
                <div class="sets-list">
                    <For
                        each=move || sets_for_type.get()
                        key=|s| format!("{:?}", s.set)
                        children=move |set_info| {
                            let is_importing =
                                Memo::new(move |_| importing.get() == Some(set_info.set));

                            view! {
                                <SetCard
                                    set_info=set_info
                                    is_importing=is_importing.get()
                                    on_import=on_import
                                />
                            }
                        }
                    />
                </div>
            </div>
        </Show>
    }
}
