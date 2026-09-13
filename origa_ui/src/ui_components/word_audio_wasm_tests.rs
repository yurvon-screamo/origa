//! DOM-level tests for the `CURRENT_AUDIO` supersede contract in
//! [`super::word_audio`]: replacing the active audio must detach the
//! previous element's handlers BEFORE its closures are dropped, or a late
//! `error`/`ended` event fires into a dead closure ("closure invoked
//! recursively or after being dropped" — Sentry RUST-B).

#![cfg(all(target_arch = "wasm32", test))]

use std::cell::Cell;
use std::rc::Rc;

use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use wasm_bindgen_test::*;

use super::word_audio::{register_audio, stop_current_audio};

wasm_bindgen_test_configure!(run_in_browser);

fn audio_with_handlers() -> (web_sys::HtmlAudioElement, Closure<dyn FnMut()>) {
    let element = web_sys::HtmlAudioElement::new().expect("audio element");
    let on_error = Closure::<dyn FnMut()>::new(|| {});
    element.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    element.set_onended(Some(on_error.as_ref().unchecked_ref()));
    (element, on_error)
}

#[wasm_bindgen_test]
fn superseded_audio_gets_handlers_detached_and_on_stop_fired() {
    let (first, first_closure) = audio_with_handlers();
    let stop_calls = Rc::new(Cell::new(0u32));
    let counted = Rc::clone(&stop_calls);
    let on_stop: Box<dyn Fn()> = Box::new(move || counted.set(counted.get() + 1));
    register_audio(first.clone(), Some(on_stop), vec![first_closure]);

    let (second, second_closure) = audio_with_handlers();
    register_audio(second, None, vec![second_closure]);

    assert!(
        first.onerror().is_none(),
        "superseded element must not point at dropped closures"
    );
    assert!(
        first.onended().is_none(),
        "superseded element must not point at dropped closures"
    );
    assert_eq!(
        stop_calls.get(),
        1,
        "replacing the active audio must release the previous listener exactly once"
    );

    stop_current_audio();
}

#[wasm_bindgen_test]
fn on_stop_fired_from_teardown_may_reenter_register_audio() {
    // The teardown contract runs `on_stop` OUTSIDE the CURRENT_AUDIO
    // borrow, so a listener that re-registers audio re-entrantly must
    // not panic with "RefCell already borrowed". The nested on_stop is
    // an inert noop so the recursion is exactly one level deep.
    let reentered = Rc::new(Cell::new(false));
    let flag = Rc::clone(&reentered);

    let (outer, outer_closure) = audio_with_handlers();
    let on_stop: Box<dyn Fn()> = Box::new(move || {
        if flag.get() {
            return;
        }
        flag.set(true);
        let (inner, inner_closure) = audio_with_handlers();
        let noop: Box<dyn Fn()> = Box::new(|| {});
        register_audio(inner, Some(noop), vec![inner_closure]);
    });
    register_audio(outer, Some(on_stop), vec![outer_closure]);

    let (next, next_closure) = audio_with_handlers();
    register_audio(next, None, vec![next_closure]);

    assert!(
        reentered.get(),
        "the superseded on_stop must have run and re-registered audio"
    );
    stop_current_audio();
}
