mod action_buttons;
mod content;
mod header;
mod integrations_card;
mod labeled_input;
mod language_selector;
mod personal_data_card;
mod settings_card;

pub use action_buttons::ActionButtons;
pub use content::ProfileContent;
pub use header::ProfileHeader;
pub use integrations_card::IntegrationsCard;
pub use labeled_input::LabeledInput;
pub use language_selector::LanguageSelector;
pub use personal_data_card::PersonalDataCard;
pub use settings_card::SettingsCard;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Profile() -> impl IntoView {
    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8">
                <ProfileHeader />
                <ProfileContent />
            </CardLayout>
        </PageLayout>
    }
}
