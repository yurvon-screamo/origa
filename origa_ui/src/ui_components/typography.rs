use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    #[allow(dead_code)]
    H5,
    #[default]
    H6,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum TypographyVariant {
    #[default]
    Primary,
    Muted,
    #[allow(dead_code)]
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
                TypographyVariant::Primary => "text-[var(--fg-black)]",
                TypographyVariant::Muted => "text-[var(--fg-muted)]",
                TypographyVariant::Olive => "text-[var(--accent-olive)]",
            };

            let size_class = match level.get() {
                HeadingLevel::H1 => "text-[clamp(1.75rem,6vw,3rem)] leading-tight break-words",
                HeadingLevel::H2 => "text-[clamp(1.5rem,5vw,2.5rem)] leading-tight break-words",
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
                TypographyVariant::Primary => "text-[var(--fg-black)]",
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
        <p class=move || format!("font-serif text-[clamp(1.5rem,5vw,2.25rem)] font-light leading-tight break-words text-[var(--fg-black)] {}", class.get())>
            {children()}
        </p>
    }
}
