use crate::ui_components::{Button, ButtonVariant, Heading, HeadingLevel};
use icondata::LuArrowLeft;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_navigate;

#[component]
pub fn PageHeader(
    #[prop(optional, into)] back_path: Signal<String>,
    #[prop(optional, into)] back_label: Signal<String>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let navigate = use_navigate();
    let on_back = Callback::new(move |_: leptos::ev::MouseEvent| {
        let path = back_path.get();
        if !path.is_empty() {
            navigate(&path, Default::default());
        }
    });

    let actions = children.map(|c| {
        let content = c();
        view! {
            <div class="ml-auto flex items-center gap-2 sm:gap-4">
                {content}
            </div>
        }
    });

    view! {
        <div class="flex items-center gap-3 mb-6">
            <Show when=move || !back_path.get().is_empty()>
                <Button
                    variant=ButtonVariant::Ghost
                    test_id=Signal::derive(move || format!("{}-back-btn", test_id.get()))
                    on_click=on_back
                >
                    <Icon icon=LuArrowLeft width="16" height="16" />
                    {move || back_label.get()}
                </Button>
            </Show>
            <Show when=move || !title.get().is_empty()>
                <Heading level=HeadingLevel::H1 test_id=Signal::derive(move || format!("{}-title", test_id.get()))>
                    {move || title.get()}
                </Heading>
            </Show>
            {actions}
        </div>
    }
}
