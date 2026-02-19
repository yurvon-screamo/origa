use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonVariant {
    #[default]
    Default,
    Filled,
    Olive,
    Ghost,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonSize {
    #[default]
    Default,
    Small,
    Large,
}

#[component]
pub fn Button(
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: Signal<bool>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let variant_classes = match variant {
        ButtonVariant::Default => "",
        ButtonVariant::Filled => "btn-filled",
        ButtonVariant::Olive => "btn-olive",
        ButtonVariant::Ghost => "btn-ghost",
    };

    let size_classes = match size {
        ButtonSize::Default => "",
        ButtonSize::Small => "btn-sm",
        ButtonSize::Large => "btn-lg",
    };

    let classes = format!("btn {} {} {}", variant_classes, size_classes, class);

    view! {
        <button
            class=classes
            disabled=disabled
            on:click=move |ev| {
                if let Some(on_click) = on_click {
                    on_click.run(ev);
                }
            }
        >
            {children()}
        </button>
    }
}
