//! WASM render tests for `pages/phrases`: `PhrasesHeader` (Router-based
//! PageHeader with the info tooltip).

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen_test::*;

use std::collections::HashSet;

use crate::pages::phrases::header::PhrasesHeader;
use crate::pages::phrases::phrase_card_item::PhraseCardItem;
use crate::test_support::{create_wrapper, mount_with_i18n, mount_with_router};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn phrases_header_renders_title_and_info_tooltip() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || view! { <PhrasesHeader /> }.into_any());
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"phrases-header\"]")
            .ok()
            .flatten()
            .is_some(),
        "header root must render"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"phrases-info-icon\"]")
            .unwrap()
            .is_some(),
        "the info icon must render"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// PhraseCardItem (phrase text/answer degrade without the CDN store)
// ═══════════════════════════════════════════════════════════════════════

fn phrase_study_card() -> origa::domain::StudyCard {
    use origa::domain::{Card, PhraseCard, StudyCard};
    StudyCard::new(Card::Phrase(PhraseCard::new(ulid::Ulid::new())))
}

#[wasm_bindgen_test]
async fn phrase_card_item_without_store_renders_skeleton_and_status() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let trigger = RwSignal::new(0u32);
        view! {
            <PhraseCardItem
                study_card=phrase_study_card()
                native_language=Signal::from(origa::domain::NativeLanguage::Russian)
                known_kanji=HashSet::new()
                on_toggle_favorite=Callback::new(|_| ())
                on_mark_as_known=Callback::new(|_| ())
                on_delete=Callback::new(|_| ())
                is_deleting=Signal::from(false)
                phrase_data_trigger=trigger
            />
        }
        .into_any()
    });
    tick().await;

    let item = wrapper
        .query_selector("[data-testid=\"phrases-card-item\"]")
        .unwrap()
        .unwrap();
    // Without the phrase store the text degrades to the skeleton placeholder
    let skeleton = item.query_selector(".anima-skeleton-paper");
    assert!(
        skeleton.is_ok_and(|sk| sk.is_some()),
        "missing phrase text must render the loading skeleton"
    );
    // The status tag and FSRS metrics render regardless
    assert!(
        item.query_selector(".tag").is_ok_and(|t| t.is_some()),
        "status tag must render"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// #914: dash-run artifacts must not reach the rendered card
// ═══════════════════════════════════════════════════════════════════════

/// The live prod instance behind #914 (card 53/65, chunk p0024): the
/// ──── run of U+2500 renders as a continuous line across the screen
/// when it leaks through the load boundary into the UI.
const DASH_RUN_CHUNK_JSON: &str = r#"[{"i":"00000000003YTM62ZWSHWQPHH5","x":"実は少し────","ru":"На самом деле, есть кое-что...","en":"Actually, there's a little something...","ko":"사실은 조금 있어요────","vi":"Thực ra là có chút..."}]"#;

/// Offline tokenizer entry for one exact string — the same hygiene trick
/// the lesson wasm tests use: TranslatorText settles through its
/// precompute branch, no CDN dictionary fetch in a bare test env.
fn install_phrase_precompute(text: &str) {
    use origa::domain::{
        PartOfSpeech, PrecomputedEntry, PrecomputedToken, install_precomputed_entry,
    };
    install_precomputed_entry(
        text,
        PrecomputedEntry {
            furigana_spans: vec![],
            tokens: vec![PrecomputedToken {
                surface: text.to_string(),
                base: text.to_string(),
                reading: text.to_string(),
                pos: PartOfSpeech::Noun,
            }],
        },
    );
}

/// Polls until the translator stops showing the dictionary loader and
/// returns the card item's text content.
async fn settled_item_text(wrapper: &web_sys::Element) -> String {
    let item = wrapper
        .query_selector("[data-testid=\"phrases-card-item\"]")
        .unwrap()
        .expect("phrase card item renders");
    for _ in 0..10 {
        crate::utils::yield_to_browser().await;
        tick().await;
        let loader = item.query_selector("[data-testid=\"translator-dict-loading\"]");
        if loader.is_ok_and(|l| l.is_none()) {
            return item.text_content().unwrap_or_default();
        }
    }
    item.text_content().unwrap_or_default()
}

#[wasm_bindgen_test]
async fn phrase_card_renders_prod_dash_run_artifact_clean() {
    use origa::dictionary::phrase::{
        cache_phrase_details, get_phrase_text, reset_phrase_data_for_test,
    };
    use origa::domain::{Card, PhraseCard, StudyCard};

    // The dirty prod chunk installs through the real parse boundary —
    // the sanitizer runs there, before any view can read the store.
    reset_phrase_data_for_test();
    cache_phrase_details(9140, DASH_RUN_CHUNK_JSON).expect("prod chunk installs");

    let prod_phrase = ulid::Ulid::from_string("00000000003YTM62ZWSHWQPHH5").expect("prod ULID");
    let question = get_phrase_text(&prod_phrase).expect("phrase detail in the store");
    assert_eq!(
        question, "実は少し",
        "the dash run must be stripped at the load boundary, before any view"
    );
    install_phrase_precompute(&question);

    let study_card = StudyCard::new(Card::Phrase(PhraseCard::new(prod_phrase)));
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, move || {
        let trigger = RwSignal::new(0u32);
        view! {
            <PhraseCardItem
                study_card=study_card.clone()
                native_language=Signal::from(origa::domain::NativeLanguage::Russian)
                known_kanji=HashSet::new()
                on_toggle_favorite=Callback::new(|_| ())
                on_mark_as_known=Callback::new(|_| ())
                on_delete=Callback::new(|_| ())
                is_deleting=Signal::from(false)
                phrase_data_trigger=trigger
            />
        }
        .into_any()
    });

    let text = settled_item_text(&wrapper).await;
    assert!(
        text.contains("実は少し"),
        "the clean phrase text must render, got: {text:?}"
    );
    for dash in ['\u{2500}', '\u{2015}', '\u{2014}'] {
        assert!(
            !text.contains(dash),
            "dash-run artifact leaked into the rendered card: {text:?}"
        );
    }

    // The translation side carries the same artifact class in prod (ko
    // runs are the most frequent) — the rendered meaning must be clean.
    let meaning = wrapper
        .query_selector("[data-testid=\"phrases-card-meaning\"]")
        .unwrap()
        .expect("meaning renders")
        .text_content()
        .unwrap_or_default();
    assert!(
        meaning.contains("На самом деле"),
        "the ru translation must render, got: {meaning:?}"
    );
    assert!(
        !meaning.contains(['\u{2500}', '\u{2015}', '\u{2014}']),
        "dash-run artifact leaked into the rendered meaning: {meaning:?}"
    );
}

#[wasm_bindgen_test]
async fn phrase_card_item_delete_opens_confirm_modal() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let trigger = RwSignal::new(0u32);
        view! {
            <PhraseCardItem
                study_card=phrase_study_card()
                native_language=Signal::from(origa::domain::NativeLanguage::Russian)
                known_kanji=HashSet::new()
                on_toggle_favorite=Callback::new(|_| ())
                on_mark_as_known=Callback::new(|_| ())
                on_delete=Callback::new(|_| ())
                is_deleting=Signal::from(false)
                phrase_data_trigger=trigger
            />
        }
        .into_any()
    });
    tick().await;

    use wasm_bindgen::JsCast;
    wrapper
        .query_selector("[data-testid=\"phrases-card-item-delete-btn\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"phrases-delete-modal\"]")
            .unwrap()
            .is_some(),
        "delete click must open the confirmation modal"
    );
}
