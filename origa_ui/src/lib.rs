#![recursion_limit = "512"]

use tracing::Level;
use tracing_subscriber::{Registry, layer::SubscriberExt};
use tracing_wasm::{ConsoleConfig, WASMLayer, WASMLayerConfigBuilder};

pub mod app;
mod core;
mod feedback;
mod hooks;
pub mod i18n;
mod loaders;
mod pages;
mod repository;
mod routes;
mod sentry;
mod store;
#[cfg(all(target_arch = "wasm32", test))]
mod test_support;
mod ui_components;
pub mod utils;

pub fn init_tracing() {
    if tracing::dispatcher::has_been_set() {
        return;
    }

    console_error_panic_hook::set_once();

    // Wrap the console panic hook so Rust panics are first forwarded to
    // Sentry, then logged to the browser console. `set_once` uses an internal
    // `Once`, so calling `take_hook` after it returns the console hook; we
    // re-wrap it with a Sentry-capturing closure. See ADR-036 §5.
    let console_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        sentry::capture_exception(&info.to_string());
        console_hook(info);
    }));

    sentry::init();

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

/// Inject the Umami Cloud analytics tracker into `<head>`.
///
/// Must run before the app mounts so auto-tracking catches the initial
/// pageview. No-op in builds compiled with `UMAMI_DISABLED=1` (CI) — see
/// ADR-054 and `core::analytics`.
pub fn init_analytics() {
    core::analytics::inject_umami();
}

#[cfg(test)]
mod i18n_counter_keys_test;
