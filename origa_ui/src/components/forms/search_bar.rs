use leptos::prelude::*;

#[component]
pub fn SearchBar(
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] value: Option<Signal<String>>,
    #[prop(into, optional)] on_change: Option<Callback<String>>,
    #[prop(into, optional)] on_clear: Option<Callback<()>>,
) -> impl IntoView {
    let placeholder_text = placeholder.unwrap_or_else(|| "Поиск...".to_string());
    let search_value = value.unwrap_or_else(|| create_signal("".to_string()));
    
    let handle_input = move |ev| {
        let new_value = event_target_value(&ev);
        if let Some(on_change) = on_change {
            on_change.run(new_value);
        }
    };
    
    let handle_clear = move |_| {
        if let Some(on_clear) = on_clear {
            on_clear.run(());
        }
    };
    
    let is_empty = Signal::derive(move || search_value.get().is_empty());
    
    view! {
        <div class="search-bar">
            <div class="search-input-container">
                <span class="search-icon">🔍</span>
                <input 
                    type="text"
                    class="search-input"
                    placeholder=placeholder_text
                    prop:value=search_value
                    on:input=handle_input
                />
                {move || !is_empty().then(|| view! {
                    <button 
                        class="search-clear-btn"
                        on:click=handle_clear
                        aria-label="Очистить поиск"
                    >
                        "✕"
                    </button>
                })}
            </div>
        </div>
    }
}

#[component]
pub fn FilterChips(
    chips: Signal<Vec<FilterChip>>,
    #[prop(into, optional)] selected: Option<Signal<String>>,
    #[prop(into, optional)] on_select: Option<Callback<String>>,
) -> impl IntoView {
    let selected_chip = selected.unwrap_or_else(|| create_signal("all".to_string()));
    
    let handle_chip_click = move |chip_value: String| {
        if let Some(on_select) = on_select {
            on_select.run(chip_value);
        }
    };
    
    view! {
        <div class="filter-chips">
            <For
                each=chips
                key=|chip| chip.value.clone()
                children=move |chip| {
                    let is_active = Signal::derive(move || selected_chip.get() == chip.value);
                    let chip_value = chip.value.clone();
                    
                    view! {
                        <button 
                            class=format!(
                                "chip {}",
                                if is_active() { "chip-active" } else { "" }
                            )
                            on:click=move |_| handle_chip_click(chip_value.clone())
                        >
                            <span class="chip-icon">{chip.icon}</span>
                            <span class="chip-label">{chip.label}</span>
                            {chip.count.map(|count| view! {
                                <span class="chip-count">{count}</span>
                            })}
                        </button>
                    }
                }
            />
        </div>
    }
}

#[derive(Clone)]
pub struct FilterChip {
    pub value: String,
    pub label: String,
    pub icon: &'static str,
    pub count: Option<u32>,
}

impl FilterChip {
    pub fn new(value: &str, label: &str, icon: &'static str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            icon,
            count: None,
        }
    }
    
    pub fn with_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }
}