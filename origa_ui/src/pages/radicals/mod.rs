mod action_buttons;
mod add_radical_modal;
mod add_radical_modal_handlers;
mod add_radical_modal_state;
mod content;
mod error_alert;
mod header;
mod radical_item;
mod radical_list;
mod selected_count;

pub use content::RadicalsContent;
pub use header::RadicalsHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Radicals() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full>
            <CardLayout size=CardLayoutSize::Adaptive class="px-4 py-8">
                <RadicalsHeader refresh_trigger=refresh_trigger />
                <RadicalsContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
