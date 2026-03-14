use crate::repository::HybridUserRepository;
use crate::ui_components::{ToastData, ToastType};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::use_cases::DeleteCardUseCase;
use ulid::Ulid;

pub fn create_delete_callback(
    repository: HybridUserRepository,
    toasts: RwSignal<Vec<ToastData>>,
) -> (RwSignal<bool>, Callback<Ulid>) {
    let is_deleting = RwSignal::new(false);
    let callback = Callback::new(move |card_id: Ulid| {
        let repo = repository.clone();
        let toasts_clone = toasts;
        let is_deleting_clone = is_deleting;

        is_deleting_clone.set(true);
        spawn_local(async move {
            let use_case = DeleteCardUseCase::new(&repo);
            if let Err(e) = use_case.execute(card_id).await {
                toasts_clone.update(|t| {
                    t.push(ToastData {
                        id: t.len(),
                        toast_type: ToastType::Error,
                        title: "Ошибка удаления".to_string(),
                        message: e.to_string(),
                        duration_ms: None,
                    });
                });
            }
            is_deleting_clone.set(false);
        });
    });
    (is_deleting, callback)
}
