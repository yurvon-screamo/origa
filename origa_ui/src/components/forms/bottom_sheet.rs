use leptos::prelude::*;

#[component]
pub function BottomSheet(
    show: Signal<bool>,
    #[prop(into, optional)] title: Option<String>,
    #[prop(into, optional)] subtitle: Option<String>,
    #[prop(into, optional)] max_height: Option<String>,
    #[prop(into, optional)] on_close: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let handle_backdrop_click = move |_| {
        if let Some(handler) = on_close {
            handler.run(());
        }
    };
    
    let max_height_class = max_height.unwrap_or_else(|| "80vh".to_string());
    
    view! {
        <Show when=show>
            <div class="modal-overlay" on:click=handle_backdrop_click>
                <div 
                    class="modal-content bottom-sheet"
                    style=format!("max-height: {}", max_height_class)
                    on:click=move |ev| ev.stop_propagation()
                >
                    <Show when=move || title.is_some()>
                        <div class="modal-header">
                            <div class="modal-title-section">
                                <h2 class="modal-title">{move || title.clone().unwrap_or_default()}</h2>
                                {move || subtitle.map(|sub| view! {
                                    <p class="modal-subtitle">{sub}</p>
                                })}
                            </div>
                            <button 
                                class="icon-button modal-close-btn"
                                on:click=move |_| {
                                    if let Some(handler) = on_close {
                                        handler.run(());
                                    }
                                }
                                aria-label="Закрыть"
                            >
                                "✕"
                            </button>
                        </div>
                    </Show>
                    
                    <div class="modal-body bottom-sheet-body">
                        {children()}
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub function Modal(
    show: Signal<bool>,
    #[prop(into, optional)] title: Option<String>,
    #[prop(into, optional)] size: Option<ModalSize>,
    #[prop(into, optional)] on_close: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    let handle_backdrop_click = move |_| {
        if let Some(handler) = on_close {
            handler.run(());
        }
    };
    
    let size_class = size.unwrap_or(ModalSize::Medium).to_class();
    
    view! {
        <Show when=show>
            <div class="modal-overlay" on:click=handle_backdrop_click>
                <div 
                    class=format!("modal-content {}", size_class)
                    on:click=move |ev| ev.stop_propagation()
                >
                    <Show when=move || title.is_some()>
                        <div class="modal-header">
                            <div class="modal-title-section">
                                <h2 class="modal-title">{move || title.clone().unwrap_or_default()}</h2>
                            </div>
                            <button 
                                class="icon-button modal-close-btn"
                                on:click=move |_| {
                                    if let Some(handler) = on_close {
                                        handler.run(());
                                    }
                                }
                                aria-label="Закрыть"
                            >
                                "✕"
                            </button>
                        </div>
                    </Show>
                    
                    <div class="modal-body">
                        {children()}
                    </div>
                </div>
            </div>
        </Show>
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