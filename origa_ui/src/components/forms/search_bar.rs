use leptos::prelude::*;
use leptos_use::watch_debounced;

#[component]
pub fn SearchBar(
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] value: Option<Signal<String>>,
    #[prop(into, optional)] on_change: Option<Callback<String>>,
    #[prop(into, optional)] on_clear: Option<Callback<()>>,
    /// Debounce delay in milliseconds (default: 300ms)
    #[prop(into, optional)]
    debounce_ms: Option<f64>,
) -> impl IntoView {
    let placeholder_text = placeholder.unwrap_or_else(|| "Поиск...".to_string());
    let debounce_delay = debounce_ms.unwrap_or(300.0);

    let (search_value, set_search_value) = value
        .map(|s| {
            // Create a local signal that syncs with the provided signal
            let (read, write) = signal(s.get());
            (read, write)
        })
        .unwrap_or_else(|| signal("".to_string()));

    // Use leptos-use watch_debounced for better search performance
    // This watches the search_value signal and triggers callback after debounce delay
    let _ = watch_debounced(
        move || search_value.get(),
        move |new_value, _, _| {
            if let Some(on_change) = on_change {
                on_change.run(new_value.clone());
            }
        },
        debounce_delay,
    );

    let handle_input = move |ev| {
        let new_value = event_target_value(&ev);
        set_search_value.set(new_value);
    };

    let handle_clear = move |_| {
        set_search_value.set("".to_string());
        // Clear callback will be triggered by watch_debounced
        if let Some(on_clear) = on_clear {
            on_clear.run(());
        }
    };

    let is_empty = Signal::derive(move || search_value.get().is_empty());

    view! {
        <div class="search-bar">
            <div class="search-input-container">
                <span class="search-icon">{"🔍"}</span>
                <input
                    type="text"
                    class="search-input"
                    placeholder=placeholder_text
                    prop:value=move || search_value.get()
                    on:input=handle_input
                />
                {(!is_empty.get())
                    .then(|| {
                        view! {
                            <button
                                class="search-clear-btn"
                                on:click=handle_clear
                                aria-label="Очистить поиск"
                            >
                                "✕"
                            </button>
                        }
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
    let (selected_chip_read, _selected_chip_write) = selected
        .map(|s| {
            let (read, write) = signal(s.get());
            (read, write)
        })
        .unwrap_or_else(|| signal("all".to_string()));
    let selected_chip = selected_chip_read;

    let handle_chip_click = Callback::new(move |chip_value: String| {
        if let Some(on_select) = on_select {
            on_select.run(chip_value);
        }
    });

    view! {
        <div class="filter-chips">
            <For
                each=move || chips.get()
                key=|chip| chip.value.clone()
                children=move |chip| {
                    let chip_value = chip.value.clone();
                    let chip_value_for_signal = chip.value.clone();
                    let chip_icon = chip.icon;
                    let chip_label = chip.label.clone();
                    let chip_count = chip.count;
                    let is_active = Signal::derive(move || {
                        selected_chip.get() == chip_value_for_signal
                    });
                    // Clone all fields before using them in closures

                    view! {
                        <button
                            class=move || {
                                format!("chip {}", if is_active.get() { "chip-active" } else { "" })
                            }
                            on:click=move |_| handle_chip_click.run(chip_value.clone())
                        >
                            <span class="chip-icon">{chip_icon}</span>
                            <span class="chip-label">{chip_label}</span>
                            {chip_count
                                .map(|count| view! { <span class="chip-count">{count}</span> })}
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
