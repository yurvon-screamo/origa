use crate::ui_components::Skeleton;
use leptos::prelude::*;

/// Full-screen skeleton loading screen shown during app initialization
/// Blocks UI until all critical data (vocabulary, kanji, radicals, grammar) is loaded
#[component]
pub fn AppSkeleton(
    #[prop(optional, into, default = "".to_string().into())] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div data-testid=test_id_val class="app-skeleton">
            <div class="app-skeleton-content">
                // Header skeleton
                <div class="mb-8">
                    <Skeleton width="200px".to_string() height="32px".to_string() />
                </div>

                // Main content skeleton
                <div class="app-skeleton-lines">
                    // Card skeleton 1
                    <div class="rounded-xl overflow-hidden">
                        <Skeleton width="100%".to_string() height="120px".to_string() />
                    </div>

                    // Card skeleton 2
                    <div class="rounded-xl overflow-hidden">
                        <Skeleton width="100%".to_string() height="80px".to_string() />
                    </div>

                    // List skeleton
                    <div class="app-skeleton-lines">
                        <Skeleton width="100%".to_string() height="48px".to_string() />
                        <Skeleton width="100%".to_string() height="48px".to_string() />
                        <Skeleton width="100%".to_string() height="48px".to_string() />
                    </div>
                </div>
            </div>
        </div>
    }
}
