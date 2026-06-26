use leptos::prelude::*;

use crate::components::cta::CtaSection;
use crate::components::seo::{PageMeta, SchemaOrg, breadcrumb_schema};
use crate::content::Locale;

#[component]
pub fn IntegrationsPage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let c = locale.content();
    let prefix = locale.path_prefix();
    let download_href = format!("{prefix}/download");

    view! {
        <PageMeta locale path="/content" title=c.integrations_meta_title description=c.integrations_meta_description/>
        <SchemaOrg json=breadcrumb_schema(locale, "/content", c.header_integrations)/>

        // ── Section 0: Hero ──
        <section class="intg-hero">
            <div class="intg-hero__inner">
                <div class="intg-hero__left">
                    <h1 class="intg-hero__title">{c.integrations_h1}</h1>
                    <hr class="intg-hero__rule"/>
                    <p class="intg-hero__subtitle">{c.integrations_subtitle}</p>
                    <p class="intg-hero__stat">{c.integrations_hero_stat}</p>
                </div>
                <div class="intg-hero__image">
                    <img src="/images/content.png" alt="Content sources" class="intg-hero__img"/>
                </div>
            </div>
        </section>

        // ── Section 1: Exam Preparation ──
        <section class="intg-section intg-section--cream">
            <div class="intg-section__inner">
                <IntgSectionHeader number="01" title=c.integrations_section_exams/>
                <IntgJlptCard
                    tag=c.integrations_tag_exam
                    name=c.integrations_jlpt_name
                    desc=c.integrations_jlpt_desc
                    detail=c.integrations_jlpt_detail
                />
            </div>
        </section>

        // ── Section 2: Textbooks ──
        <section class="intg-section intg-section--aged">
            <div class="intg-section__inner">
                <IntgSectionHeader number="02" title=c.integrations_section_textbooks/>
                <div class="intg-grid-2">
                    <IntgCard
                        tag=c.integrations_tag_textbook
                        name=c.integrations_minna_name
                        desc=c.integrations_minna_desc
                        detail=c.integrations_minna_detail
                    />
                    <IntgCard
                        tag=c.integrations_tag_textbook
                        name=c.integrations_irodori_name
                        desc=c.integrations_irodori_desc
                        detail=c.integrations_irodori_detail
                    />
                </div>
            </div>
        </section>

        // ── Section 3: Apps & Content ──
        <section class="intg-section intg-section--paper">
            <div class="intg-section__inner">
                <IntgSectionHeader number="03" title=c.integrations_section_apps/>
                <div class="intg-grid-stagger">
                    <div class="intg-grid-stagger__row intg-grid-stagger__row--3-2">
                        <IntgCard
                            tag=c.integrations_tag_app
                            name=c.integrations_duolingo_name
                            desc=c.integrations_duolingo_desc
                            detail=c.integrations_duolingo_detail
                            badges=("EN", "RU")
                        />
                        <IntgCard
                            tag=c.integrations_tag_app
                            name=c.integrations_migii_name
                            desc=c.integrations_migii_desc
                            detail=c.integrations_migii_detail
                        />
                    </div>
                    <div class="intg-grid-stagger__row intg-grid-stagger__row--2-3">
                        <IntgCard
                            tag=c.integrations_tag_content
                            name=c.integrations_spy_name
                            desc=c.integrations_spy_desc
                            detail=c.integrations_spy_detail
                        />
                        <IntgCard
                            tag=c.integrations_tag_import
                            name=c.integrations_anki_name
                            desc=c.integrations_anki_desc
                            detail=c.integrations_anki_detail
                        />
                    </div>
                </div>
            </div>
        </section>

        // ── Section 4: CTA ──
        <CtaSection title=c.home_cta_title button_text=c.home_cta_primary download_href=download_href />
    }
}

#[component]
fn IntgSectionHeader(number: &'static str, title: &'static str) -> impl IntoView {
    view! {
        <div class="intg-section__header">
            <span class="intg-section__number">{number}</span>
            " — "
            {title}
        </div>
    }
}

#[component]
fn IntgJlptCard(
    tag: &'static str,
    name: &'static str,
    desc: &'static str,
    detail: &'static str,
) -> impl IntoView {
    view! {
        <div class="intg-card intg-card--featured">
            <div class="intg-card__body">
                <div class="intg-card__text">
                    <p class="intg-card__tag">{tag}</p>
                    <h3 class="intg-card__name">{name}</h3>
                    <p class="intg-card__desc">{desc}</p>
                    <p class="intg-card__detail">{detail}</p>
                </div>
                    <div class="intg-card__levels">
                    <div class="intg-card__level intg-card__level--n5">
                        <span class="intg-card__level-label">"N5"</span>
                    </div>
                    <div class="intg-card__level intg-card__level--n4">
                        <span class="intg-card__level-label">"N4"</span>
                    </div>
                    <div class="intg-card__level intg-card__level--n3">
                        <span class="intg-card__level-label">"N3"</span>
                    </div>
                    <div class="intg-card__level intg-card__level--n2">
                        <span class="intg-card__level-label">"N2"</span>
                    </div>
                    <div class="intg-card__level intg-card__level--n1">
                        <span class="intg-card__level-label">"N1"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn IntgCard(
    tag: &'static str,
    name: &'static str,
    desc: &'static str,
    detail: &'static str,
    #[prop(optional)] badges: Option<(&'static str, &'static str)>,
) -> impl IntoView {
    view! {
        <div class="intg-card">
            <p class="intg-card__tag">{tag}</p>
            <h3 class="intg-card__name">{name}</h3>
            <p class="intg-card__desc">{desc}</p>
            <p class="intg-card__detail">{detail}</p>
            {badges.map(|(b1, b2)| view! {
                <div class="intg-card__badges">
                    <span class="intg-card__badge">{b1}</span>
                    <span class="intg-card__badge">{b2}</span>
                </div>
            })}
        </div>
    }
}
