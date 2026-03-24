use crate::ui_components::{Card, Dropdown, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

use super::super::onboarding_state::OnboardingState;
use super::duolingo_helpers::{
    build_module_items, build_unit_items, collect_units_to_import, format_import_info,
    is_unit_in_modules,
};
use super::types::DuolingoModule;

#[component]
pub fn DuolingoProgressSelector(
    app_id: String,
    is_ru: bool,
    modules: Vec<DuolingoModule>,
    state: RwSignal<OnboardingState>,
) -> impl IntoView {
    let selected_module = RwSignal::new(None::<usize>);
    let selected_unit = RwSignal::new(None::<usize>);
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let module_items = build_module_items(&modules);

    let modules_for_unit_items = modules.clone();
    let unit_items =
        Signal::derive(move || build_unit_items(&modules_for_unit_items, selected_module.get()));

    let import_info =
        Signal::derive(move || format_import_info(selected_module.get(), selected_unit.get()));

    let modules_for_effect = modules.clone();
    let app_id_for_effect = app_id;
    Effect::new(move |_| {
        let module_num = selected_module.get();
        let unit_num = selected_unit.get();
        let aid = app_id_for_effect.clone();
        let mods = modules_for_effect.clone();
        let sets = available_sets.get();

        if let (Some(m), Some(u)) = (module_num, unit_num)
            && let Some(module) = mods.iter().find(|mod_| mod_.module_number == m)
        {
            let units_to_import = collect_units_to_import(module, u);

            state.update(|s| {
                s.set_app_selection(&aid, &format!("module_{}_unit_{}", m, u));
                s.sets_to_import
                    .retain(|set| !is_unit_in_modules(set.id.as_str(), &mods));
                let sets_to_add: Vec<_> = sets
                    .iter()
                    .filter(|set_meta| units_to_import.contains(&set_meta.id))
                    .cloned()
                    .collect();
                for set_meta in sets_to_add {
                    s.add_set_to_import(set_meta);
                }
            });
        }
    });

    let selected_module_value = RwSignal::new(
        selected_module
            .get()
            .map(|n| format!("module_{}", n))
            .unwrap_or_else(|| "none".to_string()),
    );
    let selected_unit_value = RwSignal::new(
        selected_unit
            .get()
            .map(|n| format!("unit_{}", n))
            .unwrap_or_else(|| "none".to_string()),
    );

    Effect::new(move |_| {
        let val = selected_module_value.get();
        selected_module.set(
            val.strip_prefix("module_")
                .and_then(|s| s.parse::<usize>().ok()),
        );
    });

    Effect::new(move |_| {
        let val = selected_unit_value.get();
        selected_unit.set(
            val.strip_prefix("unit_")
                .and_then(|s| s.parse::<usize>().ok()),
        );
    });

    let app_label = if is_ru {
        "Duolingo (RU)"
    } else {
        "Duolingo (EN)"
    };

    view! {
        <Card class=Signal::derive(|| "p-4".to_string())>
            <Text size=TextSize::Default variant=TypographyVariant::Primary>
                {app_label}
            </Text>

            <div class="mt-4 space-y-4">
                <div>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        "Модуль"
                    </Text>
                    <div class="mt-2">
                        <Dropdown
                            _options=Signal::derive(move || module_items.clone())
                            _selected=selected_module_value
                            _placeholder=Signal::derive(|| "Выберите модуль".to_string())
                        />
                    </div>
                </div>

                <Show when=move || selected_module.get().is_some()>
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Раздел"
                        </Text>
                        <div class="mt-2">
                            <Dropdown
                                _options=unit_items
                                _selected=selected_unit_value
                                _placeholder=Signal::derive(|| "Выберите раздел".to_string())
                            />
                        </div>
                    </div>
                </Show>

                <Show when=move || import_info.get().is_some()>
                    <div class="mt-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {move || import_info.get().unwrap_or_default()}
                        </Text>
                    </div>
                </Show>
            </div>
        </Card>
    }
}
