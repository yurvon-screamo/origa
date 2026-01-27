use leptos::prelude::*;

#[component]
pub fn NextButton(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] disabled: Option<bool>,
    #[prop(into, optional)] variant: Option<NextVariant>,
) -> impl IntoView {
    let is_disabled = disabled.unwrap_or(false);
    let button_variant = variant.unwrap_or(NextVariant::Primary);
    let handle_click = move |_| {
        if !is_disabled && on_click.is_some() {
            on_click.unwrap().run(());
        }
    };

    let button_label = label.unwrap_or_else(|| "Далее".to_string());
    let button_label_for_aria = button_label.clone();

    view! {
        <button
            class=format!(
                "next-button {} {} {}",
                if is_disabled { "next-disabled" } else { "" },
                button_variant.to_class(),
                if is_disabled { "next-loading" } else { "" },
            )
            on:click=handle_click
            disabled=is_disabled
            aria-label=button_label_for_aria
        >
            <span class="next-icon">{"→"}</span>
            <span class="next-text">{button_label}</span>
            {is_disabled.then(|| view! { <span class="loading-spinner"></span> })}
        </button>
    }
}

#[component]
pub fn SkipButton(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] disabled: Option<bool>,
) -> impl IntoView {
    let is_disabled = disabled.unwrap_or(false);
    let handle_click = move |_| {
        if !is_disabled && on_click.is_some() {
            on_click.unwrap().run(());
        }
    };

    let button_label = label.unwrap_or_else(|| "Пропустить".to_string());
    let button_label_for_aria = button_label.clone();

    view! {
        <button
            class=format!(
                "skip-button {} {}",
                if is_disabled { "skip-disabled" } else { "" },
                if is_disabled { "skip-loading" } else { "" },
            )
            on:click=handle_click
            disabled=is_disabled
            aria-label=button_label_for_aria
        >
            <span class="skip-icon">{"⏭"}</span>
            <span class="skip-text">{button_label}</span>
        </button>
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum NextVariant {
    #[default]
    Primary,
    Secondary,
    Success,
    Warning,
}

impl NextVariant {
    pub fn to_class(&self) -> &'static str {
        match self {
            NextVariant::Primary => "next-primary",
            NextVariant::Secondary => "next-secondary",
            NextVariant::Success => "next-success",
            NextVariant::Warning => "next-warning",
        }
    }
}
