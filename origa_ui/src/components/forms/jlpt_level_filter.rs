use crate::components::forms::chip_group::ChipGroup;
use leptos::prelude::*;
use origa::domain::JapaneseLevel;

#[component]
pub fn JlptLevelFilter(
    #[prop(into, optional)] selected_level: Option<Signal<JapaneseLevel>>,
    #[prop(into, optional)] on_select: Option<Callback<JapaneseLevel>>,
) -> impl IntoView {
    let (selected_read, selected_write) = selected_level
        .map(|s| {
            let (read, write) = signal(s.get());
            (read, write)
        })
        .unwrap_or_else(|| signal(JapaneseLevel::N5));

    let handle_select = Callback::new(move |level: JapaneseLevel| {
        if let Some(handler) = on_select {
            handler.run(level);
        }
    });

    let chips = Signal::derive(move || {
        [
            JapaneseLevel::N5,
            JapaneseLevel::N4,
            JapaneseLevel::N3,
            JapaneseLevel::N2,
            JapaneseLevel::N1,
        ]
        .iter()
        .map(|&level| crate::components::forms::chip_group::ChipItem {
            id: level.code().to_string(),
            label: level.code().to_string(),
            count: None,
            active: selected_read.get() == level,
            color: get_jlpt_color(&level),
            icon: None,
            class: None,
        })
        .collect::<Vec<_>>()
    });

    view! {
        <div class="jlpt-filter">
            <div class="filter-header">
                <h3 class="filter-title">Уровень JLPT</h3>
                <p class="filter-subtitle">
                    Выберите уровень сложности кандзи
                </p>
            </div>

            <ChipGroup
                chips=chips
                on_select=Callback::new(move |chip_id: String| {
                    if let Ok(level) = chip_id.parse::<JapaneseLevel>() {
                        selected_write.set(level);
                        handle_select.run(level);
                    }
                })
            />

            // Progress indicator for selected level
            <div class="level-progress">
                <div class="progress-text">
                    "Уровень " {move || selected_read.get().code().to_string()}
                </div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: 25%"></div>
                </div>
            </div>
        </div>
    }
}

fn get_jlpt_color(level: &JapaneseLevel) -> &'static str {
    match level {
        JapaneseLevel::N5 => "#5a8c5a",
        JapaneseLevel::N4 => "#66a182",
        JapaneseLevel::N3 => "#b08d57",
        JapaneseLevel::N2 => "#b85450",
        JapaneseLevel::N1 => "#8b2635",
    }
}
