pub mod email_input;
pub mod error_message;
pub mod form;
pub mod header;
pub mod password_input;

pub use form::{EmailConfirmationForm, LoginForm, RegisterForm};
pub use header::LoginHeader;

use crate::app::AuthContext;
use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[derive(Clone, Copy, PartialEq)]
pub enum LoginMode {
    Login,
    Register,
    EmailNotConfirmed,
}

#[component]
pub fn Login() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let mode = RwSignal::new(LoginMode::Login);
    let auth_ctx = use_context::<AuthContext>().expect("AuthContext not provided");
    let navigate = use_navigate();

    Effect::new(move |_| {
        if auth_ctx.current_user.get().is_some() {
            navigate("/home", Default::default());
        }
    });

    view! {
        <PageLayout variant=PageLayoutVariant::Centered>
            <CardLayout size=CardLayoutSize::Medium>
                <LoginHeader />
                {move || match mode.get() {
                    LoginMode::Login => view! {
                        <LoginForm
                            email=email
                            password=password
                            error=error
                            mode=mode
                        />
                    }.into_any(),
                    LoginMode::Register => view! {
                        <RegisterForm
                            email=email
                            password=password
                            error=error
                            mode=mode
                        />
                    }.into_any(),
                    LoginMode::EmailNotConfirmed => view! {
                        <EmailConfirmationForm
                            email=email
                            error=error
                            mode=mode
                        />
                    }.into_any(),
                }}
            </CardLayout>
        </PageLayout>
    }
}
