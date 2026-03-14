mod action_buttons;
mod content;
mod header;
mod labeled_input;
mod language_selector;
mod personal_data_card;
mod settings_card;

pub use action_buttons::ActionButtons;
pub use content::ProfileContent;
pub use header::ProfileHeader;
pub use labeled_input::LabeledInput;
pub use language_selector::LanguageSelector;
pub use personal_data_card::PersonalDataCard;
pub use settings_card::SettingsCard;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;
use origa::domain::User;

#[component]
pub fn Profile() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");

    let username = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.username().to_string())
            .unwrap_or_default()
    });

    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8">
                <ProfileHeader username=username.get() />
                <ProfileContent />
            </CardLayout>
        </PageLayout>
    }
}
