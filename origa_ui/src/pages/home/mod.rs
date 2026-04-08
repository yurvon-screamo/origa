pub mod content;
pub mod content_sync;
pub mod header;
pub mod history_modal;
pub mod home_skeleton;
pub mod home_stats;
pub mod jlpt_progress_card;
pub mod lesson_buttons_card;
pub mod nav_drawer;
pub mod stat_card;
pub mod stats_grid;

pub use content::HomeContent;
pub use header::HomeHeader;
pub use history_modal::{HistoryModal, StatMetric};
pub use home_skeleton::{HomeSkeleton, JlptSkeleton};
pub use home_stats::{PrimaryStats, SecondaryStats, calculate_stats, format_delta, format_number};
pub use jlpt_progress_card::JlptProgressCard;
pub use lesson_buttons_card::LessonButtonsCard;
pub use nav_drawer::NavDrawer;
pub use stat_card::QuickStatCard;
pub use stats_grid::StatsGrid;

use crate::i18n::*;
use crate::store::auth_store::AuthStore;
use crate::ui_components::{
    PageLayout, PageLayoutVariant, Spinner, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use origa::domain::User;
use origa::traits::UserRepository;

#[component]
pub fn Home() -> impl IntoView {
    let i18n = use_i18n();
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let repository = auth_store.repository().clone();
    let navigate = use_navigate();

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let is_checking_onboarding = RwSignal::new(true);
    let drawer_open: RwSignal<bool> = RwSignal::new(false);
    let disposed = StoredValue::new(());

    Effect::new({
        let repository = repository.clone();
        let navigate = navigate.clone();
        move |_| {
            let repository = repository.clone();
            let navigate = navigate.clone();
            spawn_local(async move {
                match repository.get_current_user().await {
                    Ok(Some(user)) => {
                        if disposed.is_disposed() {
                            return;
                        }
                        if user.imported_sets().is_empty() {
                            navigate("/onboarding", Default::default());
                            return;
                        }
                        current_user.set(Some(user));
                    },
                    Ok(None) => {
                        navigate("/login", Default::default());
                    },
                    Err(e) => {
                        tracing::error!("Home: get_current_user error: {:?}", e);
                        navigate("/login", Default::default());
                    },
                }
                is_checking_onboarding.set(false);
            });
        }
    });

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="home-page">
            <Show when=move || is_checking_onboarding.get()>
                <div class="flex flex-col items-center justify-center min-h-screen gap-4" data-testid="home-loading">
                    <Spinner test_id="home-spinner" />
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, home.loading)}
                    </Text>
                </div>
            </Show>

            <Show when=move || !is_checking_onboarding.get()>
                <div class="flex flex-col pb-16" data-testid="home-content">
                    <HomeHeader current_user drawer_open=drawer_open test_id="home-header" />
                    <HomeContent test_id="home-main" />
                    <NavDrawer is_open=drawer_open test_id="nav-drawer" />
                </div>
            </Show>
        </PageLayout>
    }
}
