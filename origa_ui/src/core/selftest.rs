//! CI selftest feature probes (task 20261104060604, GH #854).
//!
//! The Android smoke job exercises a real feature path end-to-end: it fires
//! a deep link (`origa://selftest-ocr`), this module runs the probe and
//! reports the outcome through the native `selftest_marker` command. The
//! bridge exists because the smoke script needs a *stable, grep-able*
//! marker — its own `I/origa` tag, the ADR-063 marker precedent. WebView
//! console output does reach logcat (wry pipes it as `Tauri/Console`), but
//! that channel carries arbitrary app logs under a foreign tag, while
//! `tracing` itself has no logcat backend at all.
//!
//! Mounted on Android only: other platforms never see the deep link, so the
//! listener simply does not exist there (zero overhead, nothing to ignore).

use tracing::debug;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use crate::core::device_ai;
use crate::core::platform::platform_name;
use crate::core::tauri::{event_listen_fn, invoke_with_args, is_tauri};

/// Deep link fired by the Android smoke script to trigger the OCR probe.
const SELFTEST_OCR_URL: &str = "origa://selftest-ocr";

/// The native command that carries markers to logcat (Android) / tracing.
const CMD_SELFTEST_MARKER: &str = "selftest_marker";

/// Fixed probe image: a 320x120 PNG rendering `日本語` in Noto Sans CJK
/// (black on white, generated with ImageMagick). Kept tiny on purpose —
/// it lives in the bundle.
const PROBE_IMAGE_B64: &str = "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";

/// Success requires any one of the rendered glyphs: recognizers segment
/// differently (`日本語` may come back as `日本 語` or with extras), but a
/// result without any expected glyph is a failed probe, not a pass.
const PROBE_EXPECTED_CHARS: &[char] = &['日', '本', '語'];

// One probe at a time — a repeated deep link must not stack concurrent
// recognitions (the smoke script fires the link once, but a stray second
// fire during a slow first run would double-report markers). A doc comment
// here would be an `unused doc comment` — thread_local! does not propagate
// outer docs (the share-intake precedent).
thread_local! {
    static PROBE_IN_FLIGHT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Parses a deep-link URL into the selftest probe it requests.
/// `None` for everything else — coexists with the shortcut/oauth listeners
/// on the same `deep-link://new-url` channel without touching their parsing.
fn parse_selftest_url(url: &str) -> Option<SelftestKind> {
    match url.trim_end_matches(['/', '?', '&']) {
        SELFTEST_OCR_URL => Some(SelftestKind::Ocr),
        _ => None,
    }
}

/// The probes the smoke script can request.
#[derive(Debug, PartialEq, Eq)]
enum SelftestKind {
    /// `vision_recognize_text` on the fixed probe image.
    Ocr,
}

/// Starts the selftest deep-link listener (cold-start poll + warm events,
/// the `shortcut_links` pattern). Android-only: returns early elsewhere.
pub fn start_selftest_listener() {
    if !is_tauri() || platform_name() != "android" {
        return;
    }

    // Cold start: the deep-link plugin may hold the URL before this
    // listener mounts (the smoke script fires the link right after the
    // liveness checks, which can precede the WebView finishing setup).
    leptos::task::spawn_local(async move {
        match invoke_with_args("get_current_deep_link", &JsValue::UNDEFINED).await {
            Ok(url) => {
                if let Some(url) = url.as_string() {
                    route_selftest(&url);
                }
            },
            Err(e) => debug!("selftest: cold-poll failed: {e}"),
        }
    });

    let Some(listen) = event_listen_fn() else {
        return;
    };
    let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: JsValue| {
        let url = js_sys::Reflect::get(&event, &JsValue::from_str("url"))
            .ok()
            .and_then(|v| v.as_string());
        if let Some(url) = url {
            route_selftest(&url);
        }
    }) as Box<dyn Fn(JsValue)>);
    if let Some(callback_ptr) = callback.as_ref().dyn_ref::<js_sys::Function>() {
        let _ = listen.call3(
            &JsValue::UNDEFINED,
            &JsValue::from_str("deep-link://new-url"),
            callback_ptr,
            &JsValue::UNDEFINED,
        );
    }
    callback.forget();
}

fn route_selftest(url: &str) {
    let Some(kind) = parse_selftest_url(url) else {
        return;
    };
    if PROBE_IN_FLIGHT.with(|slot| slot.replace(true)) {
        debug!("selftest: probe already in flight, ignoring repeated deep link");
        return;
    }
    leptos::task::spawn_local(async move {
        match kind {
            SelftestKind::Ocr => run_ocr_probe().await,
        }
        PROBE_IN_FLIGHT.with(|slot| slot.set(false));
    });
}

/// OCR probe: recognize the fixed image through the device-ai plugin and
/// report the outcome via the logcat bridge. Every path reports — the smoke
/// script fails on a missing marker, so silence is never an outcome.
async fn run_ocr_probe() {
    match device_ai::recognize_text(PROBE_IMAGE_B64).await {
        Ok(result) => {
            let hit = PROBE_EXPECTED_CHARS
                .iter()
                .any(|glyph| result.text.contains(*glyph));
            if hit {
                report(&format!("[selftest] ocr ok: {}", result.text));
            } else {
                report(&format!(
                    "[selftest] ocr failed: no expected glyph in {:?}",
                    result.text
                ));
            }
        },
        Err(e) => report(&format!("[selftest] ocr failed: {e}")),
    }
}

/// Fires a marker through the native logcat bridge. Best-effort: a failed
/// invoke is logged, never fatal (the smoke script times out on silence).
///
/// The Tauri v2 IPC contract expects a JSON object of named argument keys —
/// a bare JS string deserialises into no arguments at all (`Value::String`
/// has no `message` key), the command rejects with
/// "missing required key message", and the marker never reaches logcat.
/// Same convention as every other call site (`read_share_file`,
/// `pick_and_read_file`): named keys on a `js_sys::Object`.
fn report(marker: &str) {
    debug!("selftest: reporting marker: {marker}");
    let args = js_sys::Object::new();
    if js_sys::Reflect::set(
        &args,
        &JsValue::from_str("message"),
        &JsValue::from_str(marker),
    )
    .is_err()
    {
        debug!("selftest: failed to build marker args, marker dropped: {marker}");
        return;
    }
    leptos::task::spawn_local(async move {
        if let Err(e) = invoke_with_args(CMD_SELFTEST_MARKER, &args.into()).await {
            debug!("selftest: marker invoke failed: {e}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// The probe asset is embedded in the source file, and a silently
    /// truncated base64 decodes into a PNG whose data chunk fails the CRC —
    /// BitmapFactory on Android then rejects the image with INVALID_IMAGE
    /// and the smoke job fails on the `ocr failed` marker. Pin the full
    /// integrity the same way the wire contracts are pinned: decode, walk
    /// the chunk list, verify every CRC, require a zero-length IEND end.
    #[test]
    fn probe_image_is_a_complete_png() {
        use base64::Engine;

        fn crc32(data: &[u8]) -> u32 {
            !data.iter().fold(0xFFFF_FFFFu32, |acc, &byte| {
                (0..8).fold(acc ^ u32::from(byte), |acc, _| {
                    if acc & 1 != 0 {
                        (acc >> 1) ^ 0xEDB8_8320
                    } else {
                        acc >> 1
                    }
                })
            })
        }

        let raw = base64::engine::general_purpose::STANDARD
            .decode(PROBE_IMAGE_B64)
            .expect("probe image base64 must decode");
        assert!(
            raw.starts_with(b"\x89PNG\r\n\x1a\n"),
            "probe image must be a PNG"
        );

        let mut pos = 8;
        loop {
            assert!(
                pos + 12 <= raw.len(),
                "truncated PNG: chunk header at {pos} runs past the end"
            );
            let len = u32::from_be_bytes(raw[pos..pos + 4].try_into().unwrap()) as usize;
            let chunk_type = &raw[pos + 4..pos + 8];
            let crc_stored =
                u32::from_be_bytes(raw[pos + 8 + len..pos + 12 + len].try_into().unwrap());
            let crc_calc = crc32(&raw[pos + 4..pos + 8 + len]);
            assert_eq!(crc_stored, crc_calc, "chunk {chunk_type:?} CRC mismatch");
            pos += 12 + len;
            if chunk_type == b"IEND" {
                assert_eq!(len, 0, "IEND must be zero-length");
                assert_eq!(pos, raw.len(), "IEND must be the last chunk");
                break;
            }
        }
    }

    /// Deep-link routing table: the smoke script's URL resolves, every
    /// neighbour (shortcut/oauth/prefix/suffix variants) stays untouched.
    #[rstest]
    #[case::exact_ocr(SELFTEST_OCR_URL, Some(SelftestKind::Ocr))]
    #[case::trailing_slash("origa://selftest-ocr/", Some(SelftestKind::Ocr))]
    #[case::shortcut_add("origa://add?source=camera", None)]
    #[case::oauth("origa://auth/callback?code=x", None)]
    #[case::prefix_variant("origa://selftest-ocrx", None)]
    #[case::other_selftest("origa://selftest-share", None)]
    #[case::unknown("origa://unknown", None)]
    #[case::empty("", None)]
    fn selftest_url_routing(#[case] url: &str, #[case] expected: Option<SelftestKind>) {
        assert_eq!(parse_selftest_url(url), expected);
    }
}
