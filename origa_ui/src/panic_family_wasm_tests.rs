//! Disposal-semantics tests for the Sentry WASM panic family (task
//! #1443, RUST-4/5/6): reads of disposed reactive values from closures
//! that outlive their owner — the root class behind the family (see the
//! task body, «Факты Design-стадии»).
//!
//! Platform contract this suite pins down (verified live on this stack,
//! see the task body for the reproduction protocol):
//!
//! - On `wasm32-unknown-unknown` a Rust panic is a **bailout trap**
//!   (`__rust_start_panic → __rust_abort → unreachable`): `catch_unwind`
//!   cannot intercept it, `should_panic` sees an empty payload, and
//!   destructors (including `RefCell` borrow guards) never run. A direct
//!   disposed read therefore kills the test runner — it was reproduced
//!   with this exact suite before the switch to the non-panicking
//!   probes below (panic message `... has already been disposed.` +
//!   `RuntimeError: unreachable` echo, the RUST-5 → RUST-4 chain).
//! - Writes to a disposed signal — both `set()` and `update()` — are
//!   **silent** in release semantics (`try_update` → `None`); overdue
//!   closures only crash when they READ.
//!
//! The panic-family fences of this task (lazy_card_details backfill,
//! home redirect branches, password_card) fire on
//! `StoredValue::is_disposed()` — a page-scoped sentinel created in the
//! owner context. That primitive is pinned DIRECTLY
//! (`disposal_sentinel_flips_after_owner_drop` reads `is_disposed()`
//! before AND after the owner drop): if a reactive_graph / leptos bump
//! ever stops the disposal sweep from marking stored values, the whole
//! fence layer would silently go inert and this suite must go red.
//! `try_get_untracked()` / `RwSignal::is_disposed()` pin the signal-
//! level view of the same disposal state — the detection primitives an
//! overdue closure can use when it wants to check before reading.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen_test::*;

use crate::test_support::{create_wrapper, mount_disposable, shared_cell};

wasm_bindgen_test_configure!(run_in_browser);

/// Mounts a wrapper capturing a page-scoped value out of the mount
/// closure. The returned closure drops the mount once (owner disposal,
/// mirroring a route unmount); disposal itself is observed on the
/// captured primitives via their `is_disposed()`.
fn mount_page_with<T: Copy + 'static>(capture: impl FnOnce() -> T + 'static) -> (T, impl FnMut()) {
    let wrapper = create_wrapper();
    let (tx, rx) = shared_cell::<T>();
    let mut handle = Some(mount_disposable(&wrapper, move || {
        tx.set(Some(capture()));
        view! { <div data-testid="panic-family-host" /> }.into_any()
    }));
    let value = rx.get().expect("page-scoped value must be captured");
    (
        value,
        Box::new(move || {
            drop(handle.take()); // route unmount: owner disposal
        }) as Box<dyn FnMut()>,
    )
}

// ═══════════════════════════════════════════════════════════════════════
// The fence primitive itself: a page-scoped `StoredValue` sentinel flips
// `is_disposed()` exactly when the owner drops — the mechanism all three
// fences of this task (lazy_card_details, home, password_card) fire on.
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn disposal_sentinel_flips_after_owner_drop() {
    let (sentinel, mut drop_mount) = mount_page_with(|| StoredValue::new(()));

    assert!(
        !sentinel.is_disposed(),
        "the sentinel must read live while the mount is alive"
    );

    drop_mount();

    // The actual flip: read from the sentinel AFTER the owner dropped.
    // If the disposal sweep ever stops marking stored values, this is
    // the assertion that goes red while the fence layer goes inert.
    assert!(
        sentinel.is_disposed(),
        "after the owner drops, the sentinel must report disposal — \
         the primitive every fence of this task fires on"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// RUST-5 root class: an overdue spawn_local task resumes after its page
// is gone. Reads trap (see the module docs), both write shapes stay
// silent, and the signal-level disposal state is visible both via
// `is_disposed()` and via the `try_*` probes — so an overdue closure
// can fence itself off before reading.
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn overdue_task_sees_disposal_via_try_probes_and_silent_writes() {
    let (signal, mut drop_mount) = mount_page_with(|| RwSignal::new(42u32));
    drop_mount();

    // Signal-level disposal state, read after the owner dropped.
    assert!(
        signal.is_disposed(),
        "the page-scoped signal must report disposal after the mount drops"
    );

    let detected = std::rc::Rc::new(std::cell::Cell::new(false));
    let flag = detected.clone();

    // The exact overdue shape from the Sentry breadcrumbs: the closure
    // spawned on click resumes AFTER its owner was disposed and touches
    // the page signals. Both write shapes must stay silent and the
    // try-probe must agree with `is_disposed()` above — the task only
    // survives to the flag if nothing panicked along the way.
    spawn_local(async move {
        signal.set(7); // try_update → None: must NOT panic
        signal.update(|v| *v += 1); // same silent path: must NOT panic
        flag.set(signal.try_get_untracked().is_none());
    });

    // Yield until the spawned task runs to completion.
    for _ in 0..50 {
        leptos::task::tick().await;
        if detected.get() {
            break;
        }
    }

    assert!(
        detected.get(),
        "an overdue task must survive silent set+update and see disposal \
         via try_get_untracked (signal-level guard semantics)"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// /phrases notify wave (experiment): refresh_trigger update racing the
// card list state — the Sentry breadcrumb pattern «каждый второй клик по
// карточной кнопке паникует». Signal-level replica of the page wiring:
// an Effect subscribed to the trigger spawns a reload task; a second
// effect stands in for the card memos. NOTE: the replica models only the
// SIGNAL graph — the real page's view layer (mounted card components and
// their disposal during re-render) is intentionally not built here, so a
// survival here does NOT close the production point (see the task body).
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn refresh_update_survives_card_unmount_wave() {
    let wrapper = create_wrapper();
    let (cards_tx, cards_rx) = shared_cell::<RwSignal<Vec<u32>>>();
    let (trigger_tx, trigger_rx) = shared_cell::<RwSignal<u32>>();
    let (reload_runs_tx, reload_runs_rx) = shared_cell::<RwSignal<u32>>();

    let handle = mount_disposable(&wrapper, move || {
        let trigger = RwSignal::new(0u32);
        let cards = RwSignal::new(Vec::new());
        let reload_runs = RwSignal::new(0u32);

        // The card_list_page wiring: every trigger change spawns a reload
        // task that writes the list back.
        Effect::new(move |_| {
            let _ = trigger.get();
            spawn_local(async move {
                reload_runs.update(|n| *n += 1);
                cards.set(vec![1, 2, 3]);
            });
        });

        // The phrase_card_item wiring stand-in: an effect subscribed to
        // the same trigger reads the list on every wave.
        Effect::new(move |_| {
            let _ = trigger.get();
            let _ = cards.with(|c| c.len());
        });

        cards_tx.set(Some(cards));
        trigger_tx.set(Some(trigger));
        reload_runs_tx.set(Some(reload_runs));

        view! { <div data-testid="panic-family-list" /> }.into_any()
    });
    std::mem::forget(handle);

    let trigger = trigger_rx.get().expect("trigger captured");
    let cards = cards_rx.get().expect("cards captured");
    let reload_runs = reload_runs_rx.get().expect("reload counter captured");

    // Wave 1: initial reload lands.
    for _ in 0..20 {
        leptos::task::tick().await;
        if reload_runs.get_untracked() >= 1 {
            break;
        }
    }

    // The «every second click» pattern: list state replaced between
    // trigger updates, then the next click fires the wave again.
    cards.set(Vec::new());
    leptos::task::tick().await;
    trigger.update(|v| *v += 1); // click #2 → notify wave
    for _ in 0..20 {
        leptos::task::tick().await;
        if reload_runs.get_untracked() >= 2 {
            break;
        }
    }

    // The experiment asserts survival: if this panics, the /phrases
    // scenario is reproduced and the test goes red at the panic site.
    assert_eq!(reload_runs.get_untracked(), 2, "reload waves completed");
}
