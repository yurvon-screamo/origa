use super::add_drawer::AddCountersDrawer;
use crate::i18n::use_i18n;
use crate::ui_components::{Button, ButtonVariant, PageHeader};
use leptos::prelude::*;

#[component]
pub fn CountersHeader(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let is_drawer_open = RwSignal::new(false);

    view! {
        <PageHeader
            back_path="".to_string()
            back_label=Signal::derive(move || i18n.get_keys().common().back().inner().to_string())
            title=Signal::derive(move || i18n.get_keys().counters().header().inner().to_string())
            test_id="counters"
        >
            <Button
                variant=Signal::derive(|| ButtonVariant::Olive)
                test_id="counters-add-btn"
                on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                    is_drawer_open.set(true);
                })
            >
                "+"
            </Button>
        </PageHeader>

        <AddCountersDrawer is_open=is_drawer_open refresh_trigger=refresh_trigger />
    }
}
