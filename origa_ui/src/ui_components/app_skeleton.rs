use crate::ui_components::Skeleton;
use leptos::prelude::*;

/// Full-screen skeleton loading screen shown during app initialization
/// Blocks UI until all critical data (vocabulary, kanji, radicals, grammar) is loaded
#[component]
pub fn AppSkeleton() -> impl IntoView {
    view! {
        <div class="fixed inset-0 z-[9999] flex items-center justify-center bg-[#f7f4ee]">
            <div class="w-full max-w-[800px] p-6">
                // Header skeleton
                <div class="mb-8">
                    <Skeleton width="200px".to_string() height="32px".to_string() />
                </div>

                // Main content skeleton
                <div class="flex flex-col gap-4">
                    // Card skeleton 1
                    <div class="rounded-xl overflow-hidden">
                        <Skeleton width="100%".to_string() height="120px".to_string() />
                    </div>

                    // Card skeleton 2
                    <div class="rounded-xl overflow-hidden">
                        <Skeleton width="100%".to_string() height="80px".to_string() />
                    </div>

                    // List skeleton
                    <div class="flex flex-col gap-2">
                        <Skeleton width="100%".to_string() height="48px".to_string() />
                        <Skeleton width="100%".to_string() height="48px".to_string() />
                        <Skeleton width="100%".to_string() height="48px".to_string() />
                    </div>
                </div>
            </div>
        </div>
    }
}
