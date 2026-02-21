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
    #[prop(optional, into)] variant: Signal<ButtonVariant>,
    #[prop(optional, into)] size: Signal<ButtonSize>,
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                let v = match variant.get() {
                    ButtonVariant::Default => "",
                    ButtonVariant::Filled => "btn-filled",
                    ButtonVariant::Olive => "btn-olive",
                    ButtonVariant::Ghost => "btn-ghost",
                };
                let s = match size.get() {
                    ButtonSize::Default => "",
                    ButtonSize::Small => "btn-sm",
                    ButtonSize::Large => "btn-lg",
                };
                format!("btn {} {} {}", v, s, class.get())
            }
            disabled=move || disabled.get()
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
