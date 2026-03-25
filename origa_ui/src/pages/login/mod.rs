pub mod auth_handlers;
pub mod email_password_form;
pub mod header;
pub mod oauth_buttons;
pub mod oauth_listeners;
mod password_input;
mod validation;

pub use header::LoginHeader;

use crate::store::auth_store::AuthStore;
use crate::ui_components::{
    CardLayout, CardLayoutSize, Divider, DividerVariant, PageLayout, PageLayoutVariant, Text,
    TextSize, TypographyVariant,
};
use email_password_form::EmailPasswordForm;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

#[component]
pub fn Login() -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let navigate = use_navigate();
    let loading = RwSignal::new(false);

    let auth_store_for_effect = auth_store.clone();
    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if auth_store_for_effect.is_authenticated().get() {
                navigate("/home", Default::default());
            }
        }
    });

    let on_email_submit = Callback::new({
        let navigate = navigate.clone();
        move |(email, password): (String, String)| {
            let auth_store = auth_store.clone();
            let navigate = navigate.clone();
            loading.set(true);

            spawn_local(async move {
                let result = auth_store.login(&email, &password).await;

                // Always clear loading state after login attempt completes
                loading.set(false);

                match result {
                    Ok(_) => {
                        navigate("/home", Default::default());
                    }
                    Err(e) => {
                        tracing::error!("Login error: {:?}", e);
                    }
                }
            });
        }
    });

    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8">
                <LoginHeader />
                <div class="space-y-6">
                    <EmailPasswordForm on_submit=on_email_submit />

                    <div class="flex items-center gap-4">
                        <Divider variant=Signal::derive(|| DividerVariant::Single) class=Signal::derive(|| "flex-1".to_string()) />
                        <Text size=TextSize::Small variant=TypographyVariant::Muted class="whitespace-nowrap">
                            "или войти/зарегистрироваться через"
                        </Text>
                        <Divider variant=Signal::derive(|| DividerVariant::Single) class=Signal::derive(|| "flex-1".to_string()) />
                    </div>

                    <oauth_buttons::OAuthButtons />
                </div>
            </CardLayout>
        </PageLayout>
    }
}
