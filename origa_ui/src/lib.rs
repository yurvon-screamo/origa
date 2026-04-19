use tracing::Level;
use tracing_subscriber::{Registry, layer::SubscriberExt};
use tracing_wasm::{ConsoleConfig, WASMLayer, WASMLayerConfigBuilder};

pub mod app;
mod core;
mod hooks;
pub mod i18n;
mod loaders;
mod pages;
mod repository;
mod routes;
mod store;
mod ui_components;
pub mod utils;

pub fn init_tracing() {
    if tracing::dispatcher::has_been_set() {
        return;
    }

    console_error_panic_hook::set_once();

    let mut builder = WASMLayerConfigBuilder::new();

    #[cfg(debug_assertions)]
    {
        builder
            .set_max_level(Level::DEBUG)
            .set_console_config(ConsoleConfig::ReportWithConsoleColor);
    }

    #[cfg(not(debug_assertions))]
    {
        builder
            .set_max_level(Level::INFO)
            .set_console_config(ConsoleConfig::ReportWithoutConsoleColor);
    }

    builder.set_report_logs_in_timings(false);
    let config = builder.build();

    let subscriber = Registry::default().with(WASMLayer::new(config));
    tracing::subscriber::set_global_default(subscriber)
        .expect("Не удалось установить глобальный subscriber для tracing");
}
