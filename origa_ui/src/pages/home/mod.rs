pub mod content;
pub mod header;

pub use content::HomeContent;
pub use header::HomeHeader;

use leptos::prelude::*;
use origa::domain::User;

#[component]
pub fn Home() -> impl IntoView {
    let current_user =
        use_context::<RwSignal<Option<User>>>().expect("current_user context not provided");

    view! {
        <div class="min-h-screen flex flex-col">
            <HomeHeader current_user />
            <HomeContent />
        </div>
    }
}
