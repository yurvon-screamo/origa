//! Minimal WASM tracing layer with level-aware console routing.
//!
//! Replaces `tracing-wasm` (0.2.1), which routed every level through
//! `console.log` — the dead delivery story (why native `error!` events
//! never reached Sentry) lives in the ADR-036 addendum.
//!
//! Routing (see [`ConsoleRoute`]):
//! - ERROR → `console.error` + direct Sentry capture via
//!   `crate::sentry::capture_exception`.
//! - WARN → `console.warn` — breadcrumbs.
//! - INFO / DEBUG → `console.info` / `console.debug`.
//!
//! Max level is pinned per build profile: dev = DEBUG, release = INFO
//! (see `lib.rs::init_tracing`).

use tracing::Level;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use wasm_bindgen::JsValue;

/// What the layer does with an event of the given level.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ConsoleRoute {
    /// `console.error` + a direct Sentry event — the revived error channel.
    Error,
    /// `console.warn` — breadcrumbs only.
    Warn,
    /// `console.info`.
    Info,
    /// `console.debug`.
    Debug,
}

/// Pure routing table: tracing level → console route (ADR-086 style).
pub(crate) fn route_level(level: Level) -> ConsoleRoute {
    match level {
        Level::ERROR => ConsoleRoute::Error,
        Level::WARN => ConsoleRoute::Warn,
        Level::INFO => ConsoleRoute::Info,
        Level::DEBUG | Level::TRACE => ConsoleRoute::Debug,
    }
}

/// Browser-online reader — the gate for network-failure severity.
/// `navigator.onLine` at the moment of failure: the same truth
/// `ConnectivityStore::new()` seeds its signal with, readable from
/// background loaders without threading a leptos store through their
/// signatures.
///
/// Host builds (unit tests) answer deterministically "online": `window()`
/// panics off-wasm, and the severity table itself is exercised by the
/// unit tests below, not by the environment.
pub(crate) fn is_browser_online() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .map(|w| w.navigator().on_line())
            .unwrap_or(true)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        true
    }
}

/// Single severity gate for the network invariant (uwuwu#1454, ADR-079
/// style): a failed load-bearing fetch while online is an incident
/// (`error!` — a Sentry event via the capture bridge), the same failure
/// offline is expected degradation (`warn!` — breadcrumbs). One helper for
/// every call site so the message shape and the level logic cannot drift
/// apart.
pub(crate) fn report_network_failure(message: &str) {
    if is_browser_online() {
        tracing::error!("{message}");
    } else {
        tracing::warn!("{message} (offline)");
    }
}

/// Collects a tracing event's fields: the `message` field plus any
/// structured fields rendered as `key=value` suffixes.
#[derive(Default)]
struct FieldVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}"));
        } else {
            self.fields
                .push((field.name().to_owned(), format!("{value:?}")));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_owned());
        } else {
            self.fields
                .push((field.name().to_owned(), value.to_owned()));
        }
    }
}

impl FieldVisitor {
    /// One-line rendering: `message key1=value1 key2=value2`.
    fn render(&self) -> String {
        let mut text = self
            .message
            .clone()
            .unwrap_or_else(|| "(no message)".to_owned());
        for (k, v) in &self.fields {
            text.push_str(&format!(" {k}={v}"));
        }
        text
    }
}

/// The level-aware console + Sentry layer. Max-level filtering happens in
/// the subscriber stack (`LevelFilter`), mirroring the previous
/// `WASMLayerConfigBuilder::set_max_level` contract.
pub(crate) struct ConsoleSentryLayer;

impl ConsoleSentryLayer {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl<S: tracing::Subscriber> Layer<S> for ConsoleSentryLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let route = route_level(*event.metadata().level());

        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);
        let mut text = visitor.render();

        // Source locator: the Sentry event's stack points at this layer's
        // capture call, not at the `error!` site — without the locator the
        // originating line is unrecoverable from the event (uwuwu#1454).
        let meta = event.metadata();
        match (meta.file(), meta.line()) {
            (Some(f), Some(l)) => text.push_str(&format!(" [{f}:{l}]")),
            (Some(f), None) => text.push_str(&format!(" [{f}]")),
            _ => {},
        }

        match route {
            ConsoleRoute::Error => {
                web_sys::console::error_1(&JsValue::from_str(&text));
                // A real Error object via the eager captureException export
                // → the event carries a stack for grouping (ADR-036).
                crate::sentry::capture_exception(&text);
            },
            ConsoleRoute::Warn => web_sys::console::warn_1(&JsValue::from_str(&text)),
            ConsoleRoute::Info => web_sys::console::info_1(&JsValue::from_str(&text)),
            ConsoleRoute::Debug => web_sys::console::debug_1(&JsValue::from_str(&text)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Routing table pinned: ERROR captures, WARN breadcrumbs-only,
    /// INFO/DEBUG plain console.
    #[test]
    fn route_table_is_pinned() {
        assert_eq!(route_level(Level::ERROR), ConsoleRoute::Error);
        assert_eq!(route_level(Level::WARN), ConsoleRoute::Warn);
        assert_eq!(route_level(Level::INFO), ConsoleRoute::Info);
        assert_eq!(route_level(Level::DEBUG), ConsoleRoute::Debug);
        assert_eq!(route_level(Level::TRACE), ConsoleRoute::Debug);
    }

    /// Field rendering: message first, structured fields appended.
    #[test]
    fn visitor_renders_message_and_fields() {
        let v = FieldVisitor {
            message: Some("Failed to load chunk 3".to_owned()),
            fields: vec![("chunk_id".to_owned(), "3".to_owned())],
        };
        assert_eq!(v.render(), "Failed to load chunk 3 chunk_id=3");
    }

    /// Missing message degrades gracefully (never panics in the layer).
    #[test]
    fn visitor_without_message_renders_placeholder() {
        let v = FieldVisitor::default();
        assert_eq!(v.render(), "(no message)");
    }
}

/// The network-invariant gate pinned live (uwuwu#1454 review round 2: the
/// table above is pure — these tests drive the actual gate through both
/// navigator states). Run on the wasm target only: `OnlineOverride` needs
/// a real `navigator`.
#[cfg(all(target_arch = "wasm32", test))]
mod wasm_tests {
    use super::*;
    use crate::core::log_capture;
    use crate::core::tauri_mock::OnlineOverride;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn report_network_failure_online_is_error() {
        let _online = OnlineOverride::install(true);
        log_capture::install();
        log_capture::take_events();
        report_network_failure("X fetch failed: chunk 7, boom");
        let events = log_capture::take_events();
        assert_eq!(events.len(), 1, "exactly one event online, got {events:?}");
        assert_eq!(
            events[0].0,
            tracing::Level::ERROR,
            "online failure = incident"
        );
        assert!(events[0].1.contains("X fetch failed"), "message preserved");
        assert!(
            !events[0].1.contains("(offline)"),
            "online message must not carry the offline marker"
        );
    }

    #[wasm_bindgen_test]
    fn report_network_failure_offline_is_warn_with_marker() {
        let _offline = OnlineOverride::install(false);
        log_capture::install();
        log_capture::take_events();
        report_network_failure("X fetch failed: chunk 7, boom");
        let events = log_capture::take_events();
        assert_eq!(events.len(), 1, "exactly one event offline, got {events:?}");
        assert_eq!(
            events[0].0,
            tracing::Level::WARN,
            "offline failure = expected degradation"
        );
        assert!(
            events[0].1.contains("(offline)"),
            "offline message carries the marker: {}",
            events[0].1
        );
    }
}
