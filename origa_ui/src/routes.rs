use crate::loaders::{
    data_loader::{load_grammar, load_kanji, load_radical, load_vocabulary},
    dictionary::load_dictionary,
    jlpt_content_loader::load_jlpt_content,
};
use crate::pages::{
    Grammar, Home, Kanji, Lesson, Login, Onboarding, Profile, Radicals, Sets, Words,
};
use crate::store::auth_store::AuthStore;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::*;
use leptos_router::path;
use std::sync::OnceLock;

static DICTIONARY_LOADING: OnceLock<RwSignal<bool>> = OnceLock::new();
static PROGRESS_MESSAGE: OnceLock<RwSignal<String>> = OnceLock::new();

fn is_dictionary_loading() -> RwSignal<bool> {
    *DICTIONARY_LOADING.get_or_init(|| RwSignal::new(false))
}

fn progress_message() -> RwSignal<String> {
    *PROGRESS_MESSAGE.get_or_init(|| RwSignal::new(String::new()))
}

pub fn start_dictionary_loading(auth_store: AuthStore) {
    let loading = is_dictionary_loading();
    let progress = progress_message();
    let disposed = StoredValue::new(());

    loading.set(true);
    progress.set("Загрузка данных...".to_string());

    spawn_local(async move {
        let _ = load_vocabulary().await;
        let _ = load_kanji().await;
        let _ = load_radical().await;
        let _ = load_grammar().await;
        let _ = load_jlpt_content().await;
        let _ = load_dictionary().await;

        if disposed.is_disposed() {
            return;
        }
        auth_store.set_dictionary_loaded();
        loading.set(false);
        progress.set(String::new());
    });
}

#[component]
pub fn ProtectedRoute(children: ChildrenFn) -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");

    let is_authenticated = auth_store.is_authenticated();
    let is_checking = auth_store.is_checking_session;
    let is_loading = is_dictionary_loading();
    let progress = progress_message();

    Effect::new({
        let auth_store = auth_store.clone();
        move |_| {
            if !is_checking.get()
                && is_authenticated.get()
                && !auth_store.is_dictionary_loaded.get()
            {
                start_dictionary_loading(auth_store.clone());
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
        } else if is_loading.get() {
            view! {
                <div class="fixed inset-0 z-[9999] flex items-center justify-center bg-[var(--bg-primary)]">
                    <div class="text-center">
                        <div class="animate-spin w-12 h-12 border-4 border-[var(--border)] border-t-[var(--accent)] rounded-full mx-auto mb-4"></div>
                        <p class="text-[var(--text-secondary)]">{move || progress.get()}</p>
                    </div>
                </div>
            }.into_any()
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
