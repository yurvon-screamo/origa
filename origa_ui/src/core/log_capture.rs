//! Test-only tracing subscriber that captures emitted events for assertions.
//!
//! The Sentry escalation this suite pins (task 20261104051142-inn7qka) works
//! through log levels: `tracing::error!` reaches Sentry as an event via the
//! console bridge, `warn!`/`info!` stay breadcrumbs. A regression that quietly
//! downgrades a native-path failure back to `warn!` would be invisible to
//! behaviour-only tests (the return value — `None`/fallback — is identical),
//! so these tests assert on the captured event levels instead.
//!
//! Install once per test binary with [`install`]; read and clear the buffer
//! with [`take_events`]. The subscriber is process-global on purpose: events
//! emitted across `.await` points escape `with_default` scopes (the poll
//! resumes from the JS event loop), so a global default subscriber is the
//! only reliable capture point for async code paths.

#![cfg(all(target_arch = "wasm32", test))]

use std::sync::{Arc, Mutex, OnceLock};

use tracing::{Event, Level, Subscriber};

static CAPTURE: OnceLock<Arc<EventLog>> = OnceLock::new();

/// Shared event buffer behind the installed subscriber.
struct EventLog {
    events: Mutex<Vec<(Level, String)>>,
}

impl EventLog {
    fn push(&self, level: Level, message: String) {
        if let Ok(mut guard) = self.events.lock() {
            guard.push((level, message));
        }
    }
}

struct CaptureSubscriber {
    log: Arc<EventLog>,
}

struct MessageVisitor {
    message: Option<String>,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}"));
        }
    }
}

impl Subscriber for CaptureSubscriber {
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }

    fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut visitor = MessageVisitor { message: None };
        event.record(&mut visitor);
        self.log.push(
            *event.metadata().level(),
            visitor.message.unwrap_or_default(),
        );
    }

    fn enter(&self, _span: &tracing::span::Id) {}

    fn exit(&self, _span: &tracing::span::Id) {}
}

/// Installs the capturing subscriber as the process-global default.
///
/// Idempotent: the first call wins, later calls reuse the same buffer (the
/// `set_global_default` error on re-entry is deliberately ignored — the
/// already-installed subscriber shares the same `OnceLock` Arc). Events are
/// read (and cleared) via [`take_events`].
pub fn install() {
    let log = CAPTURE.get_or_init(|| {
        Arc::new(EventLog {
            events: Mutex::new(Vec::new()),
        })
    });
    let _ = tracing::subscriber::set_global_default(CaptureSubscriber { log: log.clone() });
}

/// Returns the events captured since the last call and clears the buffer.
///
/// Each assertion-capable test calls this right before driving the behaviour
/// under test, so stale events from earlier tests never leak into a verdict.
pub fn take_events() -> Vec<(Level, String)> {
    install();
    let log = CAPTURE.get().expect("subscriber installed above");
    match log.events.lock() {
        Ok(mut guard) => std::mem::take(&mut *guard),
        Err(_) => Vec::new(),
    }
}
