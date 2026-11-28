//! Minimal WASM tracing layer with level-aware console routing.
//!
//! Replaces the `tracing-wasm` crate (0.2.1), which routes ALL levels —
//! including ERROR — through `console.log`. That is why native `error!`
//! escalations never reached Sentry (audit: task uwuwu#1454): the console
//! capture path keys on `console.error`, and this crate never produced one.
//!
//! Routing (see [`ConsoleRoute`]):
//! - ERROR → `console.error` **+ direct Sentry capture** via
//!   `crate::sentry::capture_exception` — the eager `captureException`
//!   export is the one delivery surface that survived every SDK major
//!   (proven in production by the panic bridge), unlike the
//!   `captureConsoleIntegration` chunk, which silently stopped being
//!   eagerly exported in SDK v10.
//! - WARN → `console.warn` — breadcrumbs via the eager Breadcrumbs
//!   integration.
//! - INFO / DEBUG → `console.info` / `console.debug`.
//!
//! Max level is pinned per build profile exactly like the previous setup:
//! dev = DEBUG, release = INFO (see `lib.rs::init_tracing`).

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

/// Severity verdict for a failed background network operation
/// (ADR-079-style invariant for the network path): a failure while online
/// of a load-bearing fetch is an incident (Sentry event); offline failures
/// and best-effort steps are expected degradation (breadcrumbs only).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum NetworkSeverity {
    /// Online failure of a load-bearing fetch — `error!`.
    Error,
    /// Offline, or a best-effort step — `warn!`.
    Warn,
}

/// Pure severity table for background network failures.
pub(crate) fn network_failure_severity(best_effort: bool, browser_online: bool) -> NetworkSeverity {
    if best_effort || !browser_online {
        NetworkSeverity::Warn
    } else {
        NetworkSeverity::Error
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
        let text = visitor.render();

        match route {
            ConsoleRoute::Error => {
                web_sys::console::error_1(&JsValue::from_str(&text));
                // The revived channel: a real Error object goes to Sentry's
                // eager captureException, so the event carries a stack for
                // grouping — same contract the panic bridge proves in
                // production (RUST-Q/P symbolized frames).
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

    /// Severity table pinned (ADR-079-style invariant, network path):
    /// online + load-bearing = error; offline or best-effort = warn.
    #[test]
    fn network_severity_table_is_pinned() {
        // Online, load-bearing → incident.
        assert_eq!(
            network_failure_severity(false, true),
            NetworkSeverity::Error
        );
        // Offline — expected degradation regardless of weight.
        assert_eq!(
            network_failure_severity(false, false),
            NetworkSeverity::Warn
        );
        // Best-effort steps stay breadcrumbs even when online.
        assert_eq!(network_failure_severity(true, true), NetworkSeverity::Warn);
        assert_eq!(network_failure_severity(true, false), NetworkSeverity::Warn);
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
