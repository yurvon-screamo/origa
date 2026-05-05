use crate::i18n::use_i18n;
use crate::store::auth_store::AuthStore;
use crate::ui_components::avatar::AvatarSize;
use crate::ui_components::{Avatar, Logo, LogoSize, derive_test_id};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use leptos_router::hooks::use_location;
use origa::domain::User;

#[component]
pub fn Sidebar(
    current_user: RwSignal<Option<User>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
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
        let has_user = current_user.with(|u| u.is_some());
        authenticated && !hidden_path && has_user
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

    let avatar_initials = Signal::derive(move || {
        current_user
            .with(|u| u.as_ref().map(|u| u.username().to_uppercase()))
            .unwrap_or_default()
    });

    view! {
        <Show when=move || is_visible.get()>
            <aside class="sidebar" data-testid=test_id_val>
                <div class="sidebar-logo">
                    <Logo size=LogoSize::Sm test_id=derive_test_id(test_id, "logo") />
                </div>
                <div class="border-b border-[var(--border-light)] mx-6"></div>
                <nav class="sidebar-nav">
                    <SidebarNavItem
                        href="/home"
                        icon=icondata::LuHouse
                        use_logo=true
                        label=home_label
                        is_active=is_home_active
                        test_id=derive_test_id(test_id, "item-home")
                    />
                    <SidebarNavItem
                        href="/words"
                        icon=icondata::LuLanguages
                        label=words_label
                        is_active=is_words_active
                        test_id=derive_test_id(test_id, "item-words")
                    />
                    <SidebarNavItem
                        href="/grammar"
                        icon=icondata::LuPencilLine
                        label=grammar_label
                        is_active=is_grammar_active
                        test_id=derive_test_id(test_id, "item-grammar")
                    />
                    <SidebarNavItem
                        href="/kanji"
                        icon=icondata::LuBookOpen
                        label=kanji_label
                        is_active=is_kanji_active
                        test_id=derive_test_id(test_id, "item-kanji")
                    />
                    <SidebarNavItem
                        href="/phrases"
                        icon=icondata::LuMessageSquare
                        label=phrases_label
                        is_active=is_phrases_active
                        test_id=derive_test_id(test_id, "item-phrases")
                    />
                    <SidebarNavItem
                        href="/profile"
                        icon=icondata::LuUser
                        label=profile_label
                        is_active=is_profile_active
                        test_id=derive_test_id(test_id, "item-profile")
                    />
                </nav>
                <div class="flex-1"></div>
                <div class="sidebar-footer">
                    <A href="/profile" attr:class="anima-avatar-hover">
                        <Avatar
                            size=Signal::derive(move || AvatarSize::Small)
                            initials=avatar_initials
                            test_id=derive_test_id(test_id, "avatar")
                        />
                    </A>
                </div>
            </aside>
        </Show>
    }
}

#[component]
fn SidebarNavItem(
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
            "sidebar-item active".to_string()
        } else {
            "sidebar-item".to_string()
        }
    });

    view! {
        <A
            href=href
            attr:class=class
            attr:data-testid=test_id_val
            attr:aria-current=move || if is_active.get() { "page" } else { "false" }
        >
            {if use_logo {
                view! { <Logo size=LogoSize::Sm /> }.into_any()
            } else {
                view! { <Icon icon=icon width="20" height="20" /> }.into_any()
            }}
            <span class="sidebar-item-label">{label}</span>
        </A>
    }
}
