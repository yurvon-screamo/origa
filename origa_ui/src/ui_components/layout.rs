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
    #[prop(optional)] variant: PageLayoutVariant,
    #[prop(optional, into, default = "max-w-7xl mx-auto".to_string())] container_class: String,
    children: Children,
) -> impl IntoView {
    let base_classes = match variant {
        PageLayoutVariant::Centered => "min-h-screen flex items-center justify-center",
        PageLayoutVariant::Full => "min-h-screen",
        PageLayoutVariant::Compact => "min-h-[calc(100vh-4rem)]",
    };

    view! {
        <div class={base_classes}>
            <div class={container_class}>
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
pub fn CardLayout(#[prop(optional)] size: CardLayoutSize, children: Children) -> impl IntoView {
    let size_class = match size {
        CardLayoutSize::Small => "max-w-sm w-full",
        CardLayoutSize::Medium => "max-w-md w-full",
        CardLayoutSize::Large => "max-w-lg w-full",
    };

    view! {
        <div class={size_class}>
            <div class="bg-[var(--bg-primary)] border border-[var(--border-color)] p-8">
                {children()}
            </div>
        </div>
    }
}
