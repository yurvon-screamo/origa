use crate::pages::{Home, Login, Profile};
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

#[component]
pub fn AppRoutes() -> impl IntoView {
    view! {
        <main class="min-h-screen paper-texture">
            <Routes fallback=|| view! { <Login/> }>
                <Route path=path!("/") view=Login />
                <Route path=path!("login") view=Login />
                <Route path=path!("home") view=Home />
                <Route path=path!("profile") view=Profile />
            </Routes>
        </main>
    }
}
