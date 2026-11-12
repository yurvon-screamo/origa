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
    /// Extra classes for the actions wrapper (the buttons row). Pages
    /// whose header carries many actions pass a deliberate wrapped-state
    /// here — e.g. words passes `max-md:basis-full max-md:justify-between`,
    /// so below 768px the actions take their own full-width row (a tab
    /// strip under the title) instead of an accidental right-hugging
    /// wrap. Pages with one or two actions stay natural: the empty
    /// default preserves the flex-wrap behaviour of #823.
    #[prop(optional, into)]
    actions_class: Signal<String>,
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
            <div
                class=move || format!("ml-auto flex items-center gap-2 sm:gap-4 {}", actions_class.get())
                data-testid=move || format!("{}-actions", test_id.get())
            >
                {content}
            </div>
        }
    });

    view! {
        // flex-wrap: on narrow viewports the actions row wraps below the
        // title instead of squeezing it into a vertical letter column
        // (#823 words header carries four actions now). Pages whose header
        // fits the viewport are unaffected — wrapping only happens on
        // overflow.
        <div class="flex flex-wrap items-center gap-3 mb-6" data-testid=move || format!("{}-header", test_id.get())>
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
