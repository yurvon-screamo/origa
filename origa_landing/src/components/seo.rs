use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::content::Locale;

const BASE_URL: &str = env!("ORIGA_LANDING_BASE_URL");

#[component]
pub fn SchemaOrg(json: String) -> impl IntoView {
    view! {
        <script type="application/ld+json" inner_html=json></script>
    }
}

pub fn software_application_schema() -> String {
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": "Origa",
        "applicationCategory": "EducationalApplication",
        "operatingSystem": "Windows, Linux, macOS, Android, Web",
        "description": "All-in-one Japanese learning app with vocabulary, kanji, grammar and native phrases.",
        "featureList": "Vocabulary, Kanji, Grammar, Listening, JLPT Analytics, Offline Mode",
        "inLanguage": ["en", "ru", "ko", "vi"]
    })
    .to_string()
}

pub fn organization_schema() -> String {
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Organization",
        "name": "Origa",
        "url": BASE_URL,
        "logo": format!("{BASE_URL}/favicon.png")
    })
    .to_string()
}

pub fn how_to_schema(name: &str, steps: &[&str]) -> String {
    let steps_json: Vec<_> = steps
        .iter()
        .enumerate()
        .map(|(i, step)| {
            serde_json::json!({
                "@type": "HowToStep",
                "position": i + 1,
                "text": step
            })
        })
        .collect();

    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "HowTo",
        "name": name,
        "step": steps_json
    })
    .to_string()
}

#[component]
pub fn PageMeta(
    locale: Locale,
    #[prop(default = "")] path: &'static str,
    title: &'static str,
    description: &'static str,
) -> impl IntoView {
    let img_prefix = locale.image_prefix();
    let og_image = format!("{BASE_URL}/images/{img_prefix}.hero.png");
    let canonical = format!("{BASE_URL}{}{path}", locale.path_prefix());

    view! {
        <Title text=title/>
        <Meta name="description" content=description/>
        <Meta property="og:title" content=title/>
        <Meta property="og:description" content=description/>
        <Meta property="og:image" content=og_image.clone()/>
        <Meta property="og:url" content=canonical.clone()/>
        <Meta property="og:type" content="website"/>
        <Meta property="og:locale" content=locale.og_locale()/>
        <Meta name="twitter:card" content="summary_large_image"/>
        <Meta name="twitter:title" content=title/>
        <Meta name="twitter:description" content=description/>
        <Meta name="twitter:image" content=og_image/>
        <link rel="canonical" href=canonical/>
        {Locale::ALL.iter().map(|loc| {
            let href = format!("{BASE_URL}{}{path}", loc.path_prefix());
            let hreflang = loc.as_str();
            view! { <link rel="alternate" hreflang=hreflang href=href/> }
        }).collect_view()}
        <link rel="alternate" hreflang="x-default" href=format!("{BASE_URL}{path}")/>
    }
}
