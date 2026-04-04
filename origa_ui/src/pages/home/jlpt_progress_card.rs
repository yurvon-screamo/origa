use leptos::prelude::*;
use origa::domain::{CategoryProgress, JlptProgress, LevelProgressDetail};

use crate::ui_components::{Card, DisplayText, Tag, TagVariant, Text, TextSize};

#[component]
pub fn JlptProgressCard(
    jlpt_progress: Signal<JlptProgress>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let current_level = Signal::derive(move || jlpt_progress.get().current_level());
    let level_detail = Signal::derive(move || {
        let level = jlpt_progress.get().current_level();
        jlpt_progress.get().level_progress(level).cloned()
    });
    let overall_pct = Signal::derive(move || {
        level_detail
            .get()
            .map(|d| d.overall_percentage())
            .unwrap_or(0.0)
    });

    view! {
        <Card
            shadow=Signal::from(true)
            class=Signal::derive(|| "p-6 sm:p-8 mb-8".to_string())
            test_id=test_id
        >
            <div
                class="flex items-center gap-4 mb-6"
                data-testid=move || {
                    let val = test_id.get();
                    if val.is_empty() { None } else { Some(format!("{}-progress", val)) }
                }
            >
                <Tag
                    variant=Signal::from(TagVariant::Default)
                    test_id=Signal::derive(move || format!("{}-stamp", test_id.get()))
                >
                    {move || format!("JLPT {}", current_level.get().code())}
                </Tag>
                <div class="flex-1 progress-track">
                    <div
                        class="progress-fill"
                        style=move || format!("width: {:.0}%", overall_pct.get().min(100.0))
                    ></div>
                </div>
                <DisplayText
                    class=Signal::derive(|| "text-sm".to_string())
                    test_id=Signal::derive(move || format!("{}-pct", test_id.get()))
                >
                    {move || format!("{:.0}%", overall_pct.get())}
                </DisplayText>
            </div>

            <CategoryDetailSection detail=level_detail test_id=test_id />
        </Card>
    }
}

#[component]
fn CategoryDetailSection(
    detail: Signal<Option<LevelProgressDetail>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let is_expanded = RwSignal::new(false);
    let kanji = Signal::derive(move || detail.get().map(|d| d.kanji.clone()));
    let words = Signal::derive(move || detail.get().map(|d| d.words.clone()));
    let grammar = Signal::derive(move || detail.get().map(|d| d.grammar.clone()));

    let section_test_id = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(format!("{}-categories", val))
        }
    };

    view! {
        <div data-testid=section_test_id>
            <button
                class="flex items-center justify-between w-full cursor-pointer select-none"
                on:click=move |_| is_expanded.update(|v| *v = !*v)
                data-testid=move || {
                    let val = test_id.get();
                    if val.is_empty() { None } else { Some(format!("{}-toggle", val)) }
                }
            >
                <Text size=Signal::from(TextSize::Small)>
                    <span>"Подробнее"</span>
                </Text>
                <span class="text-sm transition-transform" class:rotate-180=is_expanded>
                    "▼"
                </span>
            </button>

            <Show when=move || is_expanded.get()>
                <div class="mt-4 space-y-4">
                    <Show when=move || kanji.get().is_some()>
                        <CategoryRow
                            label="漢字 Кандзи"
                            tag_variant=Signal::from(TagVariant::Terracotta)
                            progress=Signal::derive(move || kanji.get().unwrap_or_default())
                            test_id=Signal::derive(move || format!("{}-kanji", test_id.get()))
                        />
                    </Show>
                    <Show when=move || words.get().is_some()>
                        <CategoryRow
                            label="言葉 Слова"
                            tag_variant=Signal::from(TagVariant::Olive)
                            progress=Signal::derive(move || words.get().unwrap_or_default())
                            test_id=Signal::derive(move || format!("{}-words", test_id.get()))
                        />
                    </Show>
                    <Show when=move || grammar.get().is_some()>
                        <CategoryRow
                            label="文法 Грамматика"
                            tag_variant=Signal::from(TagVariant::Filled)
                            progress=Signal::derive(move || grammar.get().unwrap_or_default())
                            test_id=Signal::derive(move || format!("{}-grammar", test_id.get()))
                        />
                    </Show>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn CategoryRow(
    label: &'static str,
    tag_variant: Signal<TagVariant>,
    progress: Signal<CategoryProgress>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let pct = Signal::derive(move || progress.get().percentage());
    let stats = Signal::derive(move || {
        let p = progress.get();
        format!("{}/{} ({:.0}%)", p.learned, p.total, pct.get())
    });

    view! {
        <div class="category-progress" data-testid=move || {
            let val = test_id.get();
            if val.is_empty() { None } else { Some(val) }
        }>
            <div class="flex items-center gap-3">
                <Tag variant=tag_variant test_id=Signal::derive(move || format!("{}-tag", test_id.get()))>
                    {label}
                </Tag>
                <div class="flex-1 progress-track">
                    <div
                        class="progress-fill"
                        style=move || format!("width: {:.0}%", pct.get().min(100.0))
                    ></div>
                </div>
                <Text size=Signal::from(TextSize::Small)>
                    {move || stats.get()}
                </Text>
            </div>
        </div>
    }
}
