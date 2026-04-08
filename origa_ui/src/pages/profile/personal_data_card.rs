use super::{LabeledInput, LanguageSelector};
use crate::i18n::*;
use crate::pages::shared::DailyLoadSelector;
use crate::ui_components::{Card, Heading, HeadingLevel, Text, TextSize};
use leptos::prelude::*;
use origa::domain::{DailyLoad, NativeLanguage};

#[component]
pub fn PersonalDataCard(
    #[prop(optional, into)] test_id: Signal<String>,
    user_name: Memo<String>,
    selected_language: RwSignal<NativeLanguage>,
    selected_daily_load: RwSignal<DailyLoad>,
) -> impl IntoView {
    let i18n = use_i18n();
    let user_name_signal = RwSignal::new(user_name.get_untracked());

    Effect::new(move |_| {
        user_name_signal.set(user_name.get());
    });

    view! {
        <Card test_id=test_id>
            <div class="space-y-6">
                <Heading level={HeadingLevel::H2}>
                    {t!(i18n, profile.personal_data)}
                </Heading>

                <div class="space-y-4">
                    <LabeledInput
                        label=td_string!(i18n.get_locale(), profile.username).to_string()
                        value={user_name_signal}
                        disabled={true}
                    />

                    <div>
                        <Text size={TextSize::Large}>
                            {t!(i18n, profile.interface_language)}
                        </Text>
                        <LanguageSelector selected_language={selected_language} />
                    </div>

                    <div>
                        <Text size={TextSize::Large}>
                            {t!(i18n, profile.learning_pace)}
                        </Text>
                        <DailyLoadSelector selected_load={selected_daily_load} />
                    </div>
                </div>
            </div>
        </Card>
    }
}
