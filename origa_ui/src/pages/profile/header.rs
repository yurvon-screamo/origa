use crate::i18n::use_i18n;
use crate::ui_components::PageHeader;
use leptos::prelude::*;

#[component]
pub fn ProfileHeader(username: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <PageHeader
            back_path="/home".to_string()
            back_label=Signal::derive(move || i18n.get_keys().common().back().inner().to_string())
            title=Signal::derive(move || {
                let title = i18n.get_keys().profile().title().inner().to_string();
                title.replace("{username}", &username.get())
            })
            test_id="profile"
        />
    }
}
