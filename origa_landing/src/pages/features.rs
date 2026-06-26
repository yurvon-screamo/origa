use leptos::prelude::*;

use crate::components::cta::CtaSection;
use crate::components::seo::{
    PageMeta, SchemaOrg, breadcrumb_schema, faq_schema, how_to_schema, learning_resource_schema,
};
use crate::content::Locale;

#[component]
pub fn FeaturesPage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let c = locale.content();
    let prefix = locale.path_prefix();
    let lang = locale.as_str();

    let download_href = format!("{prefix}/download");
    let all_in_one_image = format!("/images/{lang}.all_in_one.png");

    let vocab_steps = [
        c.features_vocab_step1,
        c.features_vocab_step2,
        c.features_vocab_step3,
    ];

    let vocab_tags = [
        c.features_vocab_dict,
        c.features_vocab_cards,
        c.features_vocab_audio,
        c.features_vocab_import,
        c.features_vocab_fsrs,
        c.features_vocab_jlpt,
    ];

    view! {
        <PageMeta locale path="/features" title=c.features_meta_title description=c.features_meta_description/>
        <SchemaOrg json=how_to_schema(locale, c.features_schema_how_to_name, &vocab_steps)/>
        <SchemaOrg json=breadcrumb_schema(locale, "/features", c.header_features)/>
        <SchemaOrg json=learning_resource_schema(locale)/>
        <SchemaOrg json=faq_schema(locale, &[
            (c.faq_q1, c.faq_a1),
            (c.faq_q2, c.faq_a2),
            (c.faq_q3, c.faq_a3),
            (c.faq_q4, c.faq_a4),
            (c.faq_q5, c.faq_a5),
        ])/>

        // Section 0: Hero
        <section class="feat-hero">
            <div>
                <h1 class="feat-hero__title">{c.features_h1}</h1>
                <hr class="feat-hero__rule" />
            </div>
            <div class="feat-hero__decor">
                <div
                    class="feat-hero__decor-img"
                    aria-hidden="true"
                    style=format!("background-image: url({all_in_one_image})")
                ></div>
            </div>
        </section>

        // Section 1: Vocabulary Pipeline
        <section class="feat-vocab">
            <div class="feat-vocab__inner">
                <p class="feat-vocab__label">{c.features_vocab_label}</p>
                <h2 class="feat-vocab__title">{c.features_vocab_h2}</h2>

                <div class="feat-pipeline">
                    <VocabStep number="01" text=c.features_vocab_step1 />
                    <div class="feat-pipeline__arrow"></div>
                    <VocabStep number="02" text=c.features_vocab_step2 />
                    <div class="feat-pipeline__arrow"></div>
                    <VocabStep number="03" text=c.features_vocab_step3 />
                </div>

                <div class="feat-tag-bar">
                    {vocab_tags
                        .into_iter()
                        .enumerate()
                        .map(|(i, tag)| {
                            view! {
                                <span>{tag}</span>
                                {if i < vocab_tags.len() - 1 {
                                    view! { <span>" · "</span> }.into_any()
                                } else {
                                    ().into_any()
                                }}
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
            </div>
        </section>

        // Section 2: Kanji (no image)
        <section class="feat-kanji">
            <div class="feat-kanji__inner">
                <div class="feat-kanji__content">
                    <h2>{c.features_kanji_h2}</h2>
                    <p class="feat-kanji__subtitle">{c.features_kanji_subtitle}</p>

                    <div class="feat-capabilities">
                        <FeatCapability title=c.features_kanji_furigana desc=c.features_kanji_furigana_desc />
                        <FeatCapability title=c.features_kanji_writing desc=c.features_kanji_writing_desc />
                        <FeatCapability title=c.features_kanji_dict desc=c.features_kanji_dict_desc />
                        <FeatCapability title=c.features_kanji_tests desc=c.features_kanji_tests_desc />
                    </div>

                    <FeatEditorialNote text=c.features_kanji_insight />
                </div>
            </div>
        </section>

        // Section 3: Grammar (no bg image)
        <section class="feat-grammar">
            <div class="feat-grammar__inner">
                <h2>{c.features_grammar_h2}</h2>
                <p class="feat-grammar__subtitle">{c.features_grammar_subtitle}</p>

                <div class="feat-capabilities">
                    <FeatCapability title=c.features_grammar_jlpt desc=c.features_grammar_jlpt_desc />
                    <FeatCapability title=c.features_grammar_context desc=c.features_grammar_context_desc />
                    <FeatCapability title=c.features_grammar_tests desc=c.features_grammar_tests_desc />
                    <FeatCapability title=c.features_grammar_search desc=c.features_grammar_search_desc />
                </div>

                <FeatEditorialNote text=c.features_grammar_insight />
            </div>
        </section>

        // Section 4: Listening (no bg image)
        <section class="feat-listening">
            <div class="feat-listening__inner">
                <h2>{c.features_listening_h2}</h2>
                <p class="feat-listening__subtitle">{c.features_listening_subtitle}</p>

                <div class="feat-listening__band">
                    <div class="feat-capabilities">
                        <FeatCapability title=c.features_listening_n1 desc=c.features_listening_n1_desc />
                        <FeatCapability title=c.features_listening_audio desc=c.features_listening_audio_desc />
                        <FeatCapability title=c.features_listening_comp desc=c.features_listening_comp_desc />
                        <FeatCapability title=c.features_listening_everyday desc=c.features_listening_everyday_desc />
                    </div>
                    <FeatEditorialNote text=c.features_listening_insight />
                </div>
            </div>
        </section>

        // Section 5: FAQ (visible Q&A mirrors the FAQPage JSON-LD 1:1)
        <section class="feat-faq">
            <div class="feat-faq__inner">
                <h2>{c.features_faq_h2}</h2>
                <div class="feat-faq__list">
                    <FaqItem question=c.faq_q1 answer=c.faq_a1 />
                    <FaqItem question=c.faq_q2 answer=c.faq_a2 />
                    <FaqItem question=c.faq_q3 answer=c.faq_a3 />
                    <FaqItem question=c.faq_q4 answer=c.faq_a4 />
                    <FaqItem question=c.faq_q5 answer=c.faq_a5 />
                </div>
            </div>
        </section>

        // Section 6: CTA (reuses home-cta dark olive)
        <CtaSection title=c.home_cta_title button_text=c.features_cta download_href=download_href />
    }
}

#[component]
fn VocabStep(number: &'static str, text: &'static str) -> impl IntoView {
    view! {
        <div class="feat-step">
            <p class="feat-step__number">{number}</p>
            <p class="feat-step__text">{text}</p>
        </div>
    }
}

#[component]
fn FeatCapability(title: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="feat-capability">
            <p class="feat-capability__title">{title}</p>
            <p class="feat-capability__desc">{desc}</p>
        </div>
    }
}

#[component]
fn FeatEditorialNote(text: &'static str) -> impl IntoView {
    view! {
        <div class="feat-editorial-note">
            <p class="feat-editorial-note__text">{text}</p>
        </div>
    }
}

#[component]
fn FaqItem(question: &'static str, answer: &'static str) -> impl IntoView {
    view! {
        <div class="feat-faq__item">
            <h3 class="feat-faq__question">{question}</h3>
            <p class="feat-faq__answer">{answer}</p>
        </div>
    }
}
