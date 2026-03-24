use crate::pages::{
    Grammar, Home, Kanji, Lesson, Login, Onboarding, Profile, Radicals, Sets, Words,
};
use crate::store::auth_store::AuthStore;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_navigate;
use leptos_router::path;

#[component]
pub fn ProtectedRoute(children: ChildrenFn) -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let navigate = use_navigate();

    let auth_store_for_effect = auth_store.clone();
    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            let store = auth_store_for_effect.clone();
            if store.is_checking_session.get() {
                return;
            }
            if !store.is_authenticated().get() {
                navigate("/login", Default::default());
            }
        }
    });

    move || {
        if auth_store.is_loading().get() {
            view! {
                <div class="min-h-screen flex items-center justify-center">
                    "Загрузка..."
                </div>
            }
            .into_any()
        } else if auth_store.is_authenticated().get() {
            children().into_any()
        } else {
            view! { <Login/> }.into_any()
        }
    }
}

#[component]
pub fn AppRoutes() -> impl IntoView {
    view! {
        <main class="paper-texture">
            <Routes fallback=|| view! { <Login/> }>
                <Route path=path!("/") view=Login />
                <Route path=path!("login") view=Login />
                <Route path=path!("onboarding") view=|| view! { <ProtectedRoute><Onboarding/></ProtectedRoute> } />
                <Route path=path!("home") view=|| view! { <ProtectedRoute><Home/></ProtectedRoute> } />
                <Route path=path!("profile") view=|| view! { <ProtectedRoute><Profile/></ProtectedRoute> } />
                <Route path=path!("words") view=|| view! { <ProtectedRoute><Words/></ProtectedRoute> } />
                <Route path=path!("grammar") view=|| view! { <ProtectedRoute><Grammar/></ProtectedRoute> } />
                <Route path=path!("kanji") view=|| view! { <ProtectedRoute><Kanji/></ProtectedRoute> } />
                <Route path=path!("radicals") view=|| view! { <ProtectedRoute><Radicals/></ProtectedRoute> } />
                <Route path=path!("lesson") view=|| view! { <ProtectedRoute><Lesson/></ProtectedRoute> } />
                <Route path=path!("sets") view=|| view! { <ProtectedRoute><Sets/></ProtectedRoute> } />
            </Routes>
        </main>
    }
}
