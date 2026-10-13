//! HowTo JSON-LD for articles that document a step-by-step workflow.
//!
//! Ships today for exactly one slug: `japanese-ocr-app` (issue #732 — the
//! OCR cluster's top-10 Google page). The steps are a localized compression
//! of the pipeline the article itself documents — the "Learning-integrated
//! OCR" section, the "How Origa handles this" bullet list, and the "Origa"
//! row of the "The tools at a glance" table — so the schema makes no claims
//! the visible page does not: photograph a page, on-device NDLOCR-Lite
//! recognition, each word becomes a flashcard with reading/translation/
//! audio, FSRS-scheduled review. Unlike FAQ (single source in the rendered
//! HTML), these steps are hand-maintained: when the article's pipeline
//! description changes, the `blog_ocr_howto_*` content strings must be
//! updated in the same edit.
//!
//! Google deprecated HowTo rich results (mobile August 2023, desktop
//! September 2023), so no step-snippets will appear in Google SERPs. The
//! markup stays for the remaining consumers — Bing and AI/LLM crawlers, for
//! which the landing is the primary machine-readable surface — mirroring the
//! FAQPage precedent (kept after the May 2026 FAQ rich-result deprecation).
//! Google states unused structured data does not affect Search.

use crate::components::seo::how_to_schema;
use crate::content::Locale;

/// The only slug that ships a HowTo schema today.
const OCR_HOWTO_SLUG: &str = "japanese-ocr-app";

/// Localized `(name, steps)` for a slug, or `None` when the article does not
/// ship a HowTo schema. Steps are ordered; `how_to_schema` numbers them.
fn content(locale: Locale, slug: &str) -> Option<(&'static str, [&'static str; 4])> {
    if slug != OCR_HOWTO_SLUG {
        return None;
    }
    let c = locale.content();
    Some((
        c.blog_ocr_howto_name,
        [
            c.blog_ocr_howto_step1,
            c.blog_ocr_howto_step2,
            c.blog_ocr_howto_step3,
            c.blog_ocr_howto_step4,
        ],
    ))
}

/// Render the HowTo JSON-LD for a blog article, or `None` for slugs without
/// HowTo content (the caller skips the `<SchemaOrg>` block entirely). The
/// locale must be the article's canonical locale, matching the Article and
/// BreadcrumbList schemas on fallback renders.
pub fn schema_for_slug(locale: Locale, slug: &str) -> Option<String> {
    let (name, steps) = content(locale, slug)?;
    Some(how_to_schema(locale, name, &steps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_for_slugs_without_how_to() {
        assert!(schema_for_slug(Locale::En, "anki-alternative-japanese").is_none());
        assert!(schema_for_slug(Locale::En, "does-not-exist").is_none());
    }

    #[test]
    fn returns_schema_for_ocr_slug_in_every_locale() {
        for locale in Locale::ALL {
            let json = schema_for_slug(*locale, OCR_HOWTO_SLUG)
                .unwrap_or_else(|| panic!("HowTo schema missing for {locale:?}"));
            assert!(
                json.contains(r#""@type":"HowTo""#),
                "HowTo type missing for {locale:?}: {json}"
            );
            assert!(
                json.contains("NDLOCR-Lite"),
                "step 2 must name the on-device model for {locale:?}: {json}"
            );
        }
    }
}
