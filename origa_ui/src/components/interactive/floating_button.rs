use leptos::prelude::*;

#[component]
pub fn FloatingButton(
    #[prop(into, optional)] icon: Option<String>,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] small: Option<bool>,
    #[prop(into, optional)] position: Option<FloatingPosition>,
) -> impl IntoView {
    let is_small = small.unwrap_or(false);
    let position = position.unwrap_or(FloatingPosition::BottomRight);
    let handle_click = move |_| {
        if let Some(handler) = on_click {
            handler.run(());
        }
    };
    
    let position_class = position.to_class();
    let size_class = if is_small { "floating-button-small" } else { "" };
    
    view! {
        <button 
            class=format!("floating-button {} {}", position_class, size_class)
            on:click=handle_click
            aria-label=label.clone().unwrap_or_else(|| "Действие".to_string())
        >
            <span class="floating-icon">{icon.unwrap_or_else(|| "+".to_string())}</span>
            {label.map(|lbl| view! {
                <span class="floating-label">{lbl}</span>
            })}
        </button>
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum FloatingPosition {
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

impl FloatingPosition {
    pub fn to_class(&self) -> &'static str {
        match self {
            FloatingPosition::BottomRight => "floating-bottom-right",
            FloatingPosition::BottomLeft => "floating-bottom-left", 
            FloatingPosition::TopRight => "floating-top-right",
            FloatingPosition::TopLeft => "floating-top-left",
        }
    }
}

#[component]
pub function FloatingActionButton(
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
    
    view! {
        <button 
            class=format!("fab {}", variant_class)
            on:click=handle_click
            aria-label=label.clone().unwrap_or_else(|| "Действие".to_string())
        >
            <span class="fab-icon">{icon_str}</span>
            {label.map(|lbl| view! {
                <span class="fab-label">{lbl}</span>
            })}
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