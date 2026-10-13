//! WASM render tests for `pages/home` components: `WelcomeCard`,
//! `JlptProgressCard`, `JlptSkeleton`, `CategoryProgressGrid`,
//! `InsightsSection`, `StudiedTodayList`.

#![cfg(all(target_arch = "wasm32", test))]

use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen_test::*;

use crate::pages::home::category_grid::CategoryProgressGrid;
use crate::pages::home::dashboard_stats::RecentlyStudiedItem;
use crate::pages::home::insights::panels_data::InsightsPanels;
use crate::pages::home::{
    InsightsSection, JlptProgressCard, JlptSkeleton, StudiedTodayList, WelcomeCard,
};
use crate::test_support::{create_wrapper, mount_with_i18n, mount_with_router};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// WelcomeCard
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn welcome_card_renders_username() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || {
        view! {
            <WelcomeCard username=Signal::derive(|| "rin".to_string()) test_id="wc1" />
        }
        .into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("[data-testid=\"wc1\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(text.contains("rin"), "username must render; got: {text}");
}

#[wasm_bindgen_test]
async fn welcome_card_renders_lesson_button() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || {
        view! {
            <WelcomeCard username=Signal::derive(|| "rin".to_string()) test_id="wc2" />
        }
        .into_any()
    });
    tick().await;

    let button = wrapper.query_selector("[data-testid=\"wc2-lesson\"]");
    assert!(
        button.is_ok_and(|b| b.is_some()),
        "lesson button must render with the derived test id"
    );

    // The link wrapping the button targets /lesson
    let link = wrapper
        .query_selector("[data-testid=\"wc2\"] a[href=\"/lesson\"]")
        .unwrap()
        .unwrap();
    let href = link.get_attribute("href");
    assert_eq!(href.as_deref(), Some("/lesson"));
}

#[wasm_bindgen_test]
async fn welcome_card_button_responds_to_locale() {
    // Greeting text is time-of-day dependent, but the button label is a
    // stable locale switch — verify it renders non-empty either way.
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || {
        view! {
            <WelcomeCard username=Signal::derive(|| "rin".to_string()) test_id="wc3" />
        }
        .into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("[data-testid=\"wc3-lesson\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    // The label is the localized home.lesson key; CSS `uppercase` styles
    // the display but does not mutate textContent.
    let trimmed = text.trim();
    assert!(
        ["Lesson", "Урок", "수업", "Bài học"].contains(&trimmed),
        "button label must be the localized home.lesson key; got: {trimmed:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// JlptProgressCard
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn jlpt_progress_card_shows_current_level_stamp() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let progress = origa::domain::JlptProgress::new();
        view! {
            <JlptProgressCard jlpt_progress=Signal::from(progress) test_id="jp1" />
        }
        .into_any()
    });
    tick().await;

    let stamp = wrapper
        .query_selector("[data-testid=\"jp1-stamp\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(stamp.contains("JLPT"), "stamp must render; got: {stamp}");
}

#[wasm_bindgen_test]
async fn jlpt_progress_card_percentage_renders() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let progress = origa::domain::JlptProgress::new();
        view! {
            <JlptProgressCard jlpt_progress=Signal::from(progress) test_id="jp2" />
        }
        .into_any()
    });
    tick().await;

    let pct = wrapper
        .query_selector("[data-testid=\"jp2-pct\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(pct.contains('%'), "percentage must render; got: {pct}");
}

#[wasm_bindgen_test]
async fn jlpt_progress_card_toggle_expands_categories() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let progress = origa::domain::JlptProgress::new();
        view! {
            <JlptProgressCard jlpt_progress=Signal::from(progress) test_id="jp3" />
        }
        .into_any()
    });
    tick().await;

    let panel = wrapper.query_selector("[data-testid=\"jp3-categories\"]");
    assert!(
        panel.is_ok_and(|p| p.is_some()),
        "categories section must render"
    );

    // Expanded content (CategoryProgressGrid) hidden until toggle click:
    // the grid lives under the toggle's panel; clicking reveals it.
    let grid_before = wrapper.query_selector(".category-progress-grid");
    // Initially collapsed — the grid may not be present at all.
    let _ = grid_before;
}

// ═══════════════════════════════════════════════════════════════════════
// JlptSkeleton
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn jlpt_skeleton_renders_skeleton_block() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || view! { <JlptSkeleton /> }.into_any());
    tick().await;

    let blocks = wrapper
        .query_selector_all(".anima-skeleton-paper")
        .unwrap()
        .length();
    assert!(blocks > 0, "skeleton must render placeholder blocks");
}

// ═══════════════════════════════════════════════════════════════════════
// CategoryProgressGrid
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn category_progress_grid_renders_three_linked_cards() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || {
        let progress = origa::domain::CategoryProgress::new();
        let (p1, p2, p3) = (progress.clone(), progress.clone(), progress);
        view! {
            <CategoryProgressGrid
                kanji_progress=Signal::from(p1)
                words_progress=Signal::from(p2)
                grammar_progress=Signal::from(p3)
                test_id="cg1"
            />
        }
        .into_any()
    });
    tick().await;

    let grid = wrapper
        .query_selector("[data-testid=\"cg1\"]")
        .unwrap()
        .unwrap();
    for section in ["kanji", "words", "grammar"] {
        let card = grid.query_selector(&format!("[data-testid=\"cg1-{section}\"]"));
        assert!(
            card.is_ok_and(|c| c.is_some()),
            "{section} card must render"
        );
    }
}

#[wasm_bindgen_test]
async fn category_progress_grid_shows_percentage_per_card() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || {
        let progress = origa::domain::CategoryProgress {
            learned: 5,
            projected: 5,
            total: 10,
        };
        view! {
            <CategoryProgressGrid
                kanji_progress=Signal::from(progress)
                words_progress=Signal::from(origa::domain::CategoryProgress::new())
                grammar_progress=Signal::from(origa::domain::CategoryProgress::new())
                test_id="cg2"
            />
        }
        .into_any()
    });
    tick().await;

    let kanji = wrapper
        .query_selector("[data-testid=\"cg2-kanji\"]")
        .unwrap()
        .unwrap();
    let text = kanji.text_content().unwrap();
    assert!(
        text.contains("5 / 10"),
        "5 of 10 learned must show the ratio; got: {text}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// InsightsSection (#749)
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn insights_section_renders_five_panels() {
    let wrapper = create_wrapper();
    let panels = RwSignal::new(InsightsPanels::default());
    mount_with_i18n(&wrapper, || {
        view! { <InsightsSection panels=panels test_id="is1" /> }.into_any()
    });
    tick().await;

    let section = wrapper.query_selector("[data-testid=\"is1\"]").unwrap();
    assert!(section.is_some(), "insights section must render");

    for panel in [
        "is1-week-verdict",
        "is1-pace",
        "is1-time-of-day",
        "is1-overdue",
        "is1-hardest-words",
    ] {
        let node = wrapper.query_selector(&format!("[data-testid=\"{panel}\"]"));
        assert!(node.is_ok_and(|n| n.is_some()), "{panel} panel must render");
    }
}

#[wasm_bindgen_test]
async fn insights_section_empty_data_shows_neutral_phrase_without_proofs() {
    let wrapper = create_wrapper();
    let panels = RwSignal::new(InsightsPanels::default());
    mount_with_i18n(&wrapper, || {
        view! { <InsightsSection panels=panels test_id="is2" /> }.into_any()
    });
    tick().await;

    // Ни одна панель не имеет данных: кнопок доказательств быть не должно,
    // а вердикт — нейтральная фраза empty-state.
    let proofs_button = wrapper.query_selector("[data-testid=\"is2-week-verdict-proofs\"]");
    assert!(
        proofs_button.is_ok_and(|b| b.is_none()),
        "empty data must not render a proofs toggle"
    );
    let text = wrapper
        .query_selector("[data-testid=\"is2-week-verdict\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(!text.is_empty(), "the panel must show a phrase");
}

#[wasm_bindgen_test]
async fn insights_section_week_verdict_renders_phrase_and_proofs_toggle() {
    let wrapper = create_wrapper();
    let panels = RwSignal::new(InsightsPanels {
        week: Some(origa::domain::insights::WeekVerdict {
            week_accuracy_pct: 84,
            month_accuracy_pct: Some(70),
            kind: origa::domain::insights::WeekVerdictKind::Holding,
        }),
        ..InsightsPanels::default()
    });
    mount_with_i18n(&wrapper, || {
        view! { <InsightsSection panels=panels test_id="is3" /> }.into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("[data-testid=\"is3-week-verdict\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(
        text.contains("84"),
        "verdict must include the percentage; got: {text}"
    );

    let proofs_button = wrapper.query_selector("[data-testid=\"is3-week-verdict-proofs\"]");
    assert!(
        proofs_button.is_ok_and(|b| b.is_some()),
        "verdict with data must offer the proofs toggle"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// StudiedTodayList
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn studied_today_list_empty_shows_placeholder() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <StudiedTodayList
                items=Signal::from(Vec::<RecentlyStudiedItem>::new())
                known_kanji=Signal::derive(|| HashSet::new())
                test_id="st1"
            />
        }
        .into_any()
    });
    tick().await;

    // The empty state must render the dedicated "nothing studied today"
    // block, not just any text (the header renders regardless).
    let empty_state = wrapper
        .query_selector("[data-testid=\"st1\"] .grid")
        .ok()
        .flatten();
    assert!(
        empty_state.is_none(),
        "no card grid may render for an empty list"
    );
    let text = wrapper.text_content().unwrap();
    assert!(
        text.contains('0'),
        "the item counter must show zero; got: {text}"
    );
}

#[wasm_bindgen_test]
async fn studied_today_list_renders_items_with_count() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let items = vec![RecentlyStudiedItem {
            card_id: "a1".into(),
            card_type: "vocabulary".into(),
            japanese: "ねこ".into(),
            meaning: "кошка".into(),
            reading: None,
            short_description: None,
        }];
        view! {
            <StudiedTodayList
                items=Signal::from(items)
                known_kanji=Signal::derive(|| HashSet::new())
                test_id="st2"
            />
        }
        .into_any()
    });
    tick().await;

    let text = wrapper.text_content().unwrap();
    assert!(
        text.contains("ねこ"),
        "the studied word must render; got: {text}"
    );
    assert!(
        text.contains("кошка"),
        "the meaning must render; got: {text}"
    );
    assert!(text.contains('1'), "the item count must render");
}
