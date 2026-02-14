use crate::repository::InMemoryUserRepository;
use crate::ui_components::{
    Button, ButtonVariant, Card, Heading, HeadingLevel, Input, Text, TextSize, Toggle,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::application::use_cases::{GetUserInfoUseCase, UpdateUserProfileUseCase};
use origa::domain::{JapaneseLevel, NativeLanguage, User};

#[component]
pub fn ProfileContent() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");
    let repository =
        use_context::<InMemoryUserRepository>().expect("repository context not provided");

    let user_name = RwSignal::new(current_user.with(|u| {
        u.as_ref()
            .map(|u| u.username().to_string())
            .unwrap_or_default()
    }));
    let japanese_level = current_user.with(|u| {
        u.as_ref()
            .map(|u| *u.current_japanese_level())
            .unwrap_or(JapaneseLevel::N5)
    });
    let native_language = current_user.with(|u| {
        u.as_ref()
            .map(|u| u.native_language().clone())
            .unwrap_or(NativeLanguage::Russian)
    });
    let reminders_enabled =
        current_user.with(|u| u.as_ref().map(|u| u.reminders_enabled()).unwrap_or(true));
    let duolingo_token = current_user.with(|u| {
        u.as_ref()
            .map(|u| u.duolingo_jwt_token().map(|t| t.to_string()))
            .unwrap_or(None)
    });

    let selected_level = RwSignal::new(japanese_level);
    let selected_language = RwSignal::new(native_language);
    let reminders = RwSignal::new(reminders_enabled);
    let duolingo_input = RwSignal::new(duolingo_token.unwrap_or_default());
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
        <Card>
            <div class="space-y-6">
                <Heading level={HeadingLevel::H2}>
                    "Личные данные"
                </Heading>

                <div class="space-y-4">
                    <div>
                        <Text size={TextSize::Large}>
                            "Имя пользователя"
                        </Text>
                        <Input
                            value={user_name}
                            disabled={true}
                        />
                    </div>

                    <div>
                        <Text size={TextSize::Large}>
                            "Целевой уровень JLPT"
                        </Text>
                        <div class="flex space-x-2 mt-2">
                            {LEVELS.iter().map(|level| {
                                let level_for_select = *level;
                                let level_for_click = *level;
                                let level_for_display = *level;
                                let is_selected = move || selected_level.get() == level_for_select;
                                view! {
                                    <Button
                                        variant={if is_selected() { ButtonVariant::Olive } else { ButtonVariant::Default }}
                                        on_click={Callback::new(move |_| selected_level.set(level_for_click))}
                                    >
                                        {format!("{:?}", level_for_display)}
                                    </Button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>

                    <div>
                        <Text size={TextSize::Large}>
                            "Язык интерфейса"
                        </Text>
                        <div class="flex space-x-2 mt-2">
                            {LANGUAGES.iter().map(|lang| {
                                let lang_for_select = lang.clone();
                                let lang_for_click = lang.clone();
                                let lang_for_display = lang.clone();
                                let is_selected = move || selected_language.get() == lang_for_select;
                                view! {
                                    <Button
                                        variant={if is_selected() { ButtonVariant::Olive } else { ButtonVariant::Default }}
                                        on_click={Callback::new(move |_| selected_language.set(lang_for_click.clone()))}
                                    >
                                        {format!("{:?}", lang_for_display)}
                                    </Button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>
            </div>
        </Card>

        <Card>
            <div class="space-y-4">
                <Heading level={HeadingLevel::H2}>
                    "Интеграции"
                </Heading>

                <div class="space-y-4">
                    <div>
                        <Text size={TextSize::Large}>
                            "Duolingo JWT Token"
                        </Text>
                        <Input
                            value={duolingo_input}
                        />
                    </div>
                </div>
            </div>
        </Card>

        <Card>
            <div class="space-y-4">
                <Heading level={HeadingLevel::H2}>
                    "Настройки приложения"
                </Heading>

                <div class="flex items-center justify-between">
                    <Text size={TextSize::Large}>
                        "Напоминания"
                    </Text>
                    <Toggle
                        checked={Signal::derive(move || reminders.get())}
                        on_change={Callback::new(move |_| {
                            reminders.update(|r| *r = !*r);
                        })}
                    />
                </div>
            </div>
        </Card>

        <div class="flex space-x-4">
            <Button
                variant={ButtonVariant::Filled}
                on_click={save_profile}
                disabled={false}
            >
                {move || if is_saving.get() { "Сохранение..." } else { "Сохранить изменения" }}
            </Button>

            <Button
                variant={ButtonVariant::Ghost}
                on_click={logout}
            >
                "Выйти из аккаунта"
            </Button>
        </div>
    }
}

const LEVELS: &[JapaneseLevel] = &[
    JapaneseLevel::N5,
    JapaneseLevel::N4,
    JapaneseLevel::N3,
    JapaneseLevel::N2,
    JapaneseLevel::N1,
];

const LANGUAGES: &[NativeLanguage] = &[NativeLanguage::Russian, NativeLanguage::English];
