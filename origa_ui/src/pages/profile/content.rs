use super::{ActionButtons, IntegrationsCard, PersonalDataCard, SettingsCard};
use crate::repository::InMemoryUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::use_cases::{GetUserInfoUseCase, UpdateUserProfileUseCase};
use origa::domain::User;
use origa::domain::{JapaneseLevel, NativeLanguage};

#[component]
pub fn ProfileContent() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");

    let repository =
        use_context::<InMemoryUserRepository>().expect("repository context not provided");

    let user_name = Memo::new(move |_| {
        current_user.with(|u| {
            u.as_ref()
                .map(|u| u.username().to_string())
                .unwrap_or_default()
        })
    });
    let japanese_level = Memo::new(move |_| {
        current_user.with(|u| {
            u.as_ref()
                .map(|u| *u.current_japanese_level())
                .unwrap_or(JapaneseLevel::N5)
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
    let duolingo_token = Memo::new(move |_| {
        current_user.with(|u| {
            u.as_ref()
                .map(|u| u.duolingo_jwt_token().map(|t| t.to_string()))
                .unwrap_or(None)
        })
    });

    let selected_level = RwSignal::new(japanese_level.get_untracked());
    let selected_language = RwSignal::new(native_language.get_untracked());
    let reminders = RwSignal::new(reminders_enabled.get_untracked());
    let duolingo_input = RwSignal::new(duolingo_token.get_untracked().unwrap_or_default());

    let _ = Memo::new(move |_| {
        duolingo_input.set(duolingo_token.get().unwrap_or_default());
    });
    let is_saving = RwSignal::new(false);

    let save_profile = Callback::new(move |_| {
        let user_id = current_user.with(|u| u.as_ref().map(|u| u.id())).unwrap();
        let repository = repository.clone();
        let current_user = current_user.clone();
        let level = selected_level.get();
        let language = selected_language.get();
        let reminders_enabled = reminders.get();
        let token = duolingo_input.get();
        let is_saving = is_saving.clone();

        is_saving.set(true);

        spawn_local(async move {
            let use_case = UpdateUserProfileUseCase::new(&repository);

            let result = use_case
                .execute(
                    user_id,
                    level,
                    language,
                    if token.is_empty() { None } else { Some(token) },
                    None,
                    reminders_enabled,
                )
                .await;

            is_saving.set(false);

            if let Ok(_) = result {
                let get_use_case = GetUserInfoUseCase::new(&repository);
                if let Ok(profile) = get_use_case.execute(user_id).await {
                    current_user.update(|u| {
                        if let Some(user) = u {
                            user.set_current_japanese_level(profile.current_japanese_level);
                            user.set_native_language(profile.native_language.clone());
                            user.set_reminders_enabled(profile.reminders_enabled);
                            user.set_duolingo_jwt_token(profile.duolingo_jwt_token);
                        }
                    });
                }
            }
        });
    });

    let logout = Callback::new(move |_| {
        current_user.set(None);
        let navigate = leptos_router::hooks::use_navigate();
        navigate("/", Default::default());
    });

    view! {
        <div class="space-y-4">
            <PersonalDataCard
                user_name={move || user_name.get()}
                selected_level={selected_level}
                selected_language={selected_language}
            />

            <IntegrationsCard duolingo_input={duolingo_input} />

            <SettingsCard reminders={reminders} />

            <ActionButtons
                on_save={save_profile}
                on_logout={logout}
                is_saving={Signal::derive(move || is_saving.get())}
            />
        </div>
    }
}
