pub mod analytics;
#[cfg(all(target_arch = "wasm32", test))]
mod analytics_wasm_tests;
pub mod config;
pub mod device_ai;
pub mod haptics;
pub mod platform;
pub mod tauri;
pub mod updater;
pub mod version;
