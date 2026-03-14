use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum PageLayoutVariant {
    #[default]
    Centered,
    Full,
    #[allow(dead_code)]
    Compact,
}

#[component]
pub fn PageLayout(
    #[prop(optional, into)] variant: Signal<PageLayoutVariant>,
    #[prop(optional, into, default = "w-full px-4 sm:px-6 lg:px-8".to_string().into())]
    container_class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=move || {
            match variant.get() {
                PageLayoutVariant::Centered => "min-h-screen flex items-center justify-center p-4 sm:p-6 p-safe",
                PageLayoutVariant::Full => "min-h-screen flex flex-col p-4 sm:p-6 p-safe",
                PageLayoutVariant::Compact => "min-h-[calc(100vh-4rem)] p-4 sm:p-6 px-safe",
            }
        }>
            <div class=move || container_class.get()>
                {children()}
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum CardLayoutSize {
    #[allow(dead_code)]
    Small,
    #[allow(dead_code)]
    Medium,
    #[allow(dead_code)]
    Large,
    #[default]
    Adaptive,
}

#[component]
pub fn CardLayout(
    #[prop(optional, into)] size: Signal<CardLayoutSize>,
    #[prop(optional, into)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=move || {
            let base = match size.get() {
                CardLayoutSize::Small => "max-w-sm w-full mx-auto",
                CardLayoutSize::Medium => "max-w-md w-full mx-auto",
                CardLayoutSize::Large => "max-w-lg w-full mx-auto",
                CardLayoutSize::Adaptive => "w-full",
            };
            format!("{} {}", base, class.get())
        }>
            <div class="bg-[var(--bg-cream)] border border-[var(--border-dark)] p-4 sm:p-6 lg:p-8">
                {children()}
            </div>
        </div>
    }
}
