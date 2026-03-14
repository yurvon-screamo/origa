pub mod auth_handlers;
pub mod email_password_form;
pub mod header;
pub mod oauth_buttons;
mod password_input;
mod validation;

pub use header::LoginHeader;

use crate::app::AuthContext;
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
    let auth_ctx = use_context::<AuthContext>().expect("AuthContext not provided");
    let navigate = use_navigate();
    let loading = RwSignal::new(false);

    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if auth_ctx.is_authenticated.get() {
                navigate("/home", Default::default());
            }
        }
    });

    let on_email_submit = Callback::new({
        let navigate = navigate.clone();
        move |(email, password): (String, String)| {
            let auth_ctx = auth_ctx.clone();
            let navigate = navigate.clone();
            loading.set(true);

            spawn_local(async move {
                let result =
                    auth_handlers::handle_email_password_login(&auth_ctx, &email, &password).await;

                loading.set(false);

                match result {
                    Ok(_) => {
                        auth_ctx.is_authenticated.set(true);
                        navigate("/home", Default::default());
                    }
                    Err(e) => {
                        tracing::error!("Login error: {}", e);
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
