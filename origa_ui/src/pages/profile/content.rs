use super::{DangerZoneCard, PasswordCard, PersonalDataCard, SettingsCard};
use crate::i18n::{native_language_to_locale, t, use_i18n};
use crate::store::AuthStore;
use crate::ui_components::{Card, OfflineBundleCard};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use origa::domain::{DailyLoad, NativeLanguage, User};
use origa::use_cases::UpdateUserProfileUseCase;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum AutoSaveStatus {
    #[default]
    Idle,
    Saving,
    Saved,
    Error,
}

const AUTOSAVE_STATUS_DISPLAY_MS: u32 = 1500;

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

    let save_status: RwSignal<AutoSaveStatus> = RwSignal::new(AutoSaveStatus::Idle);
    let is_logging_out = RwSignal::new(false);
    let is_deleting = RwSignal::new(false);
    let disposed = StoredValue::new(());
    let is_saving = RwSignal::new(false);
    let needs_resave = RwSignal::new(false);

    let auth_store_save = auth_store.clone();
    let trigger_save = Callback::new(move |_: ()| {
        if is_saving.get() {
            needs_resave.set(true);
            return;
        }

        is_saving.set(true);
        save_status.set(AutoSaveStatus::Saving);

        let repository = auth_store_save.repository().clone();
        let auth_store_clone = auth_store_save.clone();

        spawn_local(async move {
            loop {
                let language = selected_language.get();
                let daily_load_val = selected_daily_load.get();

                let use_case = UpdateUserProfileUseCase::new(&repository);
                let result = use_case.execute(language, daily_load_val, None).await;

                if disposed.is_disposed() {
                    return;
                }

                if result.is_err() {
                    is_saving.set(false);
                    needs_resave.set(false);
                    save_status.set(AutoSaveStatus::Error);
                    return;
                }

                let _ = auth_store_clone.refresh_user().await;

                if disposed.is_disposed() {
                    return;
                }

                if needs_resave.get() {
                    needs_resave.set(false);
                    save_status.set(AutoSaveStatus::Saving);
                    continue;
                }

                break;
            }

            is_saving.set(false);
            save_status.set(AutoSaveStatus::Saved);
            gloo_timers::future::TimeoutFuture::new(AUTOSAVE_STATUS_DISPLAY_MS).await;
            if disposed.is_disposed() {
                return;
            }
            save_status.set(AutoSaveStatus::Idle);
        });
    });

    let on_language_change = {
        let trigger = trigger_save;
        Callback::new(move |_lang: NativeLanguage| {
            trigger.run(());
        })
    };

    let on_daily_load_change = {
        let trigger = trigger_save;
        Callback::new(move |_load: DailyLoad| {
            trigger.run(());
        })
    };

    let on_retry = trigger_save;

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
            <div class="profile-header">
                <div class="profile-header-name">{move || user_name.get()}</div>
                <div class="profile-header-label">
                    <span class="label-muted">{t!(i18n, home.profile)}</span>
                </div>
            </div>

            <div class="profile-grid">
                <div class="profile-col">
                    <Card shadow=Signal::derive(|| true)>
                        <PersonalDataCard
                            selected_language={selected_language}
                            selected_daily_load={selected_daily_load}
                            save_status={Signal::derive(move || save_status.get())}
                            on_language_change={on_language_change}
                            on_daily_load_change={on_daily_load_change}
                            on_retry={on_retry}
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
                    <DangerZoneCard
                        on_logout={logout}
                        on_delete_account={delete_account}
                        is_logging_out={Signal::derive(move || is_logging_out.get())}
                        is_deleting={Signal::derive(move || is_deleting.get())}
                        test_id="profile-danger-zone"
                    />
                </div>
            </div>
        </div>
    }
}
