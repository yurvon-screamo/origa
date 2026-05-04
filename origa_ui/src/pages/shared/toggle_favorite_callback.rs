use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;
use origa::use_cases::ToggleFavoriteUseCase;
use ulid::Ulid;

pub fn create_toggle_favorite_callback(
    repository: HybridUserRepository,
    current_user: RwSignal<Option<User>>,
    refresh_trigger: RwSignal<u32>,
) -> Callback<Ulid> {
    Callback::new(move |card_id: Ulid| {
        let repository = repository.clone();
        spawn_local(async move {
            let use_case = ToggleFavoriteUseCase::new(&repository);
            if use_case.execute(card_id).await.is_ok() {
                current_user.update(|u| {
                    if let Some(user) = u {
                        let _ = user.toggle_favorite(card_id);
                    }
                });
                refresh_trigger.update(|v| *v += 1);
            }
        });
    })
}
