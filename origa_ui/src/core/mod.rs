pub mod analytics;
#[cfg(all(target_arch = "wasm32", test))]
mod analytics_wasm_tests;
pub mod capabilities;
pub mod config;
pub mod device_ai;
pub mod file_picker;
pub mod haptics;
pub mod platform;
pub mod share_intake;
pub mod shortcut_links;
pub mod tauri;
pub mod updater;
pub mod version;
