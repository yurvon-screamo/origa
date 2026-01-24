use leptos::*;
use leptos_router::*;
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
pub fn ResponsiveGrid(
    #[prop(into)] children: Children,
) -> impl IntoView {
    let breakpoints = create_signal(Breakpoint::Md); // Fixed to md for now
    
    let grid_columns = create_memo(move |_| {
        match breakpoints.get() {
            Breakpoint::Xs => "1fr",
            Breakpoint::Sm => "repeat(2, 1fr)",
            Breakpoint::Md => "repeat(3, 1fr)",
            _ => "repeat(4, 1fr)"
        }
    });
    
    view! {
        <Grid
            columns=grid_columns.get()
            gap="16px"
            padding="16px"
        >
            {children()}
        </Grid>
    }
}

#[component]
pub fn LoadingState() -> impl IntoView {
    view! {
        <div class="loading-spinner">
            <Spinner size="large" />
        </div>
    }
}

#[component]
pub fn ErrorMessage(
    #[prop(into)] message: String,
) -> impl IntoView {
    view! {
        <div class="error-message">
            <p>{message}</p>
        </div>
    }
}