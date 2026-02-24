pub mod content;
pub mod header;
pub mod history_modal;
pub mod stat_card;
pub mod tab_bar;

pub use content::HomeContent;
pub use header::HomeHeader;
pub use history_modal::{HistoryModal, StatMetric};
pub use stat_card::StatCard;
pub use tab_bar::TabBar;

use crate::ui_components::{PageLayout, PageLayoutVariant};
use leptos::prelude::*;
use origa::domain::User;

#[component]
pub fn Home() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");

    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <div class="min-h-screen flex flex-col pb-16">
                <HomeHeader current_user />
                <HomeContent />
                <TabBar />
            </div>
        </PageLayout>
    }
}
