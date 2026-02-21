use leptos::either::Either;
use leptos::ev::Event;
use leptos::prelude::*;

#[component]
pub fn Input(
    #[prop(optional)] value: RwSignal<String>,
    #[prop(optional, into)] placeholder: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] rows: Signal<Option<usize>>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional)] on_change: Option<Callback<Event>>,
) -> impl IntoView {
    let full_class = move || {
        let base_class = "input-field";
        let textarea_class = "resize-none";
        if rows.get().is_some() {
            format!("{} {} {}", base_class, textarea_class, class.get())
        } else {
            format!("{} {}", base_class, class.get())
        }
    };

    move || {
        if let Some(r) = rows.get() {
            Either::Left(view! {
                <textarea
                    class=full_class
                    placeholder=move || placeholder.get()
                    disabled=move || disabled.get()
                    rows=r
                    bind:value=value
                    on:change=move |ev| {
                        if let Some(on_change) = on_change {
                            on_change.run(ev);
                        }
                    }
                />
            })
        } else {
            Either::Right(view! {
                <input
                    type="text"
                    class=full_class
                    placeholder=move || placeholder.get()
                    disabled=move || disabled.get()
                    bind:value=value
                    on:change=move |ev| {
                        if let Some(on_change) = on_change {
                            on_change.run(ev);
                        }
                    }
                />
            })
        }
    }
}
