use leptos::prelude::*;

pub fn derive_test_id(base: Signal<String>, suffix: &str) -> Signal<String> {
    let suffix = suffix.to_string();
    Signal::derive(move || {
        let val = base.get();
        if val.is_empty() {
            suffix.clone()
        } else {
            format!("{}-{}", val, suffix)
        }
    })
}
