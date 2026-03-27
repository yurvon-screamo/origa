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
    let server_error = RwSignal::new(None::<String>);
    let disposed = StoredValue::new(());

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
            server_error.set(None);

            spawn_local(async move {
                let result = auth_store.login(&email, &password).await;

                if disposed.is_disposed() {
                    return;
                }

                loading.set(false);

                match result {
                    Ok(_) => {
                        navigate("/home", Default::default());
                    },
                    Err(e) => {
                        tracing::error!("Login error: {:?}", e);
                        server_error.set(Some(e.to_string()));
                    },
                }
            });
        }
    });

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id=Signal::derive(|| "login-page".to_string())>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8" test_id=Signal::derive(|| "login-card".to_string())>
                <LoginHeader />
                <div class="space-y-6">
                    <EmailPasswordForm
                        on_submit=on_email_submit
                        server_error=server_error
                        test_id=Signal::derive(|| "login-form".to_string())
                    />

                    <div class="flex items-center gap-4">
                        <Divider variant=Signal::derive(|| DividerVariant::Single) class=Signal::derive(|| "flex-1".to_string()) test_id=Signal::derive(|| "login-divider-left".to_string()) />
                        <Text size=TextSize::Small variant=TypographyVariant::Muted class="whitespace-nowrap" test_id=Signal::derive(|| "login-divider-text".to_string())>
                            "или войти/зарегистрироваться через"
                        </Text>
                        <Divider variant=Signal::derive(|| DividerVariant::Single) class=Signal::derive(|| "flex-1".to_string()) test_id=Signal::derive(|| "login-divider-right".to_string()) />
                    </div>

                    <oauth_buttons::OAuthButtons />
                </div>
            </CardLayout>
        </PageLayout>
    }
}
