use leptos::prelude::*;

#[component]
pub fn LessonProgress(current: Signal<usize>, total: Signal<usize>) -> impl IntoView {
    let percentage = move || {
        let t = total.get();
        if t == 0 {
            0.0
        } else {
            (current.get() as f64 / t as f64 * 100.0).min(100.0)
        }
    };

    view! {
        <div class="mb-3 sm:mb-6">
            <div class="flex justify-between mb-2">
                <span class="font-mono text-[10px] tracking-widest uppercase">
                    "Прогресс"
                </span>
                <span class="font-mono text-[10px]" data-testid="lesson-progress-text">
                    {move || format!("{}/{}", current.get(), total.get())}
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
