use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::seo::PageMeta;
use crate::content::Locale;

#[component]
pub fn IntegrationsPage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let c = locale.content();
    let prefix = locale.path_prefix();
    let download_href = format!("{prefix}/download");

    view! {
        <PageMeta locale path="/integrations" title=c.integrations_meta_title description=c.integrations_meta_description/>

        // Hero
        <section class="intg-hero">
            <h1 class="intg-hero__title">{c.integrations_h1}</h1>
            <p class="intg-hero__subtitle">{c.integrations_subtitle}</p>
            <hr class="intg-hero__rule"/>
        </section>

        // Catalog
        <section class="intg-catalog">
            <IntgCard tag=c.integrations_tag_exam name=c.integrations_jlpt_name desc=c.integrations_jlpt_desc detail=c.integrations_jlpt_detail/>
            <IntgCard tag=c.integrations_tag_app name=c.integrations_duolingo_name desc=c.integrations_duolingo_desc detail=c.integrations_duolingo_detail/>
            <IntgCard tag=c.integrations_tag_textbook name=c.integrations_minna_name desc=c.integrations_minna_desc detail=c.integrations_minna_detail/>
            <IntgCard tag=c.integrations_tag_textbook name=c.integrations_irodori_name desc=c.integrations_irodori_desc detail=c.integrations_irodori_detail/>
            <IntgCard tag=c.integrations_tag_app name=c.integrations_migii_name desc=c.integrations_migii_desc detail=c.integrations_migii_detail/>
            <IntgCard tag=c.integrations_tag_content name=c.integrations_spy_name desc=c.integrations_spy_desc detail=c.integrations_spy_detail/>

            // Anki card with editorial note
            <div class="intg-card">
                <p class="intg-card__tag">{c.integrations_tag_import}</p>
                <h3 class="intg-card__name">{c.integrations_anki_name}</h3>
                <p class="intg-card__desc">{c.integrations_anki_desc}</p>
                <p class="intg-card__detail">{c.integrations_anki_detail}</p>
                <IntgEditorialNote text=c.integrations_anki_note/>
            </div>
        </section>

        // CTA (reuses home-cta dark olive)
        <section class="home-cta">
            <hr class="home-cta__rule"/>
            <h2 class="home-cta__title">{c.home_cta_title}</h2>
            <A href=download_href attr:class="btn btn-filled">{c.home_cta_primary}</A>
            <p class="home-cta__platforms">
                "Windows · Linux · macOS · Android · iOS · Web"
            </p>
        </section>
    }
}

#[component]
fn IntgCard(
    tag: &'static str,
    name: &'static str,
    desc: &'static str,
    detail: &'static str,
) -> impl IntoView {
    view! {
        <div class="intg-card">
            <p class="intg-card__tag">{tag}</p>
            <h3 class="intg-card__name">{name}</h3>
            <p class="intg-card__desc">{desc}</p>
            <p class="intg-card__detail">{detail}</p>
        </div>
    }
}

#[component]
fn IntgEditorialNote(text: &'static str) -> impl IntoView {
    view! {
        <div class="intg-editorial-note">
            <p class="intg-editorial-note__text">{text}</p>
        </div>
    }
}
