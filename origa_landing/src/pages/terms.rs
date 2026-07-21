use leptos::prelude::*;

use crate::components::seo::{PageMeta, SchemaOrg, breadcrumb_schema};
use crate::content::Locale;

const EFFECTIVE_DATE: &str = "2026-07-07";

#[component]
pub fn TermsPage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let c = locale.content();

    view! {
        <PageMeta locale path="/terms" title=c.terms_meta_title description=c.terms_meta_description/>
        <SchemaOrg json=breadcrumb_schema(locale, &[
            (c.breadcrumb_home, "/"),
            (c.legal_terms_link, "/terms"),
        ])/>

        <article class="legal-doc">
            <header class="legal-doc__header">
                <h1 class="legal-doc__title">{c.terms_h1}</h1>
                <p class="legal-doc__updated">
                    {c.terms_last_updated_label} " " {EFFECTIVE_DATE}
                </p>
            </header>
            <div class="legal-doc__body" inner_html=c.terms_body></div>
        </article>
    }
}
