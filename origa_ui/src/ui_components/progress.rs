use leptos::prelude::*;

#[component]
pub fn ProgressBar(
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional)] value: RwSignal<u32>,
    #[prop(default = 100)] max: u32,
    #[prop(optional, into)] label: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let percentage = move || {
        let v = value.get();
        let m = max;
        (v as f64 / m as f64 * 100.0).min(100.0)
    };

    view! {
        <div data-testid=test_id_val>
            <div class="progress-header">
                <span class="progress-label">{move || label.get()}</span>
                <span class="progress-value">{move || format!("{}%", percentage() as u32)}</span>
            </div>
            <div class="progress-track">
                <div
                    class="progress-fill"
                    style=move || format!("--progress-width: {}%", percentage())
                ></div>
            </div>
        </div>
    }
}
