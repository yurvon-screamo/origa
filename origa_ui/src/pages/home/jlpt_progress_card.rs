use leptos::prelude::*;
use origa::domain::{CategoryProgress, JapaneseLevel, JlptProgress, LevelProgressDetail};

use crate::ui_components::{Heading, HeadingLevel, Text, TextSize, TypographyVariant};

#[component]
pub fn JlptProgressCard(
    jlpt_progress: Signal<JlptProgress>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let current_level = Signal::derive(move || jlpt_progress.get().current_level());
    let level_detail = Signal::derive(move || {
        let level = jlpt_progress.get().current_level();
        jlpt_progress.get().level_progress(level).cloned()
    });
    let next_lvl = Signal::derive(move || next_level(current_level.get()));
    let overall_pct = Signal::derive(move || {
        level_detail
            .get()
            .map(|d| d.overall_percentage())
            .unwrap_or(0.0)
    });

    view! {
        <div
            class="p-6 mb-6 bg-[var(--bg-paper)] border border-[var(--border-dark)]"
            data-testid=test_id_val
        >
            <Heading
                test_id=Signal::derive(move || {
                    let val = test_id.get();
                    if val.is_empty() { "jlpt-title".to_string() } else { format!("{}-title", val) }
                })
                level=Signal::from(HeadingLevel::H2)
                variant=Signal::from(TypographyVariant::Primary)
            >
                {move || format!("JLPT {}", current_level.get().code())}
            </Heading>

            <div
                class="mt-4"
                data-testid=move || {
                    let val = test_id.get();
                    if val.is_empty() { None } else { Some(format!("{}-progress", val)) }
                }
            >
                <div class="flex justify-between mb-2">
                    <Text
                        size=Signal::from(TextSize::Default)
                        variant=Signal::from(TypographyVariant::Muted)
                    >
                        {move || format!("Прогресс до {}", next_lvl.get().code())}
                    </Text>
                    <Text size=Signal::from(TextSize::Default)>
                        {move || format!("{:.0}%", overall_pct.get())}
                    </Text>
                </div>
                <div class="progress-track">
                    <div
                        class="progress-fill"
                        style=move || format!("width: {:.0}%", overall_pct.get().min(100.0))
                    ></div>
                </div>
            </div>

            <div class="mt-6">
                <CategoryProgressSection
                    test_id=Signal::derive(move || {
                        let val = test_id.get();
                        if val.is_empty() { "jlpt-categories".to_string() } else { format!("{}-categories", val) }
                    })
                    detail=level_detail
                />
            </div>
        </div>
    }
}

#[component]
fn CategoryProgressSection(
    detail: Signal<Option<LevelProgressDetail>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let is_expanded = RwSignal::new(false);
    let kanji = Signal::derive(move || detail.get().map(|d| d.kanji.clone()));
    let words = Signal::derive(move || detail.get().map(|d| d.words.clone()));
    let grammar = Signal::derive(move || detail.get().map(|d| d.grammar.clone()));

    view! {
        <div data-testid=test_id_val>
            <div
                class="flex items-center justify-between cursor-pointer select-none"
                on:click=move |_| is_expanded.update(|v| *v = !*v)
            >
                <Text size=Signal::from(TextSize::Small)>
                    <span class="font-semibold">"Детализация"</span>
                </Text>
                <span class="text-sm transition-transform" class:rotate-180=is_expanded>
                    "▼"
                </span>
            </div>

            <Show when=move || is_expanded.get()>
                <div class="mt-3 space-y-3">
                    <Show when=move || kanji.get().is_some()>
                        <CategoryProgressBar
                            test_id=Signal::derive(move || {
                                let val = test_id.get();
                                if val.is_empty() { "jlpt-kanji".to_string() } else { format!("{}-kanji", val) }
                            })
                            name="Кандзи"
                            progress=Signal::derive(move || kanji.get().unwrap())
                        />
                    </Show>
                    <Show when=move || words.get().is_some()>
                        <CategoryProgressBar
                            test_id=Signal::derive(move || {
                                let val = test_id.get();
                                if val.is_empty() { "jlpt-words".to_string() } else { format!("{}-words", val) }
                            })
                            name="Слова"
                            progress=Signal::derive(move || words.get().unwrap())
                        />
                    </Show>
                    <Show when=move || grammar.get().is_some()>
                        <CategoryProgressBar
                            test_id=Signal::derive(move || {
                                let val = test_id.get();
                                if val.is_empty() { "jlpt-grammar".to_string() } else { format!("{}-grammar", val) }
                            })
                            name="Грамматика"
                            progress=Signal::derive(move || grammar.get().unwrap())
                        />
                    </Show>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn CategoryProgressBar(
    name: &'static str,
    progress: Signal<CategoryProgress>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let pct = Signal::derive(move || progress.get().percentage());
    let size = Signal::from(TextSize::Small);

    view! {
        <div class="category-progress" data-testid=test_id_val>
            <div class="flex justify-between mb-1">
                <Text size=size>{name}</Text>
                <Text size=size>
                    {move || format!("{}/{} ({:.0}%)", progress.get().learned, progress.get().total, pct.get())}
                </Text>
            </div>
            <div class="progress-track">
                <div
                    class="progress-fill"
                    style=move || format!("width: {:.0}%", pct.get().min(100.0))
                ></div>
            </div>
        </div>
    }
}

fn next_level(current: JapaneseLevel) -> JapaneseLevel {
    match current {
        JapaneseLevel::N5 => JapaneseLevel::N4,
        JapaneseLevel::N4 => JapaneseLevel::N3,
        JapaneseLevel::N3 => JapaneseLevel::N2,
        JapaneseLevel::N2 => JapaneseLevel::N1,
        JapaneseLevel::N1 => JapaneseLevel::N1,
    }
}
