use crate::i18n::Locale;
use crate::repository::{HybridUserRepository, set_last_sync_time};
use crate::ui_components::{ToastData, ToastType};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_i18n::I18nContext;
const SYNC_TOAST_ID: usize = usize::MAX;

pub fn run_sync(
    repo: HybridUserRepository,
    disposed: StoredValue<()>,
    toasts: RwSignal<Vec<ToastData>>,
    i18n: I18nContext<Locale>,
) {
    toasts.update(|t| {
        t.push(ToastData {
            id: SYNC_TOAST_ID,
            toast_type: ToastType::Info,
            title: i18n.get_keys().home().sync().inner().to_string(),
            message: i18n.get_keys().home().sync_data().inner().to_string(),
            duration_ms: None,
            closable: false,
        });
    });

    spawn_local(async move {
        match repo.merge_current_user().await {
            Ok(()) => {
                if disposed.is_disposed() {
                    return;
                }
                toasts.update(|t| t.retain(|toast| toast.id != SYNC_TOAST_ID));
                toasts.update(|t| {
                    t.push(ToastData {
                        id: t.len(),
                        toast_type: ToastType::Success,
                        title: i18n.get_keys().home().sync().inner().to_string(),
                        message: i18n.get_keys().home().sync_success().inner().to_string(),
                        duration_ms: None,
                        closable: true,
                    });
                });
                set_last_sync_time(js_sys::Date::now() as u64 / 1000);
            },
            Err(e) => {
                if disposed.is_disposed() {
                    return;
                }
                toasts.update(|t| t.retain(|toast| toast.id != SYNC_TOAST_ID));
                toasts.update(|t| {
                    t.push(ToastData {
                        id: t.len(),
                        toast_type: ToastType::Error,
                        title: i18n.get_keys().home().sync_error().inner().to_string(),
                        message: e.to_string(),
                        duration_ms: None,
                        closable: true,
                    });
                });
            },
        }
    });
}
