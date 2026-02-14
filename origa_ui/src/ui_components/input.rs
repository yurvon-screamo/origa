use leptos::either::Either;
use leptos::ev::Event;
use leptos::prelude::*;

#[component]
pub fn Input(
    #[prop(optional)] value: RwSignal<String>,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] rows: Option<usize>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] on_change: Option<Callback<Event>>,
) -> impl IntoView {
    let base_class = "input-field";
    let textarea_class = "resize-none";
    let full_class = if rows.is_some() {
        format!("{} {} {}", base_class, textarea_class, class)
    } else {
        format!("{} {}", base_class, class)
    };

    if let Some(r) = rows {
        Either::Left(view! {
            <textarea
                class=full_class
                placeholder=placeholder
                disabled=disabled
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
                placeholder=placeholder
                disabled=disabled
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
