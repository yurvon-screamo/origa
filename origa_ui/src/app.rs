use crate::demo_data::create_demo_user;
use crate::repository::InMemoryUserRepository;
use crate::routes::AppRoutes;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::{GetUserInfoUseCase, UserRepository};
use origa::domain::User;
use origa::infrastructure::LlmServiceInvoker;

pub fn update_current_user(
    repository: InMemoryUserRepository,
    current_user: RwSignal<Option<User>>,
) {
    spawn_local(async move {
        if let Some(user) = current_user.get_untracked() {
            let user_id = user.id();
            let use_case = GetUserInfoUseCase::new(&repository);
            if let Ok(_) = use_case.execute(user_id).await {
                if let Ok(updated_user) = repository.find_by_id(user_id).await {
                    current_user.set(updated_user);
                }
            }
        }
    });
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(InMemoryUserRepository::new());
    provide_context(LlmServiceInvoker::None);
    let demo_user = create_demo_user();
    let current_user = RwSignal::new(Some(demo_user));
    provide_context(current_user);
    view! {
        <AppRoutes />
    }
}
