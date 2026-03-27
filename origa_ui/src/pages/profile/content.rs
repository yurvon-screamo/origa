use super::{ActionButtons, PersonalDataCard, SettingsCard};
use crate::store::AuthStore;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use origa::domain::{NativeLanguage, User};
use origa::use_cases::UpdateUserProfileUseCase;

#[component]
pub fn ProfileContent() -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");

    let user_name = Memo::new(move |_| {
        auth_store.user.with(|u: &Option<User>| {
            u.as_ref()
                .map(|u| u.username().to_string())
                .unwrap_or_default()
        })
    });

    let native_language = Memo::new(move |_| {
        auth_store.user.with(|u: &Option<User>| {
            u.as_ref()
                .map(|u| *u.native_language())
                .unwrap_or(NativeLanguage::Russian)
        })
    });

    let selected_language = RwSignal::new(native_language.get_untracked());

    Effect::new(move |_| {
        selected_language.set(native_language.get());
    });

    let is_saving = RwSignal::new(false);
    let is_logging_out = RwSignal::new(false);
    let is_deleting = RwSignal::new(false);
    let disposed = StoredValue::new(());

    let auth_store_for_save = auth_store.clone();
    let save_profile = Callback::new(move |_| {
        let repository = auth_store_for_save.repository().clone();
        let language = selected_language.get();
        let is_saving_signal = is_saving;
        let auth_store_clone = auth_store_for_save.clone();

        is_saving_signal.set(true);

        spawn_local(async move {
            let use_case = UpdateUserProfileUseCase::new(&repository);

            let result = use_case.execute(language, None).await;

            if disposed.is_disposed() {
                return;
            }
            is_saving_signal.set(false);

            if result.is_ok() {
                let _ = auth_store_clone.refresh_user().await;
            }
        });
    });

    let navigate = use_navigate();
    let navigate_for_logout = navigate.clone();
    let navigate_for_delete = navigate.clone();

    let auth_store_for_logout = auth_store.clone();
    let logout = Callback::new(move |_| {
        let nav = navigate_for_logout.clone();
        let auth_store_clone = auth_store_for_logout.clone();
        let is_logging_out_signal = is_logging_out;

        is_logging_out_signal.set(true);

        spawn_local(async move {
            let _ = auth_store_clone.logout().await;
            nav("/", Default::default());
        });
    });

    let auth_store_for_delete = auth_store.clone();
    let delete_account = Callback::new(move |_| {
        let nav = navigate_for_delete.clone();
        let auth_store_clone = auth_store_for_delete.clone();
        let is_deleting_signal = is_deleting;

        is_deleting_signal.set(true);

        spawn_local(async move {
            if auth_store_clone.delete_account().await.is_ok() {
                if disposed.is_disposed() {
                    return;
                }
                nav("/", Default::default());
            } else {
                is_deleting_signal.set(false);
            }
        });
    });

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4" data-testid="profile-content">
            <PersonalDataCard
                user_name={user_name}
                selected_language={selected_language}
                test_id="profile-personal-data"
            />

            <div class="space-y-4">
                <SettingsCard test_id="profile-settings" />
                <ActionButtons
                    on_save={save_profile}
                    on_logout={logout}
                    on_delete_account={delete_account}
                    is_saving={Signal::derive(move || is_saving.get())}
                    is_deleting={Signal::derive(move || is_deleting.get())}
                    is_logging_out={Signal::derive(move || is_logging_out.get())}
                    test_id="profile-actions"
                />
            </div>
        </div>
    }
}
