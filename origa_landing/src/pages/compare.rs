use leptos::prelude::*;

use crate::components::cta::CtaSection;
use crate::components::seo::PageMeta;
use crate::content::{Content, Locale};

#[component]
pub fn ComparePage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let c = locale.content();
    let prefix = locale.path_prefix();
    let download_href = format!("{prefix}/download");

    view! {
        <PageMeta locale path="/compare" title=c.compare_meta_title description=c.compare_meta_description/>

        // Section 1: Hero
        <section class="cmp-hero">
            <h1 class="cmp-hero__title">{c.compare_h1}</h1>
            <p class="cmp-hero__subtitle">{c.compare_subtitle}</p>
            <hr class="cmp-hero__rule"/>
        </section>

        // Section 2: Scoreboard
        <section class="cmp-scoreboard" aria-label="Feature comparison">
            <div class="cmp-grid">
                // Header row
                <div class="cmp-grid__row">
                    <div class="cmp-grid__cell cmp-grid__header">
                        {c.compare_table_feature}
                    </div>
                    <div class="cmp-grid__cell cmp-grid__header cmp-grid__header--origa">
                        {c.compare_table_origa}
                    </div>
                    <div class="cmp-grid__cell cmp-grid__header">
                        {c.compare_table_anki}
                    </div>
                    <div class="cmp-grid__cell cmp-grid__header">
                        {c.compare_table_wanikani}
                    </div>
                    <div class="cmp-grid__cell cmp-grid__header">
                        {c.compare_table_bunpro}
                    </div>
                    <div class="cmp-grid__cell cmp-grid__header">
                        {c.compare_table_duolingo}
                    </div>
                </div>

                // Row: Vocabulary
                <div class="cmp-grid__row">
                    <ScoreboardRow feature=c.compare_vocab>
                        <ScoreboardCell kind="yes-origa" name="Origa"/>
                        <ScoreboardCell kind="yes" name="Anki"/>
                        <ScoreboardCell kind="yes" name="WaniKani"/>
                        <ScoreboardCell kind="no" name="Bunpro"/>
                        <ScoreboardCell kind="yes" name="Duolingo"/>
                    </ScoreboardRow>
                </div>

                // Row: Kanji
                <div class="cmp-grid__row">
                    <ScoreboardRow feature=c.compare_kanji>
                        <ScoreboardCell kind="yes-origa" name="Origa"/>
                        <ScoreboardCell kind="partial" name="Anki"/>
                        <ScoreboardCell kind="yes" name="WaniKani"/>
                        <ScoreboardCell kind="no" name="Bunpro"/>
                        <ScoreboardCell kind="yes" name="Duolingo"/>
                    </ScoreboardRow>
                </div>

                // Row: Grammar
                <div class="cmp-grid__row">
                    <ScoreboardRow feature=c.compare_grammar>
                        <ScoreboardCell kind="yes-origa" name="Origa"/>
                        <ScoreboardCell kind="no" name="Anki"/>
                        <ScoreboardCell kind="no" name="WaniKani"/>
                        <ScoreboardCell kind="yes" name="Bunpro"/>
                        <ScoreboardCell kind="no" name="Duolingo"/>
                    </ScoreboardRow>
                </div>

                // Row: Listening
                <div class="cmp-grid__row">
                    <ScoreboardRow feature=c.compare_listening>
                        <ScoreboardCell kind="yes-origa" name="Origa"/>
                        <ScoreboardCell kind="no" name="Anki"/>
                        <ScoreboardCell kind="no" name="WaniKani"/>
                        <ScoreboardCell kind="no" name="Bunpro"/>
                        <ScoreboardCell kind="yes" name="Duolingo"/>
                    </ScoreboardRow>
                </div>

                // Row: Your language
                <div class="cmp-grid__row">
                    <ScoreboardRow feature=c.compare_languages>
                        <ScoreboardCell kind="yes-origa" name="Origa"/>
                        <ScoreboardSpecialCell text=c.compare_manual name="Anki"/>
                        <ScoreboardSpecialCell text=c.compare_english_only name="WaniKani"/>
                        <ScoreboardSpecialCell text=c.compare_english_only name="Bunpro"/>
                        <ScoreboardSpecialCell text=c.compare_limited name="Duolingo"/>
                    </ScoreboardRow>
                </div>

                // Row: Offline
                <div class="cmp-grid__row">
                    <ScoreboardRow feature=c.compare_offline>
                        <ScoreboardCell kind="yes-origa" name="Origa"/>
                        <ScoreboardCell kind="yes" name="Anki"/>
                        <ScoreboardCell kind="no" name="WaniKani"/>
                        <ScoreboardCell kind="no" name="Bunpro"/>
                        <ScoreboardCell kind="partial" name="Duolingo"/>
                    </ScoreboardRow>
                </div>

                // Score row
                <div class="cmp-grid__row">
                    <div class="cmp-grid__cell cmp-grid__cell--score">
                        {c.compare_score}
                    </div>
                    <div class="cmp-grid__cell cmp-grid__cell--score-origa">
                        "6/6"
                        <span class="cmp-grid__label">"Origa"</span>
                    </div>
                    <div class="cmp-grid__cell cmp-grid__cell--score">
                        "3/6"
                        <span class="cmp-grid__label">"Anki"</span>
                    </div>
                    <div class="cmp-grid__cell cmp-grid__cell--score">
                        "2/6"
                        <span class="cmp-grid__label">"WaniKani"</span>
                    </div>
                    <div class="cmp-grid__cell cmp-grid__cell--score">
                        "1/6"
                        <span class="cmp-grid__label">"Bunpro"</span>
                    </div>
                    <div class="cmp-grid__cell cmp-grid__cell--score">
                        "4/6"
                        <span class="cmp-grid__label">"Duolingo"</span>
                    </div>
                </div>
            </div>
        </section>

        <ScoreboardLegend c />

        // Section 3: Bridge
        <section class="cmp-bridge">
            <hr class="cmp-bridge__rule"/>
            <p class="cmp-bridge__label">{c.compare_bridge_label}</p>
        </section>

        // Section 4: Anki
        <CompetitorBlock
            title=c.compare_anki_title
            what=c.compare_anki_what
            when=c.compare_anki_when
            better=c.compare_anki_better
            together=c.compare_anki_together
            label_best_for=c.compare_label_best_for
            label_origa_wins=c.compare_label_origa_wins
            label_together=c.compare_label_together
            bg="cream"
        />

        // Section 5: WaniKani
        <CompetitorBlock
            title=c.compare_wanikani_title
            what=c.compare_wanikani_what
            when=c.compare_wanikani_when
            better=c.compare_wanikani_better
            together=c.compare_wanikani_together
            label_best_for=c.compare_label_best_for
            label_origa_wins=c.compare_label_origa_wins
            label_together=c.compare_label_together
            bg="paper"
        />

        // Section 6: Bunpro
        <CompetitorBlock
            title=c.compare_bunpro_title
            what=c.compare_bunpro_what
            when=c.compare_bunpro_when
            better=c.compare_bunpro_better
            together=c.compare_bunpro_together
            label_best_for=c.compare_label_best_for
            label_origa_wins=c.compare_label_origa_wins
            label_together=c.compare_label_together
            bg="cream"
        />

        // Section 7: Duolingo (now uses CompetitorBlock)
        <CompetitorBlock
            title=c.compare_duolingo_title
            what=c.compare_duolingo_start
            when=c.compare_duolingo_subtitle
            better=c.compare_duolingo_grow
            together=c.compare_duolingo_together
            label_best_for=c.compare_label_best_for
            label_origa_wins=c.compare_label_origa_wins
            label_together=c.compare_label_together
            bg="paper"
        />

        // Section 8: CTA (reuses home-cta dark olive)
        <CtaSection title=c.home_cta_title button_text=c.home_cta_primary download_href=download_href />
    }
}

#[component]
fn ScoreboardRow(feature: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="cmp-grid__cell cmp-grid__cell--feature">
            {feature}
        </div>
        {children()}
    }
}

#[component]
fn ScoreboardCell(
    kind: &'static str,
    #[prop(optional)] name: Option<&'static str>,
) -> impl IntoView {
    let label = name.map(|n| view! { <span class="cmp-grid__label">{n}</span> });

    match kind {
        "yes" => view! {
            <div class="cmp-grid__cell cmp-grid__cell--default">
                <CompareMarker kind="yes"/>
                {label}
            </div>
        },
        "yes-origa" => view! {
            <div class="cmp-grid__cell cmp-grid__cell--origa">
                <CompareMarker kind="yes-origa"/>
                {label}
            </div>
        },
        "partial" => view! {
            <div class="cmp-grid__cell cmp-grid__cell--default">
                <CompareMarker kind="partial"/>
                {label}
            </div>
        },
        _ => view! {
            <div class="cmp-grid__cell cmp-grid__cell--default">
                <CompareMarker kind="no"/>
                {label}
            </div>
        },
    }
}

#[component]
fn ScoreboardSpecialCell(
    text: &'static str,
    #[prop(optional)] name: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="cmp-grid__cell cmp-grid__cell--default">
            <CompareSpecialText text/>
            {name.map(|n| view! { <span class="cmp-grid__label">{n}</span> })}
        </div>
    }
}

#[component]
fn CompareMarker(kind: &'static str) -> AnyView {
    match kind {
        "yes" => view! {
            <div class="compare-marker compare-marker--yes" aria-label="Yes"></div>
        }
        .into_any(),
        "yes-origa" => view! {
            <div class="compare-marker compare-marker--origa" aria-label="Yes"></div>
        }
        .into_any(),
        "partial" => view! {
            <div class="compare-marker compare-marker--partial" aria-label="Partial"></div>
        }
        .into_any(),
        _ => view! {
            <div class="compare-marker compare-marker--no" aria-label="No">"—"</div>
        }
        .into_any(),
    }
}

#[component]
fn CompareSpecialText(text: &'static str) -> impl IntoView {
    view! {
        <span class="cmp-special-text">{text}</span>
    }
}

#[component]
fn ScoreboardLegend(c: &'static Content) -> impl IntoView {
    view! {
        <div class="cmp-legend" role="note">
            <span class="cmp-legend__title">{c.compare_legend_title}</span>
            <ul class="cmp-legend__list">
                <LegendItem label=c.compare_legend_origa>
                    <CompareMarker kind="yes-origa"/>
                </LegendItem>
                <LegendItem label=c.compare_legend_supported>
                    <CompareMarker kind="yes"/>
                </LegendItem>
                <LegendItem label=c.compare_legend_partial>
                    <CompareMarker kind="partial"/>
                </LegendItem>
                <LegendItem label=c.compare_legend_not_supported>
                    <CompareMarker kind="no"/>
                </LegendItem>
                <LegendItem label=c.compare_legend_value>
                    <span class="cmp-special-text">"Aa"</span>
                </LegendItem>
            </ul>
        </div>
    }
}

#[component]
fn LegendItem(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <li class="cmp-legend__item">
            <span class="cmp-legend__marker">{children()}</span>
            <span class="cmp-legend__label">{label}</span>
        </li>
    }
}

#[component]
fn CompetitorBlock(
    title: &'static str,
    what: &'static str,
    when: &'static str,
    better: &'static str,
    together: &'static str,
    label_best_for: &'static str,
    label_origa_wins: &'static str,
    label_together: &'static str,
    bg: &'static str,
) -> impl IntoView {
    let section_class = match bg {
        "cream" => "cmp-competitor cmp-competitor--cream",
        _ => "cmp-competitor cmp-competitor--paper",
    };

    let has_together = !together.is_empty();

    view! {
        <section class=section_class>
            <div class="cmp-competitor__inner">
                <h2 class="cmp-competitor__title">{title}</h2>
                <p class="cmp-competitor__desc">{what}</p>
                <div class="cmp-competitor__cols">
                    <div class="cmp-competitor__col">
                        <span class="cmp-competitor__label">{label_best_for}</span>
                        <p class="cmp-competitor__text">{when}</p>
                    </div>
                    <div class="cmp-competitor__col cmp-competitor__col--origa">
                        <span class="cmp-competitor__label">{label_origa_wins}</span>
                        <p class="cmp-competitor__text">{better}</p>
                    </div>
                </div>
                {has_together.then(|| view! {
                    <div class="cmp-competitor__together">
                        <span class="cmp-competitor__label">{label_together}</span>
                        <p class="cmp-competitor__text">{together}</p>
                    </div>
                })}
            </div>
        </section>
    }
}
