//! WASM render tests for `ExampleRubyText`: the example-sentence render
//! whose CONTEXT ruby pairs (chunk, Tanaka readings) must beat the
//! dictionary — 頭 in 「頭を冷やせ。」 shows あたま, never the lemma's
//! first dictionary entry とう (the wrong-reading bug class #929 fixes).
//!
//! The assertions pin the CONTEXT-pair behavior only: the gap fallback
//! depends on the global dictionary state, which the wasm suite does not
//! control — a dictionary-less run degrades gaps to plain runs by design.

#![cfg(all(target_arch = "wasm32", test))]

use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::tick;
use origa::dictionary::example::RubyPair;
use wasm_bindgen_test::*;

use crate::test_support::{create_wrapper, mount_to_wrapper};
use crate::ui_components::ExampleRubyText;

wasm_bindgen_test_configure!(run_in_browser);

fn pair(surface: &str, reading: &str) -> RubyPair {
    RubyPair {
        surface: surface.to_string(),
        reading: reading.to_string(),
    }
}

/// The screenshot case: the context pair must surface as the ruby text of
/// the word segment — the dictionary reading (とう) must not appear.
#[wasm_bindgen_test]
async fn context_ruby_overrides_dictionary_in_example_render() {
    let wrapper = create_wrapper();
    let known: HashSet<char> = HashSet::new();
    let ruby = vec![pair("頭", "あたま")];
    mount_to_wrapper(&wrapper, move || {
        view! {
            <ExampleRubyText
                text="頭を冷やせ。".to_string()
                ruby=ruby.clone()
                known_kanji=known.clone()
                class="ext-test-1".to_string()
            />
        }
        .into_any()
    });
    tick().await;

    let html = wrapper
        .query_selector("span.ext-test-1")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        html.contains("<rt class=\"furigana-rt\">あたま</rt>"),
        "context reading あたま must render as ruby; got: {html}"
    );
    assert!(
        !html.contains("とう"),
        "dictionary first-entry reading とう must not leak into the render; got: {html}"
    );
}

/// The word segment inside the stored highlight range carries the
/// highlight class; the rest of the sentence stays plain.
#[wasm_bindgen_test]
async fn word_segment_inside_highlight_range_is_marked() {
    let wrapper = create_wrapper();
    let known: HashSet<char> = HashSet::new();
    let ruby = vec![pair("頭", "あたま")];
    mount_to_wrapper(&wrapper, move || {
        view! {
            <ExampleRubyText
                text="頭を冷やせ。".to_string()
                ruby=ruby.clone()
                known_kanji=known.clone()
                highlight=(0, 1)
                class="ext-test-2".to_string()
            />
        }
        .into_any()
    });
    tick().await;

    let highlighted = wrapper
        .query_selector("span.ext-test-2 span.word-example-highlight")
        .unwrap();
    assert!(highlighted.is_some(), "the word segment must be marked");
    let hl_html = highlighted.unwrap().inner_html();
    assert!(
        hl_html.contains(">頭<"),
        "the highlight must wrap exactly the word; got: {hl_html}"
    );
    assert!(
        !hl_html.contains("冷"),
        "the highlight must not spill into neighbouring segments; got: {hl_html}"
    );
}

/// Recall-card contract: highlight=(-1,-1) (the card belongs to the
/// sentence, not a word) — no segment is ever marked.
#[wasm_bindgen_test]
async fn absent_highlight_marks_nothing_in_example_render() {
    let wrapper = create_wrapper();
    let known: HashSet<char> = HashSet::new();
    let ruby = vec![pair("頭", "あたま")];
    mount_to_wrapper(&wrapper, move || {
        view! {
            <ExampleRubyText
                text="頭を冷やせ。".to_string()
                ruby=ruby.clone()
                known_kanji=known.clone()
                highlight=(-1, -1)
                class="ext-test-3".to_string()
            />
        }
        .into_any()
    });
    tick().await;

    let html = wrapper
        .query_selector("span.ext-test-3")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        !html.contains("word-example-highlight"),
        "absent highlight must mark no segment; got: {html}"
    );
    assert!(
        html.contains("<rt class=\"furigana-rt\">あたま</rt>"),
        "the context reading still renders; got: {html}"
    );
}

/// The known-kanji filter applies to context segments too: the ruby stays
/// in the DOM but is marked hidden (same contract as FuriganaText).
#[wasm_bindgen_test]
async fn known_kanji_hides_context_furigana() {
    let wrapper = create_wrapper();
    let known: HashSet<char> = "頭".chars().collect();
    let ruby = vec![pair("頭", "あたま")];
    mount_to_wrapper(&wrapper, move || {
        view! {
            <ExampleRubyText
                text="頭を冷やせ。".to_string()
                ruby=ruby.clone()
                known_kanji=known.clone()
                class="ext-test-4".to_string()
            />
        }
        .into_any()
    });
    tick().await;

    let html = wrapper
        .query_selector("span.ext-test-4")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        html.contains("furigana-hidden"),
        "known kanji must mark the ruby hidden; got: {html}"
    );
}
