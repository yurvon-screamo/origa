use crate::demo_data::create_demo_user;
use crate::repository::InMemoryUserRepository;
use crate::routes::AppRoutes;
use leptos::prelude::*;
use origa::domain::User;

#[component]
pub fn App() -> impl IntoView {
    provide_context(InMemoryUserRepository::new());
    let demo_user = create_demo_user();
    let current_user = RwSignal::new(Some(demo_user));
    provide_context(current_user);
    view! {
        <AppRoutes />
    }
}
