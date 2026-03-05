use super::{ActionButtons, IntegrationsCard, PersonalDataCard, SettingsCard};
use crate::app::{AuthContext, update_current_user};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::{UpdateUserProfileUseCase, UserRepository};
use origa::domain::NativeLanguage;

#[component]
pub fn ProfileContent() -> impl IntoView {
    let ctx = use_context::<AuthContext>().expect("AuthContext not provided");
    let current_user = ctx.current_user;
    let client = ctx.client.clone();

    let user_name = Memo::new(move |_| {
        current_user.with(|u| {
            u.as_ref()
                .map(|u| u.username().to_string())
                .unwrap_or_default()
        })
    });
    let native_language = Memo::new(move |_| {
        current_user.with(|u| {
            u.as_ref()
                .map(|u| u.native_language().clone())
                .unwrap_or(NativeLanguage::Russian)
        })
    });
    let reminders_enabled = Memo::new(move |_| {
        current_user.with(|u| u.as_ref().map(|u| u.reminders_enabled()).unwrap_or(true))
    });

    let selected_language = RwSignal::new(native_language.get_untracked());
    let reminders = RwSignal::new(reminders_enabled.get_untracked());

    Effect::new(move |_| {
        selected_language.set(native_language.get());
    });
    Effect::new(move |_| {
        reminders.set(reminders_enabled.get());
    });

    let is_saving = RwSignal::new(false);
    let is_deleting = RwSignal::new(false);
    let is_logging_out = RwSignal::new(false);
    let repository = ctx.repository.clone();

    let save_profile = Callback::new(move |_| {
        let user_id = current_user.with(|u| u.as_ref().map(|u| u.id())).unwrap();
        let repository = repository.clone();
        let current_user_signal = current_user;
        let language = selected_language.get();
        let reminders_enabled = reminders.get();
        let is_saving_signal = is_saving;

        is_saving_signal.set(true);

        spawn_local(async move {
            let use_case = UpdateUserProfileUseCase::new(&repository);

            let result = use_case
                .execute(
                    user_id,
                    language,
                    None,
                    reminders_enabled,
                )
                .await;

            is_saving_signal.set(false);

            if result.is_ok() {
                update_current_user(repository, current_user_signal);
            }
        });
    });

    let client_for_logout = client.clone();
    let navigate = leptos_router::hooks::use_navigate();
    let navigate_for_logout = navigate.clone();
    let navigate_for_delete = navigate.clone();

    let logout = Callback::new(move |_| {
        let client_clone = client_for_logout.clone();
        let current_user_clone = current_user;
        let nav = navigate_for_logout.clone();
        let is_logging_out_signal = is_logging_out;

        is_logging_out_signal.set(true);

        spawn_local(async move {
            let _ = client_clone.logout().await;
            current_user_clone.set(None);
            nav("/", Default::default());
        });
    });

    let client_for_delete = client.clone();
    let repository_for_delete = ctx.repository.clone();
    let delete_account = Callback::new(move |_| {
        let client_clone = client_for_delete.clone();
        let repository_clone = repository_for_delete.clone();
        let current_user_clone = current_user;
        let is_deleting_signal = is_deleting;
        let nav = navigate_for_delete.clone();

        is_deleting_signal.set(true);

        spawn_local(async move {
            let user_id = current_user_clone.with(|u| u.as_ref().map(|u| u.id()));

            match client_clone.delete_account().await {
                Ok(()) => {
                    if let Some(uid) = user_id
                        && let Err(e) = repository_clone.delete(uid).await
                    {
                        web_sys::console::error_1(
                            &format!("Failed to delete local data: {}", e).into(),
                        );
                    }
                    current_user_clone.set(None);
                    nav("/", Default::default());
                }
                Err(e) => {
                    is_deleting_signal.set(false);
                    web_sys::console::error_1(&format!("Failed to delete account: {}", e).into());
                }
            }
        });
    });

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <PersonalDataCard
                user_name={user_name}
                selected_language={selected_language}
            />

            <div class="space-y-4">
                <IntegrationsCard />
                <SettingsCard reminders={reminders} />
                <ActionButtons
                    on_save={save_profile}
                    on_logout={logout}
                    on_delete_account={delete_account}
                    is_saving={Signal::derive(move || is_saving.get())}
                    is_deleting={Signal::derive(move || is_deleting.get())}
                    is_logging_out={Signal::derive(move || is_logging_out.get())}
                />
            </div>
        </div>
    }
}
