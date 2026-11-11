//! Compact category matrix for the expanded JLPT progress card
//! (`JlptProgressCard`): frameless stat tiles — each category is one calm
//! line (icon + label + N/M) over the layered progress track. One column
//! on mobile, 2×2 from `sm`, four in a row from `lg`. Every tile stays a
//! navigational link.

use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use origa::domain::CategoryProgress;

#[component]
pub fn CategoryProgressGrid(
    kanji_progress: Signal<CategoryProgress>,
    words_progress: Signal<CategoryProgress>,
    grammar_progress: Signal<CategoryProgress>,
    /// Счётные суффиксы (issue #415): четвёртая плитка, ведёт на /counters.
    #[prop(optional)]
    counters_progress: Option<Signal<CategoryProgress>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let grid_test_id = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div
            class="grid grid-cols-1 gap-x-4 gap-y-3 items-stretch sm:grid-cols-2 lg:grid-cols-4"
            data-testid=grid_test_id
        >
            <CategoryTile
                href="/kanji"
                icon=CategoryIcon::Kanji
                label=Signal::derive(move || td_string!(i18n.get_locale(), home.kanji_label).to_string())
                progress=kanji_progress
                fill_class="bg-[var(--accent-terracotta)]"
                icon_color_class="text-[var(--accent-terracotta)]"
                test_id=Signal::derive(move || format!("{}-kanji", test_id.get()))
            />
            <CategoryTile
                href="/words"
                icon=CategoryIcon::Words
                label=Signal::derive(move || td_string!(i18n.get_locale(), home.words_label).to_string())
                progress=words_progress
                fill_class="bg-[var(--accent-sage)]"
                icon_color_class="text-[var(--accent-sage)]"
                test_id=Signal::derive(move || format!("{}-words", test_id.get()))
            />
            <CategoryTile
                href="/grammar"
                icon=CategoryIcon::Grammar
                label=Signal::derive(move || td_string!(i18n.get_locale(), home.grammar_label).to_string())
                progress=grammar_progress
                fill_class="bg-[var(--fg-black)]"
                icon_color_class="text-[var(--fg-black)]"
                test_id=Signal::derive(move || format!("{}-grammar", test_id.get()))
            />
            {move || {
                let Some(counters_progress) = counters_progress else {
                    return ().into_any();
                };
                view! {
                    <CategoryTile
                        href="/counters"
                        icon=CategoryIcon::Counters
                        label=Signal::derive(move || {
                            td_string!(i18n.get_locale(), home.counters_label).to_string()
                        })
                        progress=counters_progress
                        fill_class="bg-[var(--accent-olive)]"
                        icon_color_class="text-[var(--accent-olive)]"
                        test_id=Signal::derive(move || format!("{}-counters", test_id.get()))
                    />
                }
                .into_any()
            }}
        </div>
    }
}

enum CategoryIcon {
    Kanji,
    Words,
    Grammar,
    Counters,
}

/// Frameless stat tile inside the outer JLPT card: no border, no shadow —
/// grouping is done by the grid, not by a nested card. The hover fill
/// carries the click affordance (DESIGN.md «Interaction Hierarchy»: no
/// border reads as not clickable, so an interactive tile must not look
/// like static text on hover).
#[component]
fn CategoryTile(
    #[prop(into)] href: Signal<String>,
    icon: CategoryIcon,
    #[prop(into)] label: Signal<String>,
    progress: Signal<CategoryProgress>,
    #[prop(into)] fill_class: Signal<String>,
    #[prop(into)] icon_color_class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let pct = Signal::derive(move || progress.get().percentage().min(100.0));
    let projected_pct = Signal::derive(move || progress.get().projected_percentage().min(100.0));
    let stats = Signal::derive(move || {
        let p = progress.get();
        format!("{} / {}", p.learned, p.total)
    });

    let tile_test_id = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let icon_view = match icon {
        CategoryIcon::Kanji => view! {
            <div class=move || format!("{} font-serif text-[18px] leading-none", icon_color_class.get())>
                "魚"
            </div>
        }
        .into_any(),
        CategoryIcon::Words => view! {
            <div class=move || icon_color_class.get().clone()>
                <Icon icon=icondata::LuBookOpen width="18" height="18" />
            </div>
        }
        .into_any(),
        CategoryIcon::Grammar => view! {
            <div class=move || icon_color_class.get().clone()>
                <Icon icon=icondata::LuClipboardList width="18" height="18" />
            </div>
        }
        .into_any(),
        CategoryIcon::Counters => view! {
            <div class=move || icon_color_class.get().clone()>
                <Icon icon=icondata::LuHash width="18" height="18" />
            </div>
        }
        .into_any(),
    };

    view! {
        <A href=move || href.get() attr:class="block h-full">
            <div
                data-testid=tile_test_id
                class="flex h-full cursor-pointer flex-col justify-between p-3 transition-all duration-200 hover:bg-[var(--bg-aged)] active:translate-y-[1px]"
            >
                <div class="flex items-start gap-2">
                    {icon_view}
                    // The label wraps if a locale outgrows the tile
                    // (never truncates — the name is the content); the
                    // N/M figure stays one line top-right.
                    <span class="min-w-0 font-mono text-[11px] uppercase leading-snug tracking-[0.15em] text-[var(--fg-muted)]">
                        {move || label.get()}
                    </span>
                    <span class="ml-auto shrink-0 whitespace-nowrap font-mono text-[13px] text-[var(--fg-black)]">
                        {move || stats.get()}
                    </span>
                </div>
                <div class="progress-track mt-3">
                    <div
                        class=move || format!("progress-fill progress-fill-projected {}", fill_class.get())
                        style=move || format!("width: {:.0}%", projected_pct.get().min(100.0))
                    ></div>
                    <div
                        class=move || format!("progress-fill progress-fill-layered {}", fill_class.get())
                        style=move || format!("width: {:.0}%", pct.get())
                    ></div>
                </div>
            </div>
        </A>
    }
}
