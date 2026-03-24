use crate::ui_components::DropdownItem;

use super::types::DuolingoModule;

pub fn build_module_items(modules: &[DuolingoModule]) -> Vec<DropdownItem> {
    let mut items = vec![DropdownItem {
        value: "none".to_string(),
        label: "Не изучал".to_string(),
    }];
    for module in modules {
        items.push(DropdownItem {
            value: format!("module_{}", module.module_number),
            label: format!("Модуль {}", module.module_number),
        });
    }
    items
}

pub fn build_unit_items(
    modules: &[DuolingoModule],
    selected_module_num: Option<usize>,
) -> Vec<DropdownItem> {
    let mut items = vec![DropdownItem {
        value: "none".to_string(),
        label: "Не изучал".to_string(),
    }];

    if let Some(num) = selected_module_num
        && let Some(module) = modules.iter().find(|m| m.module_number == num)
    {
        for unit in &module.units {
            items.push(DropdownItem {
                value: format!("unit_{}", unit.unit_number),
                label: format!("Раздел {}", unit.unit_number),
            });
        }
    }
    items
}

pub fn format_import_info(module_num: Option<usize>, unit_num: Option<usize>) -> Option<String> {
    match (module_num, unit_num) {
        (Some(m), Some(u)) => Some(format!("Будут импортированы: Модуль {} Разделы 1-{}", m, u)),
        (Some(m), None) => Some(format!("Выберите раздел в Модуле {}", m)),
        _ => None,
    }
}

pub fn collect_units_to_import(module: &DuolingoModule, unit_limit: usize) -> Vec<String> {
    module
        .units
        .iter()
        .filter(|unit| unit.unit_number <= unit_limit)
        .map(|unit| unit.id.clone())
        .collect()
}

pub fn is_unit_in_modules(unit_id: &str, modules: &[DuolingoModule]) -> bool {
    modules
        .iter()
        .any(|m| m.units.iter().any(|u| u.id == unit_id))
}
