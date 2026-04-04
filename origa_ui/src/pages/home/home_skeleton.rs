use crate::ui_components::Skeleton;
use leptos::prelude::*;

#[component]
pub fn HomeSkeleton(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div data-testid=test_id_val>
            <Skeleton
                test_id=Signal::derive(move || {
                    let val = test_id.get();
                    if val.is_empty() { "home-skeleton-hero".to_string() } else { format!("{}-hero", val) }
                })
                width="100%".to_string()
                height="200px".to_string()
                class="mb-8".to_string()
            />

            <Skeleton
                test_id=Signal::derive(move || {
                    let val = test_id.get();
                    if val.is_empty() { "home-skeleton-cta".to_string() } else { format!("{}-cta", val) }
                })
                width="100%".to_string()
                height="120px".to_string()
                class="mb-6".to_string()
            />

            <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
                {(0..4).map(|i| {
                    view! {
                        <div>
                            <Skeleton
                                test_id=Signal::derive(move || {
                                    let val = test_id.get();
                                    if val.is_empty() { format!("home-skeleton-stat-{}", i) } else { format!("{}-stat-{}", val, i) }
                                })
                                width="100%".to_string()
                                height="80px".to_string()
                                class="mb-2".to_string()
                            />
                            <Skeleton
                                test_id=Signal::derive(move || {
                                    let val = test_id.get();
                                    if val.is_empty() { format!("home-skeleton-label-{}", i) } else { format!("{}-label-{}", val, i) }
                                })
                                width="60%".to_string()
                                height="16px".to_string()
                                class="mb-1".to_string()
                            />
                            <Skeleton
                                test_id=Signal::derive(move || {
                                    let val = test_id.get();
                                    if val.is_empty() { format!("home-skeleton-desc-{}", i) } else { format!("{}-desc-{}", val, i) }
                                })
                                width="40%".to_string()
                                height="12px".to_string()
                            />
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
pub fn JlptSkeleton(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    view! {
        <Skeleton
            test_id=Signal::derive(move || {
                let val = test_id.get();
                if val.is_empty() { "jlpt-skeleton".to_string() } else { val }
            })
            width="100%".to_string()
            height="200px".to_string()
            class="mb-8".to_string()
        />
    }
}
