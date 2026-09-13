//! Network fetch primitive with an idle deadline (ADR-052).
//!
//! Every CDN/TrailBase request goes through here so a dead network can
//! never hang the startup pipeline: a request is aborted when no data has
//! arrived for `idle_ms`. Progress (a received chunk) resets the timer,
//! so slow-but-alive transfers survive while silent stalls fail fast.
//!
//! Timeout errors carry a textual marker and are recognizable via
//! [`is_idle_timeout`] — the retry gate refuses to re-run a request whose
//! neighbour already stalled (ADR-052, R3-C1).

use origa::domain::OrigaError;

/// No-data budget per network request: generous enough for a slow DNS
/// lookup on degraded Wi-Fi to still complete, small enough that a fully
/// dead network surfaces as an error in seconds, not minutes.
pub const DEFAULT_IDLE_TIMEOUT_MS: u32 = 10_000;

/// Errors produced by the idle watchdog start with this marker so callers
/// can distinguish "network stalled" from HTTP/parse failures without
/// extending the shared domain error enum.
const IDLE_TIMEOUT_MARKER: &str = "idle timeout";

/// Body of a network response fetched under the idle deadline.
#[derive(Debug)]
pub struct IdleResponse {
    /// Response reconstructed from the received bytes, suitable for
    /// Cache API storage (the original body was consumed by the
    /// progress-tracking reader).
    pub response: web_sys::Response,
    pub bytes: Vec<u8>,
}

/// Whether the error was produced by the idle watchdog (as opposed to an
/// HTTP failure or an immediate connection refusal).
pub fn is_idle_timeout(error: &OrigaError) -> bool {
    matches!(
        error,
        OrigaError::NetworkError { reason, .. } if reason.starts_with(IDLE_TIMEOUT_MARKER)
    )
}

fn network_error(url: &str, reason: impl Into<String>) -> OrigaError {
    OrigaError::NetworkError {
        url: url.to_string(),
        reason: reason.into(),
    }
}

/// Reads a response body stream to the end under the idle deadline.
/// Testable seam for the WASM tests: streams are cheap to fake, fetches
/// are not.
pub(crate) async fn read_all_with_idle(
    stream: web_sys::ReadableStream,
    url: &str,
    idle_ms: u32,
) -> Result<Vec<u8>, OrigaError> {
    let _ = (stream, url, idle_ms);
    Err(network_error(url, "not implemented"))
}

/// Fetches `url`, aborting when no data arrives for `idle_ms`.
pub async fn fetch_idle(url: &str, idle_ms: u32) -> Result<IdleResponse, OrigaError> {
    let _ = idle_ms;
    Err(network_error(url, "not implemented"))
}

#[cfg(all(target_arch = "wasm32", test))]
#[path = "net_timeout_wasm_tests.rs"]
mod net_timeout_wasm_tests;
