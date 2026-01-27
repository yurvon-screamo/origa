use leptos::prelude::*;
use origa::domain::Rating;

#[component]
pub fn RatingButtons(
    #[prop(into, optional)] on_rate: Option<Callback<Rating>>,
    #[prop(into, optional)] disabled: Option<bool>,
    #[prop(into, optional)] show_result: Option<bool>,
    #[prop(into, optional)] selected_rating: Option<Rating>,
) -> impl IntoView {
    let is_disabled = disabled.unwrap_or(false);
    let show_result_state = show_result.unwrap_or(false);
    let selected = selected_rating.unwrap_or(Rating::Good);
    let handle_rate = on_rate.unwrap_or(Callback::new(|_| {}));
    
    view! {
        <div class="rating-buttons">
            <RatingButton 
                rating=Rating::Again
                label="Не знаю"
                icon="😵"
                color="#b85450"
                is_selected=show_result_state && selected == Rating::Again
                on_click=handle_rate.clone()
                disabled=is_disabled />
            <RatingButton 
                rating=Rating::Hard
                label="Плохо"
                icon="😰"
                color="#b08d57"
                is_selected=show_result_state && selected == Rating::Hard
                on_click=handle_rate.clone()
                disabled=is_disabled />
            <RatingButton 
                rating=Rating::Good
                label="Знаю"
                icon="😊"
                color="#5a8c5a"
                is_selected=show_result_state && selected == Rating::Good
                on_click=handle_rate.clone()
                disabled=is_disabled />
            <RatingButton 
                rating=Rating::Easy
                label="Идеально"
                icon="🎉"
                color="#4a6fa5"
                is_selected=show_result_state && selected == Rating::Easy
                on_click=handle_rate
                disabled=is_disabled />
        </div>
    }
}

#[component]
fn RatingButton(
    rating: Rating,
    label: &'static str,
    icon: &'static str,
    color: &'static str,
    is_selected: bool,
    on_click: Callback<Rating>,
    disabled: bool,
) -> impl IntoView {
    let handle_click = move |_| {
        on_click.run(rating);
    };
    
    view! {
        <button 
            class=format!(
                "rating-button {} {} {} {}",
                if disabled { "rating-disabled" } else { "" },
                if is_selected { "rating-selected" } else { "" },
                if is_selected { "rating-animation" } else { "" }
            )
            style=format!("--rating-color: {}; --rating-bg: {};", color, hex_to_rgba(color, 0.1))
            on:click=handle_click
            disabled=disabled
            aria-label=label
            aria-pressed=is_selected
        >
            <span class="rating-icon">{icon}</span>
            <span class="rating-label">{label}</span>
            {is_selected.then(|| view! {
                <span class="rating-checkmark">✓</span>
            })}
        </button>
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