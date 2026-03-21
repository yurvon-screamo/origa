pub mod content;
pub mod header;
pub mod history_modal;
pub mod home_skeleton;
pub mod home_stats;
pub mod jlpt_progress_card;
pub mod lesson_buttons_card;
pub mod stat_card;
pub mod stats_grid;

pub use content::HomeContent;
pub use header::HomeHeader;
pub use history_modal::{HistoryModal, StatMetric};
pub use home_skeleton::{HomeSkeleton, JlptSkeleton};
pub use home_stats::{HomeStats, calculate_stats, format_delta, format_number};
pub use jlpt_progress_card::JlptProgressCard;
pub use lesson_buttons_card::LessonButtonsCard;
pub use stat_card::StatCard;
pub use stats_grid::StatsGrid;

use crate::store::auth_store::AuthStore;
use crate::ui_components::{PageLayout, PageLayoutVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;
use origa::traits::UserRepository;

#[component]
pub fn Home() -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let repository = auth_store.repository().clone();

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);

    Effect::new({
        let repository = repository.clone();
        move |_| {
            let repository = repository.clone();
            spawn_local(async move {
                if let Ok(Some(user)) = repository.get_current_user().await {
                    current_user.set(Some(user));
                }
            });
        }
    });

    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <div class="flex flex-col pb-16">
                <HomeHeader current_user />
                <HomeContent />
            </div>
        </PageLayout>
    }
}
