use crate::ui_components::{Card, Skeleton};
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
                    if val.is_empty() { "home-skeleton-header".to_string() } else { format!("{}-header", val) }
                })
                width="100%".to_string()
                height="200px".to_string()
                class="mb-6".to_string()
            />
            <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-6">
                {(0..6).map(|i| view! {
                    <Card class="p-6".to_string()>
                        <Skeleton
                            test_id=Signal::derive(move || {
                                let val = test_id.get();
                                if val.is_empty() { format!("home-skeleton-card-{}-title", i) } else { format!("{}-card-{}-title", val, i) }
                            })
                            width="60%".to_string()
                            height="12px".to_string()
                            class="mb-4".to_string()
                        />
                        <Skeleton
                            test_id=Signal::derive(move || {
                                let val = test_id.get();
                                if val.is_empty() { format!("home-skeleton-card-{}-value", i) } else { format!("{}-card-{}-value", val, i) }
                            })
                            width="50%".to_string()
                            height="32px".to_string()
                            class="mb-2".to_string()
                        />
                        <Skeleton
                            test_id=Signal::derive(move || {
                                let val = test_id.get();
                                if val.is_empty() { format!("home-skeleton-card-{}-desc", i) } else { format!("{}-card-{}-desc", val, i) }
                            })
                            width="70%".to_string()
                            height="12px".to_string()
                            class="mb-4".to_string()
                        />
                        <Skeleton
                            test_id=Signal::derive(move || {
                                let val = test_id.get();
                                if val.is_empty() { format!("home-skeleton-card-{}-button", i) } else { format!("{}-card-{}-button", val, i) }
                            })
                            width="80px".to_string()
                            height="36px".to_string()
                            class="".to_string()
                        />
                    </Card>
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
            class="mb-6".to_string()
        />
    }
}
