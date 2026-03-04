pub mod auth_handlers;
pub mod email_input;
pub mod error_message;
pub mod header;
pub mod oauth_buttons;

pub use header::LoginHeader;

use crate::app::AuthContext;
use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn Login() -> impl IntoView {
    let auth_ctx = use_context::<AuthContext>().expect("AuthContext not provided");
    let navigate = use_navigate();

    Effect::new(move |_| {
        if auth_ctx.current_user.get().is_some() {
            navigate("/home", Default::default());
        }
    });

    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8">
                <LoginHeader />
                <div class="space-y-6">
                    <oauth_buttons::OAuthButtons />
                </div>
            </CardLayout>
        </PageLayout>
    }
}
