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
    #[prop(optional)] level: HeadingLevel,
    #[prop(optional)] variant: TypographyVariant,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let variant_class = match variant {
        TypographyVariant::Primary => "text-[var(--fg-primary)]",
        TypographyVariant::Muted => "text-[var(--fg-muted)]",
        TypographyVariant::Olive => "text-[var(--accent-olive)]",
    };

    let size_class = match level {
        HeadingLevel::H1 => "text-5xl",
        HeadingLevel::H2 => "text-2xl",
        HeadingLevel::H3 => "text-xl",
        HeadingLevel::H4 => "text-lg",
        HeadingLevel::H5 => "text-base",
        HeadingLevel::H6 => "text-sm",
    };

    let full_class = format!("font-serif font-light tracking-tight {} {} {}", size_class, variant_class, class);

    view! {
        <h1 class=full_class>
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
    #[prop(optional)] size: TextSize,
    #[prop(optional)] variant: TypographyVariant,
    #[prop(optional)] uppercase: bool,
    #[prop(optional)] tracking_widest: bool,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let size_class = match size {
        TextSize::Default => "text-sm",
        TextSize::Small => "text-xs",
        TextSize::Large => "text-base",
    };

    let variant_class = match variant {
        TypographyVariant::Primary => "text-[var(--fg-primary)]",
        TypographyVariant::Muted => "text-[var(--fg-muted)]",
        TypographyVariant::Olive => "text-[var(--accent-olive)]",
    };

    let uppercase_class = if uppercase { "uppercase" } else { "" };
    let tracking_class = if tracking_widest { "tracking-widest" } else { "" };

    let full_class = format!("font-mono {} {} {} {} {}", size_class, variant_class, uppercase_class, tracking_class, class);

    view! {
        <p class=full_class>
            {children()}
        </p>
    }
}

#[component]
pub fn DisplayText(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let full_class = format!("font-serif text-4xl font-light text-[var(--fg-primary)] {}", class);
    view! {
        <p class=full_class>
            {children()}
        </p>
    }
}
