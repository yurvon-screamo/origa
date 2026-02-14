use leptos::prelude::*;

#[component]
pub fn ProgressBar(
    #[prop(optional)] value: RwSignal<u32>,
    #[prop(default = 100)] max: u32,
    #[prop(optional, into)] label: String,
) -> impl IntoView {
    let percentage = move || {
        let v = value.get();
        let m = max;
        (v as f64 / m as f64 * 100.0).min(100.0)
    };

    view! {
        <div>
            <div class="flex justify-between mb-2">
                <span class="font-mono text-[10px] tracking-widest">{label}</span>
                <span class="font-mono text-[10px]">{move || format!("{}%", percentage() as u32)}</span>
            </div>
            <div class="progress-track">
                <div
                    class="progress-fill"
                    style=move || format!("width: {}%", percentage())
                ></div>
            </div>
        </div>
    }
}
