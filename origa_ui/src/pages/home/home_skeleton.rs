use crate::ui_components::{Card, Skeleton};
use leptos::prelude::*;

#[component]
pub fn HomeSkeleton() -> impl IntoView {
    view! {
        <>
            <Skeleton
                width=Signal::derive(|| Some("100%".to_string()))
                height=Signal::derive(|| Some("200px".to_string()))
                class=Signal::derive(|| "mb-6".to_string())
            />
            <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-6">
                {(0..6).map(|_| view! {
                    <Card class=Signal::derive(|| "p-6".to_string())>
                        <Skeleton
                            width=Signal::derive(|| Some("60%".to_string()))
                            height=Signal::derive(|| Some("12px".to_string()))
                            class=Signal::derive(|| "mb-4".to_string())
                        />
                        <Skeleton
                            width=Signal::derive(|| Some("50%".to_string()))
                            height=Signal::derive(|| Some("32px".to_string()))
                            class=Signal::derive(|| "mb-2".to_string())
                        />
                        <Skeleton
                            width=Signal::derive(|| Some("70%".to_string()))
                            height=Signal::derive(|| Some("12px".to_string()))
                            class=Signal::derive(|| "mb-4".to_string())
                        />
                        <Skeleton
                            width=Signal::derive(|| Some("80px".to_string()))
                            height=Signal::derive(|| Some("36px".to_string()))
                            class=Signal::derive(String::new)
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
            width=Signal::derive(|| Some("100%".to_string()))
            height=Signal::derive(|| Some("200px".to_string()))
            class=Signal::derive(|| "mb-6".to_string())
        />
    }
}
