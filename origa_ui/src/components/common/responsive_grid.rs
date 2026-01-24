use leptos::children::Children;
use leptos::prelude::*;
use leptos::prelude::{Get, signal};
use thaw::*;
// use leptos_use::{use_breakpoints, Breakpoint}; // Temporarily commented

#[derive(Clone, Debug)]
enum Breakpoint {
    Xs,
    Sm,
    Md,
    Lg,
}

#[component]
pub fn ResponsiveGrid(#[prop(into)] children: Children) -> impl IntoView {
    let (breakpoints, _set_breakpoints) = signal(Breakpoint::Md); // Fixed to md for now

    let grid_columns = Memo::new(move |_| match breakpoints.get() {
        Breakpoint::Xs => "1fr",
        Breakpoint::Sm => "repeat(2, 1fr)",
        Breakpoint::Md => "repeat(3, 1fr)",
        _ => "repeat(4, 1fr)",
    });

    view! {
        <div
            class="responsive-grid"
            style:display="grid"
            style:grid-template-columns=move || grid_columns.get()
            style:gap="16px"
            style:padding="16px"
        >
            {children()}
        </div>
    }
}

#[component]
pub fn LoadingState() -> impl IntoView {
    view! {
        <div class="loading-spinner">
            <Spinner size=SpinnerSize::Large />
        </div>
    }
}

#[component]
pub fn ErrorMessage(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="error-message">
            <p>{message}</p>
        </div>
    }
}
