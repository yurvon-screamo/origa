use crate::repository::HybridUserRepository;
use crate::ui_components::{ToastData, ToastType};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;
use origa::use_cases::DeleteCardUseCase;
use ulid::Ulid;

pub fn create_delete_callback(
    repository: HybridUserRepository,
    current_user: RwSignal<Option<User>>,
    toasts: RwSignal<Vec<ToastData>>,
) -> (RwSignal<bool>, Callback<Ulid>) {
    let is_deleting = RwSignal::new(false);
    let callback = Callback::new(move |card_id: Ulid| {
        let user = current_user.get();
        let repo = repository.clone();
        let current_user_clone = current_user;
        let toasts_clone = toasts;
        let is_deleting_clone = is_deleting;

        if let Some(user) = user {
            let user_id = user.id();
            is_deleting_clone.set(true);
            spawn_local(async move {
                let use_case = DeleteCardUseCase::new(&repo);
                match use_case.execute(user_id, card_id).await {
                    Ok(()) => {
                        current_user_clone.update(|u| {
                            if let Some(user) = u {
                                let _ = user.delete_card(card_id);
                            }
                        });
                    }
                    Err(e) => {
                        toasts_clone.update(|t| {
                            t.push(ToastData {
                                id: t.len(),
                                toast_type: ToastType::Error,
                                title: "Ошибка удаления".to_string(),
                                message: e.to_string(),
                            });
                        });
                    }
                }
                is_deleting_clone.set(false);
            });
        }
    });
    (is_deleting, callback)
}