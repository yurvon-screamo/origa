use leptos::prelude::*;

#[component]
pub fn Stamp(
    #[prop(optional, into)] _text: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    };

    view! {
        <div class="stamp" data-testid=test_id_val>{move || _text.get()}</div>
    }
}
