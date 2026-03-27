use crate::loaders::{
    data_loader::{load_grammar, load_kanji, load_radical, load_vocabulary},
    dictionary::load_dictionary,
    jlpt_content_loader::load_jlpt_content,
};
use crate::pages::{
    Grammar, Home, Kanji, Lesson, Login, Onboarding, Profile, Radicals, Sets, Words,
};
use crate::store::auth_store::AuthStore;
use crate::ui_components::LoadingOverlay;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::*;
use leptos_router::path;

pub fn start_dictionary_loading(
    auth_store: AuthStore,
    is_loading: RwSignal<bool>,
    progress: RwSignal<String>,
) {
    if is_loading.get() {
        return;
    }

    is_loading.set(true);
    progress.set("Загрузка данных...".to_string());

    spawn_local(async move {
        if let Err(e) = load_vocabulary().await {
            tracing::error!("Failed to load vocabulary: {e}");
        }
        if let Err(e) = load_kanji().await {
            tracing::error!("Failed to load kanji: {e}");
        }
        if let Err(e) = load_radical().await {
            tracing::error!("Failed to load radical: {e}");
        }
        if let Err(e) = load_grammar().await {
            tracing::error!("Failed to load grammar: {e}");
        }
        if let Err(e) = load_jlpt_content().await {
            tracing::error!("Failed to load jlpt_content: {e}");
        }
        if let Err(e) = load_dictionary().await {
            tracing::error!("Failed to load dictionary: {e}");
        }

        auth_store.set_dictionary_loaded();
        is_loading.set(false);
        progress.set(String::new());
    });
}

#[component]
pub fn ProtectedRoute(children: ChildrenFn) -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");

    let is_authenticated = auth_store.is_authenticated();
    let is_checking = auth_store.is_checking_session;
    let is_loading = auth_store.is_dictionary_loading;
    let progress = auth_store.dictionary_progress_message;

    Effect::new({
        let auth_store = auth_store.clone();
        move |_| {
            if !is_checking.get()
                && is_authenticated.get()
                && !auth_store.is_dictionary_loaded.get()
                && !is_loading.get()
            {
                start_dictionary_loading(auth_store.clone(), is_loading, progress);
            }
        }
    });

    move || {
        if auth_store.is_loading().get() {
            let loading_msg: Signal<String> = Signal::derive(|| "Загрузка...".to_string());
            view! {
                <LoadingOverlay message=loading_msg />
            }
            .into_any()
        } else if is_loading.get() {
            view! {
                <LoadingOverlay message=progress />
            }
            .into_any()
        } else if is_authenticated.get() {
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
