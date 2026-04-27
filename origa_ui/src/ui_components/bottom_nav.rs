use crate::i18n::use_i18n;
use crate::store::auth_store::AuthStore;
use crate::ui_components::{Logo, LogoSize, derive_test_id};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

#[component]
pub fn BottomTabBar(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let location = use_location();

    let home_label = Signal::derive(move || i18n.get_keys().home().home_tab().inner().to_string());
    let words_label = Signal::derive(move || i18n.get_keys().home().words().inner().to_string());
    let grammar_label =
        Signal::derive(move || i18n.get_keys().home().grammar().inner().to_string());
    let kanji_label = Signal::derive(move || i18n.get_keys().home().kanji().inner().to_string());
    let phrases_label =
        Signal::derive(move || i18n.get_keys().home().phrases().inner().to_string());
    let profile_label =
        Signal::derive(move || i18n.get_keys().home().profile().inner().to_string());

    let is_visible = Signal::derive(move || {
        let authenticated = auth_store.is_authenticated().get();
        let path = location.pathname.get();
        let hidden_path = path == "/lesson" || path == "/onboarding";
        authenticated && !hidden_path
    });

    let is_home_active = Signal::derive(move || {
        let path = location.pathname.get();
        path.starts_with("/home") || path == "/" || path.is_empty()
    });
    let is_words_active = Signal::derive(move || {
        let path = location.pathname.get();
        path.starts_with("/words") || path.starts_with("/sets")
    });
    let is_grammar_active = Signal::derive(move || location.pathname.get().starts_with("/grammar"));
    let is_kanji_active = Signal::derive(move || location.pathname.get().starts_with("/kanji"));
    let is_phrases_active = Signal::derive(move || location.pathname.get().starts_with("/phrases"));
    let is_profile_active = Signal::derive(move || location.pathname.get().starts_with("/profile"));

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <Show when=move || is_visible.get()>
            <nav class="bottom-tab-bar md:hidden" data-testid=test_id_val>
                <BottomTabItem
                    href="/home"
                    icon=icondata::LuHouse
                    use_logo=true
                    label=home_label
                    is_active=is_home_active
                    test_id=derive_test_id(test_id, "tab-home")
                />
                <BottomTabItem
                    href="/words"
                    icon=icondata::LuLanguages
                    label=words_label
                    is_active=is_words_active
                    test_id=derive_test_id(test_id, "tab-words")
                />
                <BottomTabItem
                    href="/grammar"
                    icon=icondata::LuPencilLine
                    label=grammar_label
                    is_active=is_grammar_active
                    test_id=derive_test_id(test_id, "tab-grammar")
                />
                <BottomTabItem
                    href="/kanji"
                    icon=icondata::LuBookOpen
                    label=kanji_label
                    is_active=is_kanji_active
                    test_id=derive_test_id(test_id, "tab-kanji")
                />
                <BottomTabItem
                    href="/phrases"
                    icon=icondata::LuMessageSquare
                    label=phrases_label
                    is_active=is_phrases_active
                    test_id=derive_test_id(test_id, "tab-phrases")
                />
                <BottomTabItem
                    href="/profile"
                    icon=icondata::LuUser
                    label=profile_label
                    is_active=is_profile_active
                    test_id=derive_test_id(test_id, "tab-profile")
                />
            </nav>
        </Show>
    }
}

#[component]
fn BottomTabItem(
    #[prop(into)] href: String,
    icon: icondata::Icon,
    #[prop(optional)] use_logo: bool,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] is_active: Signal<bool>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let class = Signal::derive(move || {
        if is_active.get() {
            "bottom-tab-item active".to_string()
        } else {
            "bottom-tab-item".to_string()
        }
    });

    view! {
        <A href=href attr:class=class attr:data-testid=test_id_val attr:aria-current=move || if is_active.get() { "page" } else { "false" }>
            {if use_logo {
                view! { <Logo size=LogoSize::Sm /> }.into_any()
            } else {
                view! { <Icon icon=icon width="24" height="24" /> }.into_any()
            }}
            <span class="bottom-tab-item-label">{label}</span>
        </A>
    }
}
