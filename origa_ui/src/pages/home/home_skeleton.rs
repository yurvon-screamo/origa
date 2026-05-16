use crate::ui_components::Skeleton;
use leptos::prelude::*;

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
