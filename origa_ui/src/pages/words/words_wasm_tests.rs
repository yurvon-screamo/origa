//! WASM render tests for `pages/words`: `WordsHeader` (Router-based,
//! modal wiring needs repository context), `VocabularyCardItem`,
//! `AnalyzedWordItem`.
//!
//! NOT TESTED HERE: `asr_provider` (Tauri-only async device-ai invoke
//! paths, no DOM), OCR/audio/anki input stages (media + ONNX + IndexedDB).

#![cfg(all(target_arch = "wasm32", test))]

use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::tick;
use origa::use_cases::AnalyzedWord;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::pages::words::WordsHeader;
use crate::pages::words::analyzed_word_item::AnalyzedWordItem;
use crate::pages::words::vocabulary_card_item::VocabularyCardItem;
use crate::test_support::{
    create_wrapper, mount_with_i18n, mount_with_router_and_stores, shared_cell,
};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// WordsHeader (Router + repository context for the embedded modal)
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn words_header_renders_title_and_actions() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router_and_stores(&wrapper, None, || {
        let refresh = RwSignal::new(0u32);
        view! { <WordsHeader refresh_trigger=refresh /> }.into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"words-add-btn\"]")
            .ok()
            .flatten()
            .is_some(),
        "words header must render the add button"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"words-sets-btn\"]")
            .ok()
            .flatten()
            .is_some(),
        "words header must render the sets button"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"words-title\"]")
            .ok()
            .flatten()
            .is_some_and(|t| !t.text_content().unwrap().is_empty()),
        "words header must render its title"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// VocabularyCardItem
// ═══════════════════════════════════════════════════════════════════════

/// A reversed vocabulary study card: the Japanese lives in `reverse_side`,
/// so `answer()` works without the translation dictionary.
fn reversed_study_card(japanese: &str) -> origa::domain::StudyCard {
    use origa::domain::{Card, Question, StudyCard, VocabularyCard};
    let vocab = VocabularyCard::new_with_pos(
        Question::new("translation".to_string()).unwrap(),
        None,
        Some(Question::new(japanese.to_string()).unwrap()),
    );
    StudyCard::new(Card::Vocabulary(vocab))
}

#[wasm_bindgen_test]
async fn vocabulary_card_item_renders_word_and_answer() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let card = reversed_study_card("ねこ");
        view! {
            <VocabularyCardItem
                study_card=card
                native_language=Signal::from(origa::domain::NativeLanguage::Russian)
                known_kanji=HashSet::new()
                on_open_detail_cb=Callback::new(|_| ())
                on_toggle_favorite=Callback::new(|_| ())
                on_mark_as_known=Callback::new(|_| ())
                on_delete=Callback::new(|_| ())
                is_deleting=Signal::from(false)
            />
        }
        .into_any()
    });
    tick().await;

    let item = wrapper
        .query_selector("[data-testid=\"words-card-item\"]")
        .unwrap()
        .unwrap();
    let text = item.text_content().unwrap();
    assert!(text.contains("ねこ"), "the word must render; got: {text}");
    // reverse_side answer renders as a translation item
    assert!(
        item.query_selector("[data-testid=\"words-card-translations\"]")
            .unwrap()
            .is_some(),
        "translations block must render"
    );
    assert!(
        item.query_selector("[data-testid=\"words-card-fsrs\"]")
            .unwrap()
            .is_some(),
        "FSRS metrics must render"
    );
}

#[wasm_bindgen_test]
async fn vocabulary_card_item_delete_opens_confirm_modal() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let card = reversed_study_card("ねこ");
        view! {
            <VocabularyCardItem
                study_card=card
                native_language=Signal::from(origa::domain::NativeLanguage::Russian)
                known_kanji=HashSet::new()
                on_open_detail_cb=Callback::new(|_| ())
                on_toggle_favorite=Callback::new(|_| ())
                on_mark_as_known=Callback::new(|_| ())
                on_delete=Callback::new(|_| ())
                is_deleting=Signal::from(false)
            />
        }
        .into_any()
    });
    tick().await;

    // Act: click the delete action in the card's action bar
    let delete_btn = wrapper
        .query_selector("[data-testid=\"words-card-item-delete-btn\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>();
    delete_btn.click();
    tick().await;

    // Assert: the delete confirmation modal opened
    assert!(
        wrapper
            .query_selector("[data-testid=\"words-delete-modal\"]")
            .unwrap()
            .is_some(),
        "delete click must open the confirmation modal"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// AnalyzedWordItem
// ═══════════════════════════════════════════════════════════════════════

fn analyzed_word(known: bool, meaning: Option<String>) -> AnalyzedWord {
    AnalyzedWord {
        base_form: "食べる".to_string(),
        reading: "たべる".to_string(),
        part_of_speech: origa::domain::PartOfSpeech::Verb,
        is_known: known,
        meaning,
        is_counter: false,
    }
}

#[wasm_bindgen_test]
async fn analyzed_word_item_new_word_with_meaning_is_selectable() {
    let wrapper = create_wrapper();
    let (set_toggled, get_toggled) = shared_cell::<RwSignal<bool>>();
    mount_with_i18n(&wrapper, move || {
        let toggled = RwSignal::new(false);
        set_toggled.set(Some(toggled));
        let selected: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        view! {
            <AnalyzedWordItem
                analyzed_word=analyzed_word(false, Some("есть".to_string()))
                selected_words=selected
                known_kanji=HashSet::new()
                on_toggle=Callback::new(move |()| toggled.set(true))
            />
        }
        .into_any()
    });
    let toggled = get_toggled.get().expect("captured");
    tick().await;

    let item = wrapper
        .query_selector("[data-testid=\"words-drawer-item\"]")
        .unwrap()
        .unwrap();
    let text = item.text_content().unwrap();
    assert!(text.contains("食べる"), "the word must render; got: {text}");
    assert!(
        text.contains("есть"),
        "the meaning must render; got: {text}"
    );

    // Act: click the row
    item.unchecked_into::<web_sys::HtmlElement>().click();
    tick().await;

    assert!(toggled.get(), "click must invoke on_toggle");
}

#[wasm_bindgen_test]
async fn analyzed_word_item_no_translation_word_ignores_clicks() {
    let wrapper = create_wrapper();
    let (set_toggled, get_toggled) = shared_cell::<RwSignal<bool>>();
    mount_with_i18n(&wrapper, move || {
        let toggled = RwSignal::new(false);
        set_toggled.set(Some(toggled));
        let selected: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        view! {
            <AnalyzedWordItem
                analyzed_word=analyzed_word(false, None)
                selected_words=selected
                known_kanji=HashSet::new()
                on_toggle=Callback::new(move |()| toggled.set(true))
            />
        }
        .into_any()
    });
    let toggled = get_toggled.get().expect("captured");
    tick().await;

    // Act: click the row of a word that has no translation
    wrapper
        .query_selector("[data-testid=\"words-drawer-item\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    // Assert: the click must not toggle selection
    assert!(
        !toggled.get(),
        "no-translation word must ignore clicks (cannot be added)"
    );
}

/// Счётный суффикс-кандидат (issue #415): строка помечается бейджем типа —
/// юзер видит, что заведётся counter-карта, ещё до добавления.
#[wasm_bindgen_test]
async fn analyzed_word_item_counter_candidate_shows_type_badge() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, move || {
        let selected: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        let mut word = analyzed_word(false, Some("длинные предметы".to_string()));
        word.is_counter = true;
        word.base_form = "本".to_string();
        word.part_of_speech = origa::domain::PartOfSpeech::Suffix;
        view! {
            <AnalyzedWordItem
                analyzed_word=word
                selected_words=selected
                known_kanji=HashSet::new()
                on_toggle=Callback::new(|_| {})
            />
        }
        .into_any()
    });
    tick().await;

    let badge = wrapper.query_selector("[data-testid=\"analyzed-counter-badge\"]");
    assert!(
        badge.is_ok_and(|b| b.is_some()),
        "the counter candidate must carry the type badge"
    );
}

#[wasm_bindgen_test]
async fn analyzed_word_item_known_word_ignores_clicks() {
    let wrapper = create_wrapper();
    let (set_toggled, get_toggled) = shared_cell::<RwSignal<bool>>();
    mount_with_i18n(&wrapper, move || {
        let toggled = RwSignal::new(false);
        set_toggled.set(Some(toggled));
        let selected: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        view! {
            <AnalyzedWordItem
                analyzed_word=analyzed_word(true, Some("есть".to_string()))
                selected_words=selected
                known_kanji=HashSet::new()
                on_toggle=Callback::new(move |()| toggled.set(true))
            />
        }
        .into_any()
    });
    let toggled = get_toggled.get().expect("captured");
    tick().await;

    // Act: click the row of an already-known word
    wrapper
        .query_selector("[data-testid=\"words-drawer-item\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    // Assert: the click must not toggle selection
    assert!(
        !toggled.get(),
        "known word must ignore clicks (already in the deck)"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// VocabularyCardItem: /words/:word entry point (#528)
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn word_card_link_is_a_real_anchor_and_opens_detail() {
    let wrapper = create_wrapper();
    let (set_opened, get_opened) = shared_cell::<RwSignal<Option<String>>>();
    mount_with_i18n(&wrapper, move || {
        let opened = RwSignal::new(None::<String>);
        set_opened.set(Some(opened));
        let on_open = Callback::new(move |w: String| opened.set(Some(w)));
        // reversed_study_card keeps "translation" as the word surface; the
        // link test needs a real Japanese word (serde fixture, same as the
        // lesson tests) so the encoded href is meaningful.
        let card = origa::domain::StudyCard::new(
            serde_json::from_str(
                r#"{"Vocabulary":{"word":{"text":"ねこ"},"reverse_side":null,"pos":null}}"#,
            )
            .expect("deserialize vocab card fixture"),
        );
        view! {
            <VocabularyCardItem
                study_card=card
                native_language=Signal::from(origa::domain::NativeLanguage::Russian)
                on_open_detail_cb=on_open
                known_kanji=HashSet::new()
                on_toggle_favorite=Callback::new(|_| ())
                on_mark_as_known=Callback::new(|_| ())
                on_delete=Callback::new(|_| ())
                is_deleting=Signal::from(false)
            />
        }
        .into_any()
    });
    tick().await;

    let link = wrapper
        .query_selector("[data-testid=\"words-card-word-link\"]")
        .unwrap()
        .expect("word link rendered")
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    // Native link semantics for a11y/middle-click: a real <a> with an
    // encoded href (not a click-only div).
    assert_eq!(
        link.tag_name().to_ascii_lowercase(),
        "a",
        "entry point must be an anchor"
    );
    assert_eq!(
        link.get_attribute("href").unwrap_or_default(),
        "/words/%E3%81%AD%E3%81%93",
        "href carries the URL-encoded word"
    );

    // A plain click upgrades to in-SPA navigation via the injected
    // callback (the router lives in the parent).
    link.click();
    tick().await;
    assert_eq!(
        get_opened.get().expect("captured").get(),
        Some("ねこ".to_string()),
        "plain click runs the SPA navigation callback"
    );
}
