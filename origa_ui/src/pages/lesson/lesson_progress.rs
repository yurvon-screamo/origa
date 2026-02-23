use leptos::prelude::*;

#[component]
pub fn LessonProgress(current: usize, total: usize) -> impl IntoView {
    let percentage = move || {
        if total == 0 {
            0.0
        } else {
            (current as f64 / total as f64 * 100.0).min(100.0)
        }
    };

    view! {
        <div class="mb-6">
            <div class="flex justify-between mb-2">
                <span class="font-mono text-[10px] tracking-widest uppercase">
                    "Прогресс"
                </span>
                <span class="font-mono text-[10px]">
                    {move || format!("{}/{}", current, total)}
                </span>
            </div>
            <div class="progress-track">
                <div
                    class="progress-fill"
                    style=move || format!("width: {}%", percentage() as u32)
                ></div>
            </div>
        </div>
    }
}
