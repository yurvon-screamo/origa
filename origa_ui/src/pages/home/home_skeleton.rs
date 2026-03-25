use crate::ui_components::{Card, Skeleton};
use leptos::prelude::*;

#[component]
pub fn HomeSkeleton() -> impl IntoView {
    view! {
        <>
            <Skeleton
                width="100%".to_string()
                height="200px".to_string()
                class="mb-6".to_string()
            />
            <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-6">
                {(0..6).map(|_| view! {
                    <Card class="p-6".to_string()>
                        <Skeleton
                            width="60%".to_string()
                            height="12px".to_string()
                            class="mb-4".to_string()
                        />
                        <Skeleton
                            width="50%".to_string()
                            height="32px".to_string()
                            class="mb-2".to_string()
                        />
                        <Skeleton
                            width="70%".to_string()
                            height="12px".to_string()
                            class="mb-4".to_string()
                        />
                        <Skeleton
                            width="80px".to_string()
                            height="36px".to_string()
                            class="".to_string()
                        />
                    </Card>
                }).collect::<Vec<_>>()}
            </div>
        </>
    }
}

#[component]
pub fn JlptSkeleton() -> impl IntoView {
    view! {
        <Skeleton
            width="100%".to_string()
            height="200px".to_string()
            class="mb-6".to_string()
        />
    }
}
