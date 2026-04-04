use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::use_cases::MarkCardAsKnownUseCase;
use ulid::Ulid;

pub fn create_mark_as_known_callback(
    repository: HybridUserRepository,
    refresh_trigger: RwSignal<u32>,
) -> Callback<Ulid> {
    Callback::new(move |card_id: Ulid| {
        let repo = repository.clone();
        let refresh = refresh_trigger;
        spawn_local(async move {
            let use_case = MarkCardAsKnownUseCase::new(&repo);
            if use_case.execute(card_id).await.is_ok() {
                refresh.update(|t| *t += 1);
            }
        });
    })
}
