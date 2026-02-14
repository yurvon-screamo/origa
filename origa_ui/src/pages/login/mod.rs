pub mod form;
pub mod header;

pub use form::LoginForm;
pub use header::LoginHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Login() -> impl IntoView {
    let username = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);

    view! {
        <PageLayout variant=PageLayoutVariant::Centered>
            <CardLayout size=CardLayoutSize::Medium>
                <LoginHeader />
                <LoginForm
                    username=username
                    error=error
                />
            </CardLayout>
        </PageLayout>
    }
}
