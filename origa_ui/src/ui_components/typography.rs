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
    #[prop(optional, into)] test_id: Signal<String>,
    children: Children,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <h1
            data-testid=test_id_val
            class=move || {
                let variant_class = match variant.get() {
                    TypographyVariant::Primary => "text-primary",
                    TypographyVariant::Muted => "text-muted",
                    TypographyVariant::Olive => "text-olive",
                };

                let size_class = match level.get() {
                    HeadingLevel::H1 => "heading-h1",
                    HeadingLevel::H2 => "heading-h2",
                    HeadingLevel::H3 => "heading-h3",
                    HeadingLevel::H4 => "heading-h4",
                    HeadingLevel::H5 => "heading-h5",
                    HeadingLevel::H6 => "heading-h6",
                };

                format!("{} {} {}", size_class, variant_class, class.get())
            }
        >
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
    #[prop(optional, into)] test_id: Signal<String>,
    children: Children,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <p
            data-testid=test_id_val
            class=move || {
                let variant_class = match variant.get() {
                    TypographyVariant::Primary => "text-primary",
                    TypographyVariant::Muted => "text-muted",
                    TypographyVariant::Olive => "text-olive",
                };

                let size_class = match size.get() {
                    TextSize::Default => "",
                    TextSize::Small => "text-xs",
                    TextSize::Large => "text-base",
                };

                let uppercase_class = if uppercase.get() { "uppercase" } else { "" };
                let tracking_class = if tracking_widest.get() { "tracking-widest" } else { "" };

                format!("{} {} {} {} {}", size_class, variant_class, uppercase_class, tracking_class, class.get())
            }
        >
            {children()}
        </p>
    }
}

#[component]
pub fn DisplayText(
    #[prop(optional, into)] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    children: Children,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <p
            data-testid=test_id_val
            class=move || format!("display-text {}", class.get())
        >
            {children()}
        </p>
    }
}
