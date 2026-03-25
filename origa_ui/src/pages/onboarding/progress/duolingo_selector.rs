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
    let selected_module_value = RwSignal::new("none".to_string());
    let selected_unit_value = RwSignal::new("none".to_string());
    let available_sets = Signal::derive(move || state.get().available_sets.clone());

    let module_items = build_module_items(&modules);

    let modules_for_unit_items = modules.clone();
    let unit_items = Signal::derive(move || {
        let module_str = selected_module_value.get();
        let module_num = module_str
            .strip_prefix("module_")
            .and_then(|s| s.parse::<usize>().ok());
        build_unit_items(&modules_for_unit_items, module_num)
    });

    let import_info = Signal::derive(move || {
        let module_str = selected_module_value.get();
        let unit_str = selected_unit_value.get();

        let module_num = module_str
            .strip_prefix("module_")
            .and_then(|s| s.parse::<usize>().ok());
        let unit_num = unit_str
            .strip_prefix("unit_")
            .and_then(|s| s.parse::<usize>().ok());

        format_import_info(module_num, unit_num)
    });

    // Clone app_id before the Effect to avoid ownership issues
    let app_id_for_effect = app_id.clone();

    // Single Effect that handles both module and unit selection
    Effect::new(move |_| {
        let module_str = selected_module_value.get();
        let unit_str = selected_unit_value.get();

        let module_num = module_str
            .strip_prefix("module_")
            .and_then(|s| s.parse::<usize>().ok());
        let unit_num = unit_str
            .strip_prefix("unit_")
            .and_then(|s| s.parse::<usize>().ok());

        if let (Some(m), Some(u)) = (module_num, unit_num)
            && let Some(module) = modules.iter().find(|mod_| mod_.module_number == m)
        {
            // Collect IDs to import - use iterator without cloning the whole vector
            let units_to_import = collect_units_to_import(module, u);

            let mods_ref = modules.clone();
            let sets = available_sets.get();

            state.update(|s| {
                s.set_app_selection(&app_id_for_effect, &format!("module_{}_unit_{}", m, u));
                // Remove old Duolingo sets for this app
                s.sets_to_import
                    .retain(|set| !is_unit_in_modules(set.id.as_str(), &mods_ref));
                // Add new sets
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

    let app_label = if is_ru {
        "Duolingo (RU)"
    } else {
        "Duolingo (EN)"
    };

    let app_id_for_module_dropdown = app_id.clone();
    let app_id_for_unit_dropdown = app_id.clone();

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
                            options=Signal::derive(move || module_items.clone())
                            selected=selected_module_value
                            placeholder=Signal::derive(|| "Выберите модуль".to_string())
                            test_id=Signal::derive({
                                let app_id = app_id_for_module_dropdown.clone();
                                move || format!("{}-module-dropdown", app_id)
                            })
                        />
                    </div>
                </div>

                <Show when=move || selected_module_value.get() != "none">
                    <div>
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            "Раздел"
                        </Text>
                        <div class="mt-2">
                            <Dropdown
                                options=unit_items
                                selected=selected_unit_value
                                placeholder=Signal::derive(|| "Выберите раздел".to_string())
                                test_id=Signal::derive({
                                    let app_id = app_id_for_unit_dropdown.clone();
                                    move || format!("{}-unit-dropdown", app_id)
                                })
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
