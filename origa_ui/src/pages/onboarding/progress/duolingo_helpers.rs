use crate::i18n::{I18nContext, Locale};
use crate::ui_components::DropdownItem;

use super::types::DuolingoModule;

pub fn build_module_items(
    i18n: &I18nContext<Locale>,
    modules: &[DuolingoModule],
) -> Vec<DropdownItem> {
    let mut items = vec![DropdownItem {
        value: "none".to_string(),
        label: i18n
            .get_keys()
            .onboarding()
            .progress()
            .not_studied()
            .inner()
            .to_string(),
    }];
    for module in modules {
        items.push(DropdownItem {
            value: format!("module_{}", module.module_number),
            label: i18n
                .get_keys()
                .onboarding()
                .progress()
                .module_number()
                .inner()
                .to_string()
                .replacen("{}", &module.module_number.to_string(), 1),
        });
    }
    items
}

pub fn build_unit_items(
    i18n: &I18nContext<Locale>,
    modules: &[DuolingoModule],
    selected_module_num: Option<usize>,
) -> Vec<DropdownItem> {
    let mut items = vec![DropdownItem {
        value: "none".to_string(),
        label: i18n
            .get_keys()
            .onboarding()
            .progress()
            .not_studied()
            .inner()
            .to_string(),
    }];

    if let Some(num) = selected_module_num
        && let Some(module) = modules.iter().find(|m| m.module_number == num)
    {
        for unit in &module.units {
            items.push(DropdownItem {
                value: format!("unit_{}", unit.unit_number),
                label: i18n
                    .get_keys()
                    .onboarding()
                    .progress()
                    .section_number()
                    .inner()
                    .to_string()
                    .replacen("{}", &unit.unit_number.to_string(), 1),
            });
        }
    }
    items
}

pub fn format_import_info(
    i18n: &I18nContext<Locale>,
    module_num: Option<usize>,
    unit_num: Option<usize>,
) -> Option<String> {
    match (module_num, unit_num) {
        (Some(1), Some(u)) => Some(
            i18n.get_keys()
                .onboarding()
                .progress()
                .import_module_section()
                .inner()
                .to_string()
                .replacen("{}", &u.to_string(), 1),
        ),
        (Some(m), Some(u)) if m > 1 => Some(
            i18n.get_keys()
                .onboarding()
                .progress()
                .import_modules_section()
                .inner()
                .to_string()
                .replacen("{}", &(m - 1).to_string(), 1)
                .replacen("{}", &m.to_string(), 1)
                .replacen("{}", &u.to_string(), 1),
        ),
        (Some(m), None) => Some(
            i18n.get_keys()
                .onboarding()
                .progress()
                .select_section_in_module()
                .inner()
                .to_string()
                .replacen("{}", &m.to_string(), 1),
        ),
        _ => None,
    }
}

pub fn collect_all_units_to_import(
    modules: &[DuolingoModule],
    selected_module: usize,
    unit_limit: usize,
) -> Vec<String> {
    let mut ids = Vec::new();

    for module in modules {
        if module.module_number < selected_module {
            for unit in &module.units {
                ids.push(unit.id.clone());
            }
        } else if module.module_number == selected_module {
            for unit in &module.units {
                if unit.unit_number <= unit_limit {
                    ids.push(unit.id.clone());
                }
            }
        }
    }

    ids
}

pub fn is_unit_in_modules(unit_id: &str, modules: &[DuolingoModule]) -> bool {
    modules
        .iter()
        .any(|m| m.units.iter().any(|u| u.id == unit_id))
}
