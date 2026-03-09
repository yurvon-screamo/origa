use crate::app::AuthContext;
use crate::pages::{Grammar, Home, Kanji, Lesson, Login, Profile, Sets, Words};
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_navigate;
use leptos_router::path;

#[component]
pub fn ProtectedRoute(children: ChildrenFn) -> impl IntoView {
    let auth_ctx = use_context::<AuthContext>().expect("AuthContext not provided");
    let navigate = use_navigate();

    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if !auth_ctx.is_session_loading.get() && auth_ctx.current_user.get().is_none() {
                navigate("/login", Default::default());
            }
        }
    });

    move || {
        if auth_ctx.is_session_loading.get() {
            view! {
                <div class="min-h-screen flex items-center justify-center">
                    "Загрузка..."
                </div>
            }
            .into_any()
        } else if auth_ctx.current_user.get().is_some() {
            children().into_any()
        } else {
            view! { <Login/> }.into_any()
        }
    }
}

#[component]
pub fn AppRoutes() -> impl IntoView {
    view! {
        <main class="min-h-screen paper-texture">
            <Routes fallback=|| view! { <Login/> }>
                <Route path=path!("/") view=Login />
                <Route path=path!("login") view=Login />
                <Route path=path!("home") view=|| view! { <ProtectedRoute><Home/></ProtectedRoute> } />
                <Route path=path!("profile") view=|| view! { <ProtectedRoute><Profile/></ProtectedRoute> } />
                <Route path=path!("words") view=|| view! { <ProtectedRoute><Words/></ProtectedRoute> } />
                <Route path=path!("grammar") view=|| view! { <ProtectedRoute><Grammar/></ProtectedRoute> } />
                <Route path=path!("kanji") view=|| view! { <ProtectedRoute><Kanji/></ProtectedRoute> } />
                <Route path=path!("lesson") view=|| view! { <ProtectedRoute><Lesson/></ProtectedRoute> } />
                <Route path=path!("sets") view=|| view! { <ProtectedRoute><Sets/></ProtectedRoute> } />
            </Routes>
        </main>
    }
}
