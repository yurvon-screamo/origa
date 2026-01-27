use leptos::prelude::*;
use leptos_use::on_click_outside;

#[component]
pub fn BottomSheet(
    show: Signal<bool>,
    #[prop(into, optional)] title: Option<String>,
    #[prop(into, optional)] subtitle: Option<String>,
    #[prop(into, optional)] max_height: Option<String>,
    #[prop(into, optional)] on_close: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let max_height_style = max_height.unwrap_or_else(|| "80vh".to_string());

    // Extract all optional values before view! macro
    let title_text = title;
    let subtitle_text = subtitle;

    // Call children() once outside the reactive closure
    let children_view = children();

    // Use leptos-use on_click_outside for cleaner outside click handling
    let content_ref = NodeRef::<leptos::html::Div>::new();
    let _ = on_click_outside(content_ref, move |_| {
        if show.get() {
            if let Some(handler) = on_close {
                handler.run(());
            }
        }
    });

    let handle_close_click = move |_| {
        if let Some(handler) = on_close {
            handler.run(());
        }
    };

    view! {
        <div class=move || {
            if show.get() {
                "modal-overlay modal-visible"
            } else {
                "modal-overlay modal-hidden"
            }
        }>
            <div
                node_ref=content_ref
                class="modal-content bottom-sheet"
                style=format!("max-height: {}", max_height_style)
            >
                {title_text
                    .map(|t| {
                        let sub = subtitle_text.clone();
                        view! {
                            <div class="modal-header">
                                <div class="modal-title-section">
                                    <h2 class="modal-title">{t}</h2>
                                    {sub.map(|s| view! { <p class="modal-subtitle">{s}</p> })}
                                </div>
                                <button
                                    class="icon-button modal-close-btn"
                                    on:click=handle_close_click
                                    aria-label="Закрыть"
                                >
                                    "✕"
                                </button>
                            </div>
                        }
                    })}

                <div class="modal-body bottom-sheet-body">{children_view}</div>
            </div>
        </div>
    }
}

#[component]
pub fn Modal(
    show: Signal<bool>,
    #[prop(into, optional)] title: Option<String>,
    #[prop(into, optional)] size: Option<ModalSize>,
    #[prop(into, optional)] on_close: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let size_class = size.unwrap_or(ModalSize::Medium).to_class();

    // Extract title before view! macro
    let title_text = title;

    // Call children() once outside the reactive closure
    let children_view = children();

    // Use leptos-use on_click_outside for cleaner outside click handling
    let content_ref = NodeRef::<leptos::html::Div>::new();
    let _ = on_click_outside(content_ref, move |_| {
        if show.get() {
            if let Some(handler) = on_close {
                handler.run(());
            }
        }
    });

    let handle_close_click = move |_| {
        if let Some(handler) = on_close {
            handler.run(());
        }
    };

    view! {
        <div class=move || {
            if show.get() {
                "modal-overlay modal-visible"
            } else {
                "modal-overlay modal-hidden"
            }
        }>
            <div node_ref=content_ref class=format!("modal-content {}", size_class)>
                {title_text
                    .map(|t| {
                        view! {
                            <div class="modal-header">
                                <div class="modal-title-section">
                                    <h2 class="modal-title">{t}</h2>
                                </div>
                                <button
                                    class="icon-button modal-close-btn"
                                    on:click=handle_close_click
                                    aria-label="Закрыть"
                                >
                                    "✕"
                                </button>
                            </div>
                        }
                    })}

                <div class="modal-body">{children_view}</div>
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ModalSize {
    Small,
    Medium,
    Large,
    Full,
}

impl ModalSize {
    pub fn to_class(&self) -> &'static str {
        match self {
            ModalSize::Small => "modal-small",
            ModalSize::Medium => "modal-medium",
            ModalSize::Large => "modal-large",
            ModalSize::Full => "modal-full",
        }
    }
}
