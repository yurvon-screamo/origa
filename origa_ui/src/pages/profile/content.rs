use super::{ActionButtons, PasswordCard, PersonalDataCard, SettingsCard};
use crate::i18n::{native_language_to_locale, t, use_i18n};
use crate::store::AuthStore;
use crate::ui_components::{Avatar, AvatarSize, Card, OfflineBundleCard};
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

    let i18n = use_i18n();
    let i18n_for_sync = i18n;
    Effect::new(move |_| {
        let lang = selected_language.get();
        i18n_for_sync.set_locale(native_language_to_locale(&lang));
    });

    let daily_load = Memo::new(move |_| {
        auth_store
            .user
            .with(|u: &Option<User>| u.as_ref().map(|u| *u.daily_load()).unwrap_or_default())
    });

    let selected_daily_load = RwSignal::new(daily_load.get_untracked());

    Effect::new(move |_| {
        selected_daily_load.set(daily_load.get());
    });

    let is_saving = RwSignal::new(false);
    let is_logging_out = RwSignal::new(false);
    let is_deleting = RwSignal::new(false);
    let disposed = StoredValue::new(());

    let auth_store_for_save = auth_store.clone();
    let save_profile = Callback::new(move |_| {
        let repository = auth_store_for_save.repository().clone();
        let language = selected_language.get();
        let daily_load_val = selected_daily_load.get();
        let is_saving_signal = is_saving;
        let auth_store_clone = auth_store_for_save.clone();

        is_saving_signal.set(true);

        spawn_local(async move {
            let use_case = UpdateUserProfileUseCase::new(&repository);

            let result = use_case.execute(language, daily_load_val, None).await;

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
            nav("/login", Default::default());
        });
    });

    let auth_store_for_delete = auth_store.clone();
    let delete_account = Callback::new(move |_| {
        let nav = navigate_for_delete.clone();
        let auth_store_clone = auth_store_for_delete.clone();

        is_deleting.set(true);

        spawn_local(async move {
            let _ = auth_store_clone.delete_account().await;
            if disposed.is_disposed() {
                return;
            }
            nav("/login", Default::default());
        });
    });

    view! {
        <div class="profile-layout" data-testid="profile-content">
            <div class="profile-identity">
                <Avatar
                    size=Signal::derive(move || AvatarSize::Large)
                    initials=Signal::derive(move || {
                        let name = user_name.get();
                        name.chars().take(2).collect::<String>().to_uppercase()
                    })
                />
                <div>
                    <div class="profile-identity-name">{move || user_name.get()}</div>
                    <span class="label-muted">{t!(i18n, home.profile)}</span>
                </div>
            </div>

            <div class="profile-grid">
                <div class="profile-col">
                    <Card shadow=Signal::derive(|| true)>
                        <PersonalDataCard
                            selected_language={selected_language}
                            selected_daily_load={selected_daily_load}
                            test_id="profile-personal-data"
                        />
                    </Card>
                    <Card shadow=Signal::derive(|| true)>
                        <OfflineBundleCard test_id="profile-offline-bundle" />
                    </Card>
                </div>

                <div class="profile-col">
                    <Card shadow=Signal::derive(|| true)>
                        <PasswordCard test_id="profile-password" />
                    </Card>
                    <Card>
                        <SettingsCard test_id="profile-settings" />
                    </Card>
                </div>
            </div>

            <div class="profile-actions-bar">
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
