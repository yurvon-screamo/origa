use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::use_cases::MarkCardAsKnownUseCase;
use ulid::Ulid;

pub fn create_mark_as_known_callback(
    repository: HybridUserRepository,
    refresh_trigger: RwSignal<u32>,
) -> (Callback<Ulid>, RwSignal<bool>) {
    let is_pending = RwSignal::new(false);
    let callback = Callback::new(move |card_id: Ulid| {
        let repo = repository.clone();
        let refresh = refresh_trigger;
        let pending = is_pending;
        spawn_local(async move {
            pending.set(true);
            let use_case = MarkCardAsKnownUseCase::new(&repo);
            let result = use_case.execute(card_id).await;
            pending.set(false);
            if result.is_ok() {
                refresh.update(|t| *t += 1);
            }
        });
    });
    (callback, is_pending)
}
