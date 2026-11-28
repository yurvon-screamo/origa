#![recursion_limit = "512"]

use tracing::Level;
use tracing_subscriber::{Registry, layer::SubscriberExt};

pub mod app;
mod core;
mod feedback;
mod hooks;
pub mod i18n;
mod loaders;
mod pages;
#[cfg(all(target_arch = "wasm32", test))]
mod panic_family_wasm_tests;
mod repository;
mod routes;
mod sentry;
mod store;
#[cfg(all(target_arch = "wasm32", test))]
mod test_support;
mod tracing_console;
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

    // Level-aware console routing (uwuwu#1454; story in the ADR-036
    // addendum): ERROR → console.error + direct Sentry capture. Max level
    // is pinned per build profile exactly as before: dev = DEBUG,
    // release = INFO.
    let max_level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = Registry::default()
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            max_level,
        ))
        .with(tracing_console::ConsoleSentryLayer::new());
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
mod i18n_parity_test;
