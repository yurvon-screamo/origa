use crate::ui_components::{
    Button, ButtonVariant, Card, Heading, HeadingLevel, Input, Text, TextSize, Toggle,
};
use leptos::prelude::*;
use origa::domain::{JapaneseLevel, NativeLanguage, User};

#[component]
pub fn ProfileContent() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");

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

    let selected_level = RwSignal::new(japanese_level);
    let selected_language = RwSignal::new(native_language);
    let reminders = RwSignal::new(reminders_enabled);

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
                            "Duolingo"
                        </Text>
                        <Button variant={ButtonVariant::Default}>
                            "Подключить аккаунт"
                        </Button>
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
                    <Toggle checked={Signal::derive(move || reminders.get())} />
                </div>
            </div>
        </Card>

        <Button
            variant={ButtonVariant::Filled}
            on_click={Callback::new(move |_| {
                current_user.set(None);
            })}
        >
            "Выйти из аккаунта"
        </Button>
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
