mod content;
mod header;

pub use content::ProfileContent;
pub use header::ProfileHeader;

use crate::ui_components::{PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Profile() -> impl IntoView {
    view! {
        <PageLayout variant={PageLayoutVariant::Centered}>
            <ProfileHeader />
            <ProfileContent />
        </PageLayout>
    }
}
