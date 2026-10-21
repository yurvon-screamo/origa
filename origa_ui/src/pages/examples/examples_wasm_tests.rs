//! WASM render tests for `pages/examples`: the page header built by
//! `card_list_page_header` (Router-based PageHeader with the info
//! tooltip) and `ExampleCardItem` (sentence/translation degrade without
//! the CDN detail cache). Mirrors the phrases page tests (#823).

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen_test::*;

use crate::pages::examples::example_card_item::ExampleCardItem;
use crate::pages::shared::card_list_page_header;
use crate::test_support::{create_wrapper, mount_with_i18n, mount_with_router};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn examples_header_renders_title_and_info_tooltip() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || {
        card_list_page_header("examples", || "Examples".to_string(), || "hint".to_string())
            .into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"examples-header\"]")
            .ok()
            .flatten()
            .is_some(),
        "header root must render"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"examples-info-icon\"]")
            .unwrap()
            .is_some(),
        "the info icon must render"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// ExampleCardItem (sentence/translation degrade without the CDN store)
// ═══════════════════════════════════════════════════════════════════════

fn example_study_card() -> origa::domain::StudyCard {
    use origa::domain::{Card, ExampleCard, StudyCard};
    StudyCard::new(Card::Example(ExampleCard::new("たべる", 1)))
}

#[wasm_bindgen_test]
async fn example_card_item_without_store_renders_skeleton_and_status() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let trigger = RwSignal::new(0u32);
        view! {
            <ExampleCardItem
                study_card=example_study_card()
                native_language=Signal::from(origa::domain::NativeLanguage::Russian)
                actions=crate::pages::shared::CardItemActions {
                    on_toggle_favorite: Callback::new(|_| ()),
                    on_mark_as_known: Callback::new(|_| ()),
                    on_delete: Callback::new(|_| ()),
                    is_deleting: Signal::from(false),
                }
                details_trigger=trigger
            />
        }
        .into_any()
    });
    tick().await;

    let item = wrapper
        .query_selector("[data-testid=\"examples-card-item\"]")
        .unwrap()
        .unwrap();
    // Without the example store the sentence degrades to the skeleton
    let skeleton = item.query_selector(".anima-skeleton-paper");
    assert!(
        skeleton.is_ok_and(|sk| sk.is_some()),
        "missing sentence must render the loading skeleton"
    );
    // The status tag and FSRS metrics render regardless
    assert!(
        item.query_selector(".tag").is_ok_and(|t| t.is_some()),
        "status tag must render"
    );
    // The owner word renders from the domain card (no CDN data needed)
    let word = item.query_selector("[data-testid=\"examples-card-word\"]");
    assert!(
        word.is_ok_and(
            |w| w.is_some_and(|el| el.text_content().unwrap_or_default().contains("たべる"))
        ),
        "the owner word must render from the domain card"
    );
}

#[wasm_bindgen_test]
async fn example_card_item_delete_opens_confirm_modal() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let trigger = RwSignal::new(0u32);
        view! {
            <ExampleCardItem
                study_card=example_study_card()
                native_language=Signal::from(origa::domain::NativeLanguage::Russian)
                actions=crate::pages::shared::CardItemActions {
                    on_toggle_favorite: Callback::new(|_| ()),
                    on_mark_as_known: Callback::new(|_| ()),
                    on_delete: Callback::new(|_| ()),
                    is_deleting: Signal::from(false),
                }
                details_trigger=trigger
            />
        }
        .into_any()
    });
    tick().await;

    use wasm_bindgen::JsCast;
    wrapper
        .query_selector("[data-testid=\"examples-card-item-delete-btn\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"examples-delete-modal\"]")
            .unwrap()
            .is_some(),
        "delete click must open the confirmation modal"
    );
}
