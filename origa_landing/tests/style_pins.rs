//! Style pins for the landing card borders (task origa#404).
//!
//! The landing instance of the "279" card-in-a-card family: card classes
//! carried an offset `::after` pseudo-element with its own border, which
//! reads as a second frame. The canon is one border level per card —
//! cf. PR #937 for the product-side pins of the same family. This file pins
//! the fixed state: the offset frame pseudo-elements stay gone and each
//! card keeps exactly its own 1px border.

#![cfg(feature = "ssr")]

const CSS: &str = include_str!("../style/landing.css");

/// Card classes that must not carry the offset-frame `::after` again.
const NO_OFFSET_FRAME_SELECTORS: [&str; 4] = [
    "download-secondary__card",
    "home-feature-card",
    "feat-step",
    "intg-card",
];

/// Base selectors whose own border is the only frame level.
const ONE_BORDER_SELECTORS: [&str; 4] = [
    ".download-secondary__card {",
    ".home-feature-card {",
    ".feat-step {",
    ".intg-card {",
];

#[test]
fn card_classes_keep_no_offset_frame_pseudo_elements() {
    for selector in NO_OFFSET_FRAME_SELECTORS {
        let pin = format!("{selector}::after");
        assert!(
            !CSS.contains(&pin),
            "{selector}::after is back — the offset frame reads as a \
             second border on the card (canon: one border level per card)"
        );
    }
}

#[test]
fn card_classes_keep_their_own_single_border() {
    for selector in ONE_BORDER_SELECTORS {
        let block = rule_block(selector)
            .unwrap_or_else(|| panic!("{selector} rule is missing from style/landing.css"));
        assert!(
            block.contains("border: 1px solid var(--border-dark);"),
            "{selector} must keep exactly its own 1px border"
        );
    }
}

#[test]
fn listening_band_is_a_frameless_wrapper() {
    let block = rule_block(".feat-listening__band {")
        .expect(".feat-listening__band rule is missing from style/landing.css");
    assert!(
        !block.contains("border:"),
        ".feat-listening__band must not frame its framed capability tiles \
         (card-in-a-card); it stays a padding wrapper"
    );
}

/// Returns the body of the first top-level rule for an exact selector
/// (the stylesheet has no nested braces outside `@media`, and none of the
/// pinned selectors live inside a media query).
fn rule_block(selector: &str) -> Option<&str> {
    let start = CSS.find(selector)?;
    let open = CSS[start..].find('{')? + start;
    let close = CSS[open..].find('}')? + open;
    Some(&CSS[open + 1..close])
}
