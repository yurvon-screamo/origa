use crate::repository::InMemoryUserRepository;
use crate::routes::AppRoutes;
use leptos::prelude::*;
use origa::domain::User;

#[component]
pub fn App() -> impl IntoView {
    provide_context(InMemoryUserRepository::new());
    let current_user = RwSignal::new(None::<User>);
    provide_context(current_user);
    view! {
        <AppRoutes />
    }
}
