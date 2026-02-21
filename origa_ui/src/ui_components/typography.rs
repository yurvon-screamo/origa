use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    #[default]
    H6,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum TypographyVariant {
    #[default]
    Primary,
    Muted,
    Olive,
}

#[component]
pub fn Heading(
    #[prop(optional, into)] level: Signal<HeadingLevel>,
    #[prop(optional, into)] variant: Signal<TypographyVariant>,
    #[prop(optional, into)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <h1 class=move || {
            let variant_class = match variant.get() {
                TypographyVariant::Primary => "text-[var(--fg-primary)]",
                TypographyVariant::Muted => "text-[var(--fg-muted)]",
                TypographyVariant::Olive => "text-[var(--accent-olive)]",
            };

            let size_class = match level.get() {
                HeadingLevel::H1 => "text-5xl",
                HeadingLevel::H2 => "text-2xl",
                HeadingLevel::H3 => "text-xl",
                HeadingLevel::H4 => "text-lg",
                HeadingLevel::H5 => "text-base",
                HeadingLevel::H6 => "text-sm",
            };

            format!("font-serif font-light tracking-tight {} {} {}", size_class, variant_class, class.get())
        }>
            {children()}
        </h1>
    }
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum TextSize {
    #[default]
    Default,
    Small,
    Large,
}

#[component]
pub fn Text(
    #[prop(optional, into)] size: Signal<TextSize>,
    #[prop(optional, into)] variant: Signal<TypographyVariant>,
    #[prop(optional, into)] uppercase: Signal<bool>,
    #[prop(optional, into)] tracking_widest: Signal<bool>,
    #[prop(optional, into)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <p class=move || {
            let size_class = match size.get() {
                TextSize::Default => "text-sm",
                TextSize::Small => "text-xs",
                TextSize::Large => "text-base",
            };

            let variant_class = match variant.get() {
                TypographyVariant::Primary => "text-[var(--fg-primary)]",
                TypographyVariant::Muted => "text-[var(--fg-muted)]",
                TypographyVariant::Olive => "text-[var(--accent-olive)]",
            };

            let uppercase_class = if uppercase.get() { "uppercase" } else { "" };
            let tracking_class = if tracking_widest.get() { "tracking-widest" } else { "" };

            format!("font-mono {} {} {} {} {}", size_class, variant_class, uppercase_class, tracking_class, class.get())
        }>
            {children()}
        </p>
    }
}

#[component]
pub fn DisplayText(
    #[prop(optional, into)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <p class=move || format!("font-serif text-4xl font-light text-[var(--fg-primary)] {}", class.get())>
            {children()}
        </p>
    }
}
