use crate::i18n::use_i18n;
use crate::store::auth_store::AuthStore;
use crate::ui_components::nav_config::NavRoute;
use crate::ui_components::{Logo, LogoSize, derive_test_id};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

/// Routes where the bottom tab bar (and the sidebar) live outside the
/// shell chrome: the lesson and onboarding are full-screen flows.
pub fn is_nav_hidden_path(path: &str) -> bool {
    path == "/lesson" || path == "/onboarding"
}

/// Whether the mobile bottom tab bar is on screen for this shell state.
///
/// Pure predicate — the callers supply the reactive authentication state.
/// This is the SINGLE source of the bar's visibility: `BottomTabBar` hides
/// the `<nav>` by it, and the shell `<main>` reserves the bar's clearance
/// by the same answer (`app-main-with-nav`, #845). A mismatch between the
/// two would put page content under the fixed bar.
pub fn bottom_nav_visible_for(is_authenticated: bool, path: &str) -> bool {
    is_authenticated && !is_nav_hidden_path(path)
}

#[component]
pub fn BottomTabBar(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let location = use_location();

    // One Memo per mount — `is_authenticated()` constructs a fresh Memo on
    // every call, which inside a Signal::derive would mean a new Memo per
    // recompute.
    let is_authenticated = auth_store.is_authenticated();

    let is_visible = Signal::derive(move || {
        bottom_nav_visible_for(is_authenticated.get(), &location.pathname.get())
    });

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <Show when=move || is_visible.get()>
            <nav class="bottom-tab-bar lg:hidden" data-testid=test_id_val>
                <For
                    each=NavRoute::all
                    key=|route| format!("{:?}", route)
                    children=move |route: &NavRoute| {
                        let i18n = use_i18n();
                        let location = use_location();
                        let label = Signal::derive(move || route.label(&i18n));
                        let is_active =
                            Signal::derive(move || route.is_active(&location.pathname.get()));
                        let test_id = derive_test_id(test_id, route.test_id_suffix());

                        view! {
                            <BottomTabItem
                                href=route.href().to_string()
                                icon=route.icon()
                                use_logo=route.use_logo()
                                label=label
                                is_active=is_active
                                test_id=test_id
                            />
                        }
                    }
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
                view! {
                    <span class="bottom-tab-item-logo">
                        <Logo size=LogoSize::Sm />
                    </span>
                }.into_any()
            } else {
                view! { <Icon icon=icon width="26" height="26" /> }.into_any()
            }}
            <span class="bottom-tab-item-label">{label}</span>
        </A>
    }
}

#[cfg(test)]
mod tests {
    use super::{bottom_nav_visible_for, is_nav_hidden_path};

    #[test]
    fn bar_hidden_on_fullscreen_routes() {
        assert!(is_nav_hidden_path("/lesson"));
        assert!(is_nav_hidden_path("/onboarding"));
        assert!(!is_nav_hidden_path("/home"));
        assert!(!is_nav_hidden_path("/words"));
        // Sub-routes are not the hidden paths themselves.
        assert!(!is_nav_hidden_path("/lesson-extra"));
    }

    #[test]
    fn bar_visible_only_when_authenticated_and_not_hidden() {
        // Authenticated on a normal page → the bar is on screen.
        assert!(bottom_nav_visible_for(true, "/home"));
        assert!(bottom_nav_visible_for(true, "/words/ペン"));
        // Fullscreen routes hide the bar even when authenticated.
        assert!(!bottom_nav_visible_for(true, "/lesson"));
        assert!(!bottom_nav_visible_for(true, "/onboarding"));
        // Unauthenticated → never.
        assert!(!bottom_nav_visible_for(false, "/home"));
        assert!(!bottom_nav_visible_for(false, "/lesson"));
    }
}
