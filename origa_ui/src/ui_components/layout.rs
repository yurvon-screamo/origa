use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PageLayoutVariant {
    Centered,
    Full,
    Compact,
}

impl Default for PageLayoutVariant {
    fn default() -> Self {
        Self::Centered
    }
}

#[component]
pub fn PageLayout(
    #[prop(optional, into)] variant: Signal<PageLayoutVariant>,
    #[prop(optional, into, default = "max-w-7xl mx-auto".to_string().into())]
    container_class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=move || {
            match variant.get() {
                PageLayoutVariant::Centered => "min-h-screen flex items-center justify-center",
                PageLayoutVariant::Full => "min-h-screen",
                PageLayoutVariant::Compact => "min-h-[calc(100vh-4rem)]",
            }
        }>
            <div class=move || container_class.get()>
                {children()}
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CardLayoutSize {
    Small,
    Medium,
    Large,
}

impl Default for CardLayoutSize {
    fn default() -> Self {
        Self::Medium
    }
}

#[component]
pub fn CardLayout(
    #[prop(optional, into)] size: Signal<CardLayoutSize>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=move || {
            match size.get() {
                CardLayoutSize::Small => "max-w-sm w-full",
                CardLayoutSize::Medium => "max-w-md w-full",
                CardLayoutSize::Large => "max-w-lg w-full",
            }
        }>
            <div class="bg-[var(--bg-primary)] border border-[var(--border-color)] p-8">
                {children()}
            </div>
        </div>
    }
}
