use crate::i18n::*;
use crate::pages::shared::DailyLoadSelector;
use crate::ui_components::NativeLanguageToggle;
use leptos::prelude::*;
use origa::domain::{DailyLoad, NativeLanguage};

#[component]
pub fn PersonalDataCard(
    #[prop(optional, into)] test_id: Signal<String>,
    selected_language: RwSignal<NativeLanguage>,
    selected_daily_load: RwSignal<DailyLoad>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div data-testid=test_id_val class="p-6">
            <div class="flex flex-col gap-4">
                <div class="profile-section">
                    <div class="label-muted">{t!(i18n, profile.interface_language)}</div>
                    <NativeLanguageToggle
                        selected_language={selected_language}
                        test_id=Signal::derive(|| "profile-lang-toggle".to_string())
                    />
                </div>
                <div class="profile-section">
                    <div class="label-muted">{t!(i18n, profile.learning_pace)}</div>
                    <DailyLoadSelector selected_load={selected_daily_load} />
                </div>
            </div>
        </div>
    }
}
