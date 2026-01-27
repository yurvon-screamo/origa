use leptos::prelude::*;

#[component]
pub function ChipGroup(
    chips: Signal<Vec<ChipItem>>,
    #[prop(into, optional)] on_select: Option<Callback<String>>,
    #[prop(into, optional)] multi_select: Option<bool>,
) -> impl IntoView {
    let allow_multi = multi_select.unwrap_or(false);
    let (selected_chips, set_selected_chips) = create_signal(std::collections::HashSet::<String>::new());
    
    let handle_chip_click = move |chip_id: String| {
        if allow_multi {
            let mut current = selected_chips.get();
            if current.contains(&chip_id) {
                current.remove(&chip_id);
            } else {
                current.insert(chip_id.clone());
            }
            set_selected_chips.set(current);
        } else {
            // Single selection - clear others, select this one
            set_selected_chips.set(std::iter::once(chip_id).collect());
        }
        
        if let Some(handler) = on_select {
            handler.run(chip_id);
        }
    };
    
    view! {
        <div class="chip-group">
            <For
                each=chips
                key=|chip| chip.id.clone()
                children=move |chip| {
                    let is_selected = Signal::derive(move || {
                        selected_chips().contains(&chip.id)
                    });
                    let chip_id = chip.id.clone();
                    let color = chip.color;
                    
                    view! {
                        <button 
                            class=format!(
                                "chip {} {} {}",
                                if is_selected() { "chip-active" } else { "" },
                                if is_selected() { "chip-bordered" } else { "" },
                                chip.class.unwrap_or("")
                            )
                            style=move || if is_selected() {
                                format!("--chip-color: {}; --chip-bg: {}", color, hex_to_rgba(color, 0.1))
                            } else {
                                format!("")
                            }
                            on:click=move |_| handle_chip_click(chip_id.clone())
                        >
                            <span class="chip-label">{chip.label}</span>
                            {chip.count.map(|count| view! {
                                <span class="chip-count">{count}</span>
                            })}
                            {chip.icon.map(|icon| view! {
                                <span class="chip-icon">{icon}</span>
                            })}
                        </button>
                    }
                }
            />
        </div>
    }
}

#[derive(Clone)]
pub struct ChipItem {
    pub id: String,
    pub label: String,
    pub count: Option<u32>,
    pub active: bool,
    pub color: &'static str,
    pub icon: Option<String>,
    pub class: Option<&'static str>,
}

impl ChipItem {
    pub fn new(id: &str, label: &str, color: &'static str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            count: None,
            active: false,
            color,
            icon: None,
            class: None,
        }
    }
    
    pub fn with_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }
    
    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }
    
    pub fn with_class(mut self, class: &'static str) -> Self {
        self.class = Some(class);
        self
    }
    
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

// Helper function to convert hex to rgba
fn hex_to_rgba(hex: &str, alpha: f32) -> String {
    if hex.len() != 7 || !hex.starts_with('#') {
        return format!("rgba(0, 0, 0, {})", alpha);
    }
    
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0);
    
    format!("rgba({}, {}, {}, {})", r, g, b, alpha)
}