use super::{LabeledInput, LanguageSelector, LevelSelector};
use crate::ui_components::{Card, Heading, HeadingLevel, Text, TextSize};
use leptos::prelude::*;
use origa::domain::{JapaneseLevel, NativeLanguage};

#[component]
pub fn PersonalDataCard(
    user_name: impl Fn() -> String + 'static,
    selected_level: RwSignal<JapaneseLevel>,
    selected_language: RwSignal<NativeLanguage>,
) -> impl IntoView {
    let user_name_signal = RwSignal::new(user_name());

    Effect::new(move |_| {
        user_name_signal.set(user_name());
    });

    view! {
        <Card>
            <div class="space-y-6">
                <Heading level={HeadingLevel::H2}>
                    "Личные данные"
                </Heading>

                <div class="space-y-4">
                    <LabeledInput
                        label="Имя пользователя".to_string()
                        value={user_name_signal}
                        disabled={true}
                    />

                    <div>
                        <Text size={TextSize::Large}>
                            "Целевой уровень JLPT"
                        </Text>
                        <LevelSelector selected_level={selected_level} />
                    </div>

                    <div>
                        <Text size={TextSize::Large}>
                            "Язык интерфейса"
                        </Text>
                        <LanguageSelector selected_language={selected_language} />
                    </div>
                </div>
            </div>
        </Card>
    }
}
