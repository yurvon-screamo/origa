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

use futures::future::{Fuse, FutureExt};
use gloo_timers::future::TimeoutFuture;
use leptos::wasm_bindgen::JsCast;
use origa::domain::OrigaError;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

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

fn idle_timeout_error(url: &str, idle_ms: u32) -> OrigaError {
    network_error(
        url,
        format!("{IDLE_TIMEOUT_MARKER} after {idle_ms} ms without data"),
    )
}

/// Races `work` against the idle deadline. On timeout the optional
/// controller aborts the underlying request (a dropped future alone
/// leaves the HTTP connection running) and a marker error is returned.
async fn with_idle_deadline<F>(
    work: F,
    url: &str,
    idle_ms: u32,
    abort: Option<&web_sys::AbortController>,
) -> Result<F::Output, OrigaError>
where
    F: Future + Unpin,
{
    let mut work: Fuse<F> = work.fuse();
    let mut timer: Fuse<TimeoutFuture> = TimeoutFuture::new(idle_ms).fuse();
    futures::select! {
        output = work => Ok(output),
        _ = timer => {
            if let Some(controller) = abort {
                controller.abort();
            }
            Err(idle_timeout_error(url, idle_ms))
        }
    }
}

/// Reads a response body stream to the end under the idle deadline:
/// every received chunk restarts the watchdog, so only a silent stall
/// (no bytes for `idle_ms`) aborts. Testable seam for the WASM tests —
/// streams are cheap to fake, fetches are not.
pub(crate) async fn read_all_with_idle(
    stream: web_sys::ReadableStream,
    url: &str,
    idle_ms: u32,
) -> Result<Vec<u8>, OrigaError> {
    let generic_reader = stream.get_reader();
    let reader: web_sys::ReadableStreamDefaultReader = generic_reader
        .dyn_into()
        .map_err(|e| network_error(url, format!("stream reader is not a default reader: {e:?}")))?;

    let mut bytes = Vec::new();
    loop {
        let read_promise = reader.read();
        let result = with_idle_deadline(JsFuture::from(read_promise), url, idle_ms, None)
            .await?
            .map_err(|e| network_error(url, format!("stream read rejected: {e:?}")))?;

        let done = js_sys::Reflect::get(&result, &JsValue::from_str("done"))
            .map_err(|e| network_error(url, format!("malformed chunk envelope: {e:?}")))?
            .as_bool()
            .unwrap_or(false);
        if done {
            break;
        }

        let value = js_sys::Reflect::get(&result, &JsValue::from_str("value"))
            .map_err(|e| network_error(url, format!("malformed chunk envelope: {e:?}")))?;
        let chunk = js_sys::Uint8Array::new(&value);
        bytes.extend_from_slice(&chunk.to_vec());
    }
    Ok(bytes)
}

/// Fetches `url` (GET), aborting when no data arrives for `idle_ms`.
/// The deadline covers both the headers phase and the body transfer;
/// body chunks reset it. The returned response is rebuilt from the
/// received bytes and is safe to store in the Cache API.
pub async fn fetch_idle(url: &str, idle_ms: u32) -> Result<IdleResponse, OrigaError> {
    let window = web_sys::window().ok_or_else(|| network_error(url, "No window found"))?;
    let controller = web_sys::AbortController::new()
        .map_err(|e| network_error(url, format!("AbortController unavailable: {e:?}")))?;

    let init = web_sys::RequestInit::new();
    init.set_signal(Some(&controller.signal()));

    let request = web_sys::Request::new_with_str_and_init(url, &init)
        .map_err(|e| network_error(url, format!("failed to build request: {e:?}")))?;

    let fetch_promise = window.fetch_with_request_and_init(&request, &init);
    let response_value = with_idle_deadline(
        JsFuture::from(fetch_promise),
        url,
        idle_ms,
        Some(&controller),
    )
    .await?
    .map_err(|e| network_error(url, format!("Failed to fetch: {e:?}")))?;
    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|e| network_error(url, format!("Failed to cast response: {e:?}")))?;

    if !response.ok() {
        return Err(network_error(url, format!("HTTP {}", response.status())));
    }

    let bytes = match response.body() {
        Some(stream) => read_all_with_idle(stream, url, idle_ms).await?,
        None => {
            // No streaming body available (legacy/opaque responses):
            // fall back to arrayBuffer under the same deadline.
            let buffer_promise = response
                .array_buffer()
                .map_err(|e| network_error(url, format!("no body reader: {e:?}")))?;
            let buffer = with_idle_deadline(
                JsFuture::from(buffer_promise),
                url,
                idle_ms,
                Some(&controller),
            )
            .await?
            .map_err(|e| network_error(url, format!("Failed to read body: {e:?}")))?;
            js_sys::Uint8Array::new(&buffer).to_vec()
        },
    };

    let response = rebuild_response(&response, &bytes, url)?;
    Ok(IdleResponse { response, bytes })
}

/// Rebuilds a storable Response from consumed bytes, preserving the
/// content type for later `blob:` consumers (audio decoding depends on
/// the correct MIME, cdn_provider.rs).
fn rebuild_response(
    original: &web_sys::Response,
    bytes: &[u8],
    url: &str,
) -> Result<web_sys::Response, OrigaError> {
    let options = web_sys::BlobPropertyBag::new();
    if let Ok(Some(content_type)) = original.headers().get("content-type") {
        options.set_type(&content_type);
    }

    let array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
    array.copy_from(bytes);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
        &js_sys::Array::of1(&array),
        &options,
    )
    .map_err(|e| network_error(url, format!("failed to rebuild blob: {e:?}")))?;

    let init = web_sys::ResponseInit::new();
    init.set_status(200);
    init.set_status_text("OK");
    web_sys::Response::new_with_opt_blob_and_init(Some(&blob), &init)
        .map_err(|e| network_error(url, format!("failed to rebuild response: {e:?}")))
}

/// GET text under the default idle deadline; returns the storable
/// response together with the decoded body.
pub async fn fetch_text_idle(url: &str) -> Result<(web_sys::Response, String), OrigaError> {
    let idle = fetch_idle(url, DEFAULT_IDLE_TIMEOUT_MS).await?;
    let text = String::from_utf8(idle.bytes)
        .map_err(|_| network_error(url, "Response is not valid UTF-8"))?;
    Ok((idle.response, text))
}

/// GET bytes under the default idle deadline.
pub async fn fetch_bytes_idle(url: &str) -> Result<(web_sys::Response, Vec<u8>), OrigaError> {
    let idle = fetch_idle(url, DEFAULT_IDLE_TIMEOUT_MS).await?;
    Ok((idle.response, idle.bytes))
}

/// Sends a request built by the caller (method/headers/body in `init`),
/// aborting when no data arrives for `idle_ms`. Unlike [`fetch_idle`],
/// HTTP error statuses are NOT translated into an error — API callers
/// inspect statuses themselves (401 refresh, 424 Apple flow).
pub async fn send_request_idle(
    url: &str,
    init: &web_sys::RequestInit,
    idle_ms: u32,
) -> Result<IdleResponse, OrigaError> {
    let _ = (url, init, idle_ms);
    let empty = web_sys::Response::new()
        .map_err(|e| network_error(url, format!("stub response: {e:?}")))?;
    Ok(IdleResponse {
        response: empty,
        bytes: Vec::new(),
    })
}

#[cfg(all(target_arch = "wasm32", test))]
#[path = "net_timeout_wasm_tests.rs"]
mod net_timeout_wasm_tests;
