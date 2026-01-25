use leptos::prelude::*;
use thaw::*;

#[component]
pub fn CounterBtn(#[prop(default = 1)] increment: i32) -> impl IntoView {
    let (count, set_count) = signal(0);
    view! {
        <Button on_click=move |_| {
            set_count.set(count.get() + increment)
        }>

            "Click me: " {count}
        </Button>
    }
}
