mod action_buttons;
mod content;
mod header;
mod labeled_input;
mod language_selector;
mod password_card;
mod personal_data_card;
mod settings_card;

pub use action_buttons::ActionButtons;
pub use content::ProfileContent;
pub use header::ProfileHeader;
pub use labeled_input::LabeledInput;
pub use language_selector::LanguageSelector;
pub use password_card::PasswordCard;
pub use personal_data_card::PersonalDataCard;
pub use settings_card::SettingsCard;

use crate::repository::HybridUserRepository;
use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;
use origa::traits::UserRepository;

#[component]
pub fn Profile() -> impl IntoView {
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let username = RwSignal::new(String::new());
    let disposed = StoredValue::new(());
    let repo_for_init = repository.clone();

    Effect::new(move |_| {
        let repo = repo_for_init.clone();
        spawn_local(async move {
            if let Ok(Some(user)) = repo.get_current_user().await {
                if disposed.is_disposed() {
                    return;
                }
                username.set(user.username().to_string());
                current_user.set(Some(user));
            }
        });
    });

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="profile-page">
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8" test_id="profile-card">
                <ProfileHeader username=Signal::derive(move || username.get()) />
                <ProfileContent />
            </CardLayout>
        </PageLayout>
    }
}
