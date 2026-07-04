//! Self-hosted `@font-face` declarations for the app's typography system.
//!
//! The font files live on the project CDN (`<cdn>/fonts/<logical>-<sha8>.woff2`)
//! and are enumerated at build time by `build.rs` into the `FONT_FILES` table.
//! The content-hash suffix is the cache-bust key (see ADR-028): a regenerated
//! subset changes the hash, so immutable CDN edge caches can never serve a
//! stale font.
//!
//! URLs are built through `core::config::cdn_url`, keeping `ORIGA_CDN_BASE_URL`
//! the single source of truth and letting E2E run against the local CDN.
//!
//! The stylesheet is injected imperatively into `document.head` (from the
//! `App` component body) rather than rendered through the `view!` macro.
//! The `origa_ui_bin` crate mounts the whole `<App/>` view tree and encodes
//! it as deeply nested generic tuples; adding any element to the root view
//! tips the bin past its monomorphization depth limit (`error: queries
//! overflow the depth limit!`). A `<style>` node added via the DOM API adds
//! nothing to the view tuple, so the bin keeps compiling.

use crate::core::config::cdn_url;

include!(concat!(env!("OUT_DIR"), "/fonts.rs"));

const STYLE_ID: &str = "origa-font-faces";

struct FaceSpec {
    logical: &'static str,
    family: &'static str,
    weight: &'static str,
    style: &'static str,
    unicode_range: &'static str,
}

const LATIN_RANGE: &str = "U+0000-007F,U+00A0-00FF,U+2000-206F,U+20A0-20CF,U+2100-214F";
const CJK_RANGE: &str = "U+3000-303F,U+3040-30FF,U+3400-4DBF,U+4E00-9FFF,U+F900-FAFF,U+FF00-FFEF";

const FACES: &[FaceSpec] = &[
    FaceSpec {
        logical: "cormorant-garamond",
        family: "Cormorant Garamond",
        weight: "300 700",
        style: "normal",
        unicode_range: LATIN_RANGE,
    },
    FaceSpec {
        logical: "cormorant-garamond-italic",
        family: "Cormorant Garamond",
        weight: "300 700",
        style: "italic",
        unicode_range: LATIN_RANGE,
    },
    FaceSpec {
        logical: "dm-mono-300",
        family: "DM Mono",
        weight: "300",
        style: "normal",
        unicode_range: LATIN_RANGE,
    },
    FaceSpec {
        logical: "dm-mono-400",
        family: "DM Mono",
        weight: "400",
        style: "normal",
        unicode_range: LATIN_RANGE,
    },
    FaceSpec {
        logical: "dm-mono-500",
        family: "DM Mono",
        weight: "500",
        style: "normal",
        unicode_range: LATIN_RANGE,
    },
    FaceSpec {
        logical: "dm-mono-400-italic",
        family: "DM Mono",
        weight: "400",
        style: "italic",
        unicode_range: LATIN_RANGE,
    },
    FaceSpec {
        logical: "noto-sans-jp-400",
        family: "Noto Sans JP",
        weight: "400",
        style: "normal",
        unicode_range: CJK_RANGE,
    },
    FaceSpec {
        logical: "noto-serif-jp-400",
        family: "Noto Serif JP",
        weight: "400",
        style: "normal",
        unicode_range: CJK_RANGE,
    },
];

/// Inject the `@font-face` stylesheet into `document.head`.
///
/// Idempotent: skips when a `<style id="origa-font-faces">` already exists, so
/// repeated calls (or test re-mounts) never duplicate it. A no-op when no subset
/// files are registered, leaving the app on system fallback fonts.
pub fn inject_font_faces() {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    if document.get_element_by_id(STYLE_ID).is_some() {
        return;
    }
    let css = font_face_css();
    if css.is_empty() {
        return;
    }
    let Ok(style) = document.create_element("style") else {
        return;
    };
    style.set_id(STYLE_ID);
    style.set_text_content(Some(&css));
    if let Some(head) = document.head() {
        let _ = head.append_child(&style);
    }
}

/// Build the `@font-face` stylesheet for every font present in `FONT_FILES`.
///
/// Returns an empty string when no subset files are registered (e.g. a build
/// without `cdn/fonts/` populated), so the app falls back to system fonts
/// instead of emitting dangling `@font-face` rules.
fn font_face_css() -> String {
    if FONT_FILES.is_empty() {
        return String::new();
    }

    let mapped: Vec<&str> = FACES
        .iter()
        .filter_map(|spec| filename_for(spec.logical).map(|_| spec.logical))
        .collect();
    let unmapped: Vec<&str> = FONT_FILES
        .iter()
        .filter_map(|(logical, _)| {
            (!FACES.iter().any(|spec| spec.logical == *logical)).then_some(*logical)
        })
        .collect();
    if !unmapped.is_empty() {
        tracing::warn!(
            "cdn/fonts has files with no @font-face spec: {unmapped:?}; \
             they will be downloaded but never served"
        );
    }
    if mapped.len() != FACES.len() {
        tracing::warn!(
            "{} @font-face spec(s) have no matching woff2 in cdn/fonts",
            FACES.len() - mapped.len()
        );
    }

    let mut css = String::new();
    for spec in FACES {
        let Some(filename) = filename_for(spec.logical) else {
            continue;
        };
        let url = cdn_url(&format!("/fonts/{filename}"));
        css.push_str("@font-face{font-family:\"");
        css.push_str(spec.family);
        css.push_str("\";font-weight:");
        css.push_str(spec.weight);
        css.push_str(";font-style:");
        css.push_str(spec.style);
        css.push_str(";font-display:swap;unicode-range:");
        css.push_str(spec.unicode_range);
        css.push_str(";src:url(\"");
        css.push_str(&url);
        css.push_str("\") format(\"woff2\");}\n");
    }
    css
}

fn filename_for(logical: &str) -> Option<&'static str> {
    FONT_FILES
        .iter()
        .find(|(name, _)| *name == logical)
        .map(|(_, filename)| *filename)
}
