use crate::repository::InMemoryUserRepository;
use crate::routes::AppRoutes;
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    provide_context(InMemoryUserRepository::new());
    view! {
        <AppRoutes />
    }
}
