use crate::i18n::Locale;
use crate::repository::HybridUserRepository;
use crate::ui_components::{ToastData, ToastType};
use leptos::prelude::*;
use leptos_i18n::I18nContext;
use origa::domain::{OrigaError, User};
use origa::traits::UserRepository;

const SYNC_TOAST_ID: usize = usize::MAX;

pub fn show_sync_toast(toasts: RwSignal<Vec<ToastData>>, i18n: I18nContext<Locale>) {
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
}

pub fn show_sync_success_toast(toasts: RwSignal<Vec<ToastData>>, i18n: I18nContext<Locale>) {
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
}

pub fn show_sync_error_toast(
    toasts: RwSignal<Vec<ToastData>>,
    i18n: I18nContext<Locale>,
    error: &OrigaError,
) {
    toasts.update(|t| t.retain(|toast| toast.id != SYNC_TOAST_ID));
    toasts.update(|t| {
        t.push(ToastData {
            id: t.len(),
            toast_type: ToastType::Error,
            title: i18n.get_keys().home().sync_error().inner().to_string(),
            message: error.to_string(),
            duration_ms: None,
            closable: true,
        });
    });
}

pub async fn run_sync(repo: HybridUserRepository) -> Result<Option<User>, OrigaError> {
    repo.merge_current_user().await?;
    repo.get_current_user().await
}
