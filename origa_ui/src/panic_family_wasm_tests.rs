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
//! - `set()`/`update()` on a disposed signal are **silent** in release
//!   semantics (`try_update` → `None`), so overdue closures only crash
//!   when they READ.
//! - `try_get_untracked()` reports the same disposal state as `None` —
//!   the semantics dispose-guards in overdue closures must use.
//!
//! The committed tests pin the guard semantics the fixes rely on; the
//! panicking shapes are documented by the protocol in the task body
//! because no wasm32 test can catch a trap without killing the runner.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen_test::*;

use crate::test_support::{create_wrapper, mount_disposable, shared_cell};

wasm_bindgen_test_configure!(run_in_browser);

/// Mounts a wrapper with a page-scoped `RwSignal<u32>` captured out of the
/// mount closure, then disposes the mount (drop of the handle = owner
/// disposal, mirroring a route unmount).
fn mount_page_with_signal() -> RwSignal<u32> {
    let wrapper = create_wrapper();
    let (tx, rx) = shared_cell::<RwSignal<u32>>();
    let handle = mount_disposable(&wrapper, move || {
        let signal = RwSignal::new(42u32);
        tx.set(Some(signal));
        view! { <div data-testid="panic-family-host" /> }.into_any()
    });
    let signal = rx.get().expect("page-scoped signal must be captured");
    drop(handle); // route unmount: the owner and its signals are disposed
    signal
}

// ═══════════════════════════════════════════════════════════════════════
// RUST-5 root class: an overdue spawn_local task resumes after its page
// is gone. Reads panic (trap — see the module docs), writes stay silent,
// and `try_*` probes report the disposal so guards can fence the task.
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn overdue_task_sees_disposal_via_try_probes_and_silent_writes() {
    let signal = mount_page_with_signal();

    let detected = std::rc::Rc::new(std::cell::Cell::new(false));
    let write_silent = std::rc::Rc::new(std::cell::Cell::new(true));
    let flag1 = detected.clone();
    let flag2 = write_silent.clone();

    // The exact overdue shape from the Sentry breadcrumbs: the closure
    // spawned on click resumes AFTER its owner was disposed and touches
    // the page signals.
    spawn_local(async move {
        signal.set(7); // try_update → None: must NOT panic
        flag2.set(true);

        // Non-panicking probe of the same disposal state `get_untracked()`
        // would trap on — the semantics dispose-guards should use.
        flag1.set(signal.try_get_untracked().is_none());
    });

    // Yield until the spawned task runs to completion.
    for _ in 0..50 {
        leptos::task::tick().await;
        if detected.get() {
            break;
        }
    }

    assert!(
        write_silent.get(),
        "set-after-dispose must stay silent (release semantics)"
    );
    assert!(
        detected.get(),
        "try_get_untracked must report disposal to an overdue task (guard semantics)"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// /phrases notify wave (experiment): refresh_trigger update after the
// card list re-rendered — the Sentry breadcrumb pattern «каждый второй
// клик по карточной кнопке паникует». Mirrors the page wiring: an
// Effect subscribed to the trigger spawns a reload task; card memos
// subscribe to the same trigger; the list re-render drops old cards.
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

        // The phrase_card_item wiring: card memos subscribe to the same
        // trigger and read on every wave.
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

    // The «every second click» pattern: list re-render (cards replaced)
    // between trigger updates, then the next click fires the wave again.
    cards.set(Vec::new()); // old view nodes drop here
    leptos::task::tick().await;
    trigger.update(|v| *v += 1); // click #2 → notify wave over dropped cards
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
