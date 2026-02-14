pub mod form;
pub mod header;

pub use form::LoginForm;
pub use header::LoginHeader;

use leptos::prelude::*;
use origa::domain::User;

#[component]
pub fn Login() -> impl IntoView {
    let username = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let current_user = RwSignal::new(None::<User>);
    provide_context(current_user);

    view! {
        <div class="min-h-screen flex items-center justify-center px-4">
            <div class="max-w-md w-full">
                <LoginHeader />
                <LoginForm
                    username=username
                    error=error
                />
            </div>
        </div>
    }
}
