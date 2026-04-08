use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn ProfileHeader(username: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();

    view! {
        <div class="flex flex-wrap justify-between items-center gap-4 mb-6">
            <div class="flex flex-col items-center space-y-4 flex-1">
                <Heading level=HeadingLevel::H1 test_id="profile-title">
                    {move || {
                        let locale = i18n.get_locale();
                        td_string!(locale, profile.title)
                            .replace("{username}", &username.get())
                    }}
                </Heading>
            </div>
            <div class="flex items-center gap-2">
                <Button
                    variant=ButtonVariant::Ghost
                    test_id="profile-back-btn"
                    on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                        navigate("/home", Default::default());
                    })
                >
                    {t!(i18n, common.back)}
                </Button>
            </div>
        </div>
    }
}
