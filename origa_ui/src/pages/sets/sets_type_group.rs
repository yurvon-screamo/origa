use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use lexical_sort::natural_lexical_cmp;
use origa::traits::SetType;

use super::filters::TypeFilter;
use super::set_card::SetCard;
use super::types::SetInfo;

#[component]
pub fn SetsTypeGroup(
    set_type: SetType,
    sets_for_level: Memo<Vec<SetInfo>>,
    type_filter: RwSignal<TypeFilter>,
    on_import: Callback<(String, String)>,
) -> impl IntoView {
    let sets_for_type = Memo::new(move |_| {
        let current_filter = type_filter.get();
        let mut sets: Vec<_> = sets_for_level
            .get()
            .into_iter()
            .filter(|s| s.set_type == set_type && current_filter.matches(set_type))
            .collect();

        sets.sort_by(|a, b| natural_lexical_cmp(&a.title, &b.title));
        sets
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
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <For
                        each=move || sets_for_type.get()
                        key=|s| s.set_id.clone()
                        children=move |set_info| {
                            view! {
                                <SetCard
                                    set_info=set_info
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
