use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn LessonProgress(
    current: Signal<usize>,
    total: Signal<usize>,
    #[prop(optional, into)] core_count: Signal<usize>,
) -> impl IntoView {
    let i18n = use_i18n();

    let has_phrases = move || {
        let cc = core_count.get();
        let t = total.get();
        cc > 0 && cc < t
    };

    let label_text = move || {
        let c = current.get();
        let t = total.get();
        let cc = core_count.get();
        if cc > 0 && cc < t {
            let phrase_total = t - cc;
            let core_done = c.min(cc);
            let phrase_done = c.saturating_sub(cc);
            format!("{}/{} + {}/{}", core_done, cc, phrase_done, phrase_total)
        } else {
            format!("{}/{}", c, t)
        }
    };

    let core_fill_width = move || {
        let c = current.get();
        let t = total.get();
        let cc = core_count.get();
        if t == 0 || cc == 0 {
            return 0.0_f64;
        }
        let core_done = (c.min(cc)) as f64;
        let section_ratio = cc as f64 / t as f64;
        (core_done / cc as f64 * section_ratio * 100.0).min(100.0)
    };

    let phrase_fill_width = move || {
        let c = current.get();
        let t = total.get();
        let cc = core_count.get();
        let phrase_total = t.saturating_sub(cc);
        if t == 0 || phrase_total == 0 {
            return 0.0_f64;
        }
        let phrase_done = c.saturating_sub(cc) as f64;
        let section_ratio = phrase_total as f64 / t as f64;
        (phrase_done / phrase_total as f64 * section_ratio * 100.0).min(100.0)
    };

    let divider_position = move || {
        let t = total.get();
        let cc = core_count.get();
        if t == 0 {
            0.0_f64
        } else {
            cc as f64 / t as f64 * 100.0
        }
    };

    let single_fill_width = move || {
        let c = current.get();
        let t = total.get();
        if t == 0 {
            0.0_f64
        } else {
            (c as f64 / t as f64 * 100.0).min(100.0)
        }
    };

    view! {
        <div class="mb-3 sm:mb-6">
            <div class="flex justify-between mb-2">
                <span class="font-mono text-[10px] tracking-widest uppercase">
                    {t!(i18n, lesson.progress)}
                </span>
                <span class="font-mono text-[10px]" data-testid="lesson-progress-text">
                    {label_text}
                </span>
            </div>

            <Show
                when=has_phrases
                fallback=move || {
                    view! {
                        <div class="progress-track-lesson">
                            <div
                                class="progress-fill-core"
                                style=move || format!("width: {}%", single_fill_width() as u32)
                            ></div>
                        </div>
                    }
                }
            >
                <div class="progress-container">
                    <div class="progress-track-lesson">
                        <div
                            class="progress-fill-core"
                            style=move || format!("width: {}%", core_fill_width() as u32)
                        ></div>
                        <div
                            class="progress-fill-phrase"
                            style=move || {
                                let divider = divider_position();
                                format!(
                                    "width: {}%; left: {}%",
                                    phrase_fill_width() as u32,
                                    divider as u32,
                                )
                            }
                        ></div>
                    </div>
                    <div
                        class="progress-divider"
                        style=move || format!("left: {}%", divider_position() as u32)
                    ></div>
                </div>
            </Show>
        </div>
    }
}
