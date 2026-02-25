use crate::pages::{Grammar, Home, Kanji, Lesson, Login, Profile, Sets, Words};
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
                <Route path=path!("words") view=Words />
                <Route path=path!("grammar") view=Grammar />
                <Route path=path!("kanji") view=Kanji />
                <Route path=path!("lesson") view=Lesson />
                <Route path=path!("sets") view=Sets />
            </Routes>
        </main>
    }
}
