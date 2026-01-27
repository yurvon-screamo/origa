use leptos::prelude::*;

#[component]
pub fn FloatingActionButton(
    #[prop(into, optional)] icon: Option<String>,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] variant: Option<FabVariant>,
) -> impl IntoView {
    let variant = variant.unwrap_or(FabVariant::Primary);
    let icon_str = icon.unwrap_or_else(|| "+".to_string());
    let handle_click = move |_| {
        if let Some(handler) = on_click {
            handler.run(());
        }
    };

    let variant_class = variant.to_class();
    let label_str = label.clone().unwrap_or_else(|| "Действие".to_string());

    view! {
        <button class=format!("fab {}", variant_class) on:click=handle_click aria-label=label_str>
            <span class="fab-icon">{icon_str}</span>
            {label.map(|lbl| view! { <span class="fab-label">{lbl}</span> })}
        </button>
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum FabVariant {
    Primary,
    Secondary,
    Success,
    Error,
}

impl FabVariant {
    pub fn to_class(&self) -> &'static str {
        match self {
            FabVariant::Primary => "fab-primary",
            FabVariant::Secondary => "fab-secondary",
            FabVariant::Success => "fab-success",
            FabVariant::Error => "fab-error",
        }
    }
}
