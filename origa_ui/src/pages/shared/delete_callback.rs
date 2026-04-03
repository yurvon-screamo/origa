use crate::repository::HybridUserRepository;
use crate::ui_components::{ToastData, ToastType};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::use_cases::DeleteCardUseCase;
use ulid::Ulid;

pub struct DeleteRequest {
    pub card_id: Ulid,
    pub on_success: Callback<()>,
}

pub fn create_delete_callback(
    repository: HybridUserRepository,
    toasts: RwSignal<Vec<ToastData>>,
    refresh_trigger: RwSignal<u32>,
) -> (RwSignal<bool>, Callback<DeleteRequest>) {
    let is_deleting = RwSignal::new(false);
    let callback = Callback::new(move |request: DeleteRequest| {
        let disposed = StoredValue::new(());
        let repo = repository.clone();
        let toasts_clone = toasts;
        let is_deleting_clone = is_deleting;
        let on_success = request.on_success;

        is_deleting_clone.set(true);
        spawn_local(async move {
            let use_case = DeleteCardUseCase::new(&repo);
            match use_case.execute(request.card_id).await {
                Ok(()) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    refresh_trigger.update(|v| *v += 1);
                    on_success.run(())
                },
                Err(e) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    toasts_clone.update(|t| {
                        t.push(ToastData {
                            id: t.len(),
                            toast_type: ToastType::Error,
                            title: "Ошибка удаления".to_string(),
                            message: e.to_string(),
                            duration_ms: None,
                            closable: true,
                        });
                    });
                },
            }
            if disposed.is_disposed() {
                return;
            }
            is_deleting_clone.set(false);
        });
    });
    (is_deleting, callback)
}
