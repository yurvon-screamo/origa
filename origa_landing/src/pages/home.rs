use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::cta::CtaSection;
use crate::components::seo::{
    PageMeta, SchemaOrg, faq_schema, organization_schema, software_application_schema,
};
use crate::content::Locale;

#[component]
pub fn HomePage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let c = locale.content();
    let prefix = locale.path_prefix();
    let lang = locale.image_prefix();
    let app_url = env!("ORIGA_APP_BASE_URL");

    let features_href = format!("{prefix}/features");
    let download_href = format!("{prefix}/download");

    // Real app screens, one per slide: the fastest way for a visitor to
    // see what the product actually is. Phone captures only — their big
    // elements stay legible at hero scale. KO/VI fall back to EN via
    // `image_prefix()` until the app ships those interface locales.
    let carousel: &[(&str, &str)] = &[
        ("main", c.home_carousel_labels[0]),
        ("lesson", c.home_carousel_labels[1]),
        ("grammar", c.home_carousel_labels[2]),
        ("writing", c.home_carousel_labels[3]),
        ("phrases", c.home_carousel_labels[4]),
    ];

    // Mini-FAQ: the "free?", WaniKani and kanji-start pairs are
    // homepage-specific long-tails; the rest reuse the features-page
    // strings so the same answer never diverges between pages. Visible
    // Q&A mirrors the FAQPage JSON-LD 1:1 (same contract as
    // `features.rs`).
    let faq_pairs = [
        (c.home_faq_free_question, c.home_faq_free_answer),
        (c.faq_q1, c.faq_a1),
        (c.faq_q2, c.faq_a2),
        (c.faq_q3, c.faq_a3),
        (c.faq_q4, c.faq_a4),
        (c.faq_q5, c.faq_a5),
        (c.home_faq_wanikani_question, c.home_faq_wanikani_answer),
        (
            c.home_faq_kanji_start_question,
            c.home_faq_kanji_start_answer,
        ),
    ];
    let faq_json = faq_schema(locale, &faq_pairs);

    view! {
        <PageMeta locale title=c.home_meta_title description=c.home_meta_description/>
        <SchemaOrg json=software_application_schema(locale)/>
        <SchemaOrg json=organization_schema()/>
        <SchemaOrg json=faq_json/>

        // Section 1: Hero (Split layout)
        <section class="home-hero">
            <div class="home-hero__text">
                <h1 class="home-hero__title">{c.home_hero_title}</h1>
                <hr class="home-hero__rule" />
                <p class="home-hero__subtitle">{c.home_hero_subtitle}</p>
                <div class="home-hero__cta">
                    <A
                        href=format!("{prefix}/download")
                        attr:class="btn btn-filled"
                        attr:data-umami-event="hero_cta_download"
                    >
                        {c.home_cta_primary}
                    </A>
                    <a href=app_url class="btn" attr:data-umami-event="open_webapp">
                        {c.home_cta_secondary}
                    </a>
                </div>
            </div>
            <div class="home-hero__decor">
                <img
                    src="/images/app/hero-art.webp"
                    alt=""
                    class="home-hero__art"
                    aria-hidden="true"
                />
            </div>
        </section>
        <script inner_html=carousel_inline_script() />

        <hr class="divider-full" />

        // Section 1b: stat strip — concrete, scannable facts under the hero
        <section class="home-stats">
            <p class="home-stats__line">{c.home_stats_line}</p>
        </section>

        <hr class="divider-full" />

        // Section 1c: app screens carousel — what the product actually
        // looks like, one legible phone capture at a time. Text column
        // frames the phone on wide viewports; the two stack on mobile.
        <section class="home-shots">
            <div class="home-shots__inner">
                <div class="home-shots__text">
                    <h2 class="home-shots__title">{c.home_shots_title}</h2>
                    <hr class="home-shots__rule" />
                    <p class="home-shots__desc">{c.home_shots_text}</p>
                </div>
                <div class="home-shots__phone">
                    <div class="hero-carousel" id="hero-carousel">
                        {carousel.iter().enumerate().map(|(i, (img, label))| {
                            view! {
                                <figure class="hero-carousel__slide">
                                    <img
                                        src=format!("/images/app/{lang}.car.{img}.webp")
                                        alt=label.to_string()
                                        loading=if i == 0 { "eager" } else { "lazy" }
                                    />
                                    <figcaption class="hero-carousel__caption">
                                        {label.to_string()}
                                    </figcaption>
                                </figure>
                            }
                        }).collect_view()}
                    </div>
                    <div class="hero-carousel__dots" id="hero-carousel-dots"></div>
                </div>
            </div>
        </section>

        <hr class="divider-full" />

        // Section 2: Problem + Principles (2-column)
        <section class="home-dual">
            <div class="home-dual__inner">
                // Left: Problem
                <div class="home-dual__problem">
                    <h2 class="home-dual__problem-title">{c.home_problem_h2}</h2>
                    <hr class="home-dual__problem-rule" />
                    <p class="home-dual__problem-text">{c.home_problem_text}</p>
                </div>
                // Right: Principles (no heading)
                <div class="home-dual__principles">
                    <div class="home-dual__principle">
                        <strong>{c.home_principle_content_title}</strong>
                        " "
                        {c.home_principle_content_text}
                    </div>
                    <div class="home-dual__principle">
                        <strong>{c.home_principle_fsrs_title}</strong>
                        " "
                        {c.home_principle_fsrs_text}
                    </div>
                    <div class="home-dual__principle">
                        <strong>{c.home_principle_local_title}</strong>
                        " "
                        {c.home_principle_local_text}
                    </div>
                    <div class="home-dual__principle">
                        <strong>{c.home_principle_offline_title}</strong>
                        " "
                        {c.home_principle_offline_text}
                    </div>
                </div>
            </div>
        </section>

        <hr class="divider-full" />

        // Section 3: Features Preview (staggered grid)
        <section class="home-features">
            <h2 class="home-features__title">{c.home_features_h2}</h2>
            <div class="home-features__grid home-features__grid--top">
                <HomeFeatureCard
                    title=c.home_feature_vocab_title
                    text=c.home_feature_vocab_text
                    href=features_href.clone()
                />
                <HomeFeatureCard
                    title=c.home_feature_kanji_title
                    text=c.home_feature_kanji_text
                    href=features_href.clone()
                />
            </div>
            <div
                class="home-features__grid home-features__grid--bottom"
                style="margin-top: var(--space-lg)"
            >
                <HomeFeatureCard
                    title=c.home_feature_grammar_title
                    text=c.home_feature_grammar_text
                    href=features_href.clone()
                />
                <HomeFeatureCard
                    title=c.home_feature_listening_title
                    text=c.home_feature_listening_text
                    href=features_href
                />
            </div>
        </section>

        <hr class="divider-full" />

        // Section 4: Mini-FAQ (visible Q&A mirrors the FAQPage JSON-LD 1:1)
        <section class="feat-faq">
            <div class="feat-faq__inner">
                <h2>{c.features_faq_h2}</h2>
                <div class="feat-faq__list">
                    {faq_pairs.map(|(question, answer)| {
                        view! {
                            <div class="feat-faq__item">
                                <p class="feat-faq__question">{question}</p>
                                <p class="feat-faq__answer">{answer}</p>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </section>

        <hr class="divider-full" />

        // Section 5: Final CTA (dark olive) with platforms
        <CtaSection title=c.home_cta_title button_text=c.home_cta_primary download_href=download_href />
    }
}

#[component]
fn HomeFeatureCard(title: &'static str, text: &'static str, href: String) -> impl IntoView {
    view! {
        <div class="home-feature-card">
            <h3 class="home-feature-card__title">{title}</h3>
            <p class="home-feature-card__text">{text}</p>
            <A href=href attr:class="landing-feature-card__link">{title}</A>
        </div>
    }
}

/// Carousel behaviour: auto-advance with pause on hover/focus, dot
/// navigation, native swipe (scroll-snap does the panning; JS only
/// syncs the dots and drives the timer). No dependencies, mirrors the
/// header script pattern.
///
/// Initialization is deferred to DOMContentLoaded: the script tag sits
/// above the carousel markup in the SSR output, and a synchronous run
/// would find no `#hero-carousel` (the bd061bc5 regression). Deferring
/// also makes the script's position in the markup irrelevant; the
/// `dots.children` guard keeps a duplicated script tag from double-init.
fn carousel_inline_script() -> String {
    r#"
    (function() {
        function init() {
            var track = document.getElementById('hero-carousel');
            var dots = document.getElementById('hero-carousel-dots');
            if (!track || !dots || dots.children.length) return;
            var slides = track.children.length;
            for (var i = 0; i < slides; i++) {
                var d = document.createElement('button');
                d.type = 'button';
                d.className = 'hero-carousel__dot';
                d.setAttribute('aria-label', 'Slide ' + (i + 1));
                (function(idx) {
                    d.addEventListener('click', function() {
                        track.scrollTo({ left: idx * track.clientWidth, behavior: 'smooth' });
                    });
                })(i);
                dots.appendChild(d);
            }
            function current() {
                return Math.round(track.scrollLeft / track.clientWidth);
            }
            function sync() {
                var cur = current();
                var ds = dots.children;
                for (var j = 0; j < ds.length; j++) {
                    ds[j].classList.toggle('is-active', j === cur);
                }
            }
            track.addEventListener('scroll', function() {
                clearTimeout(track._t);
                track._t = setTimeout(sync, 80);
            });
            var timer = setInterval(function() {
                if (track.matches(':hover')) return;
                var next = (current() + 1) % slides;
                track.scrollTo({ left: next * track.clientWidth, behavior: 'smooth' });
            }, 4000);
            track.addEventListener('pointerdown', function() { clearInterval(timer); });
            sync();
        }
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', init);
        } else {
            init();
        }
    })();
    "#
    .to_string()
}
