# ADR-018: JSON-LD Extension (Breadcrumb, LearningResource, Organization sameAs, FAQ)

## Status

Accepted

## Date

2026-06-26

## Context

`origa_landing` already emits three Schema.org JSON-LD types: `SoftwareApplication`
and `Organization` on the home page, and a `HowTo` on `/features` (the vocabulary
pipeline). Search engines and rich-result eligibility favour a richer entity graph:

- **Navigation context.** A `BreadcrumbList` tells Google the page's position in
  the site hierarchy and powers the breadcrumb rich result in SERPs. Without it,
  Google infers a breadcrumb from page text, which is often wrong for a locale-
  prefixed site (it may surface `/ru/features` as a raw path segment).
- **Educational entity typing.** `SoftwareApplication` is generic. `LearningResource`
  is the schema.org type purpose-built for educational content and is what some
  discovery surfaces (Google for Education, library/EdTech aggregators) query
  for. It carries structured fields (`educationalLevel`, `teaches`, `audience`,
  `isAccessibleForFree`) that `SoftwareApplication` does not.
- **Entity connectivity.** The `Organization` block had no `sameAs` link, so
  knowledge panels could not connect the Origa entity to its GitHub source.
- **Long-tail Q&A.** Five high-intent queries (getting started, Anki
  comparison, offline mode, JLPT, interface languages) map naturally to a
  `FAQPage`. FAQ rich results were deprecated for most sites by Google in 2025
  but the schema is still consumed by other engines and voice assistants, and
  the visible Q&A improves the page independently of any rich result.

Google's `FAQPage` policy has a hard requirement that the schema's Q&A must
**match visible text on the page** — schema-only FAQ is a policy violation and
can trigger a manual action. This means the FAQ must have a single source of
truth (the `Content` struct) feeding both the JSON-LD and a rendered `<section>`.

A separate, absolute rule: **never fabricate `Review` or `AggregateRating`.**
Origa has no review system; inventing ratings is schema spam that risks a
manual penalty.

## Decision

Extend the JSON-LD surface with four additions, all driven by the existing
`Content`/`Locale` types:

### 1. `BreadcrumbList` (not on the home page)

`breadcrumb_schema(locale, path, current_name)` emits a two-item
`BreadcrumbList`: the locale-localised home label (`Home` / `Главная` / `홈` /
`Trang chủ`) at position 1, and the current page at position 2. Item URLs are
**slash-canonical** per ADR-011 — the home root keeps its trailing slash, locale
roots do not, and page paths never have one.

Wired on `/features`, `/compare`, `/content`, `/download`. Deliberately **not**
wired on the home page: the home page IS the breadcrumb root, and a self-
referential one-item `BreadcrumbList` is a schema error.

### 2. `LearningResource` on `/features`

`learning_resource_schema(locale)` emits a `LearningResource` with:

- `educationalLevel`: `["JLPT N5", "JLPT N4", "JLPT N3", "JLPT N2", "JLPT N1"]`
  (canonical English — these are the international level names)
- `learningResourceType`: `"Interactive Application"`
- `audience`: `EducationalAudience` with `EducationalRole: student`
- `isAccessibleForFree: true`
- `teaches`: vocabulary, kanji, grammar, listening — **localised per locale**
  (EN/RU/KO/VI). schema.org treats `teaches` as free text, not an enum, so a
  localised value improves per-locale SEO; it lives in the `Content` struct
  alongside the other locale copy.

`educationalLevel` and `learningResourceType` stay English across all locales
(enum-like / international standard), while `name`/`description`/`teaches`
carry the locale.

It joins (does not replace) the existing `HowTo` on `/features`.

### 3. `Organization.sameAs`

The `Organization` block gains `"sameAs":
["https://github.com/yurvon-screamo/origa"]`, connecting the entity to its
source repository for knowledge-graph consumers.

### 4. `FAQPage` with a visible Q&A mirror on `/features`

`faq_schema(locale, qas)` emits a `FAQPage` whose `mainEntity` is built from
`&[(&'static str, &'static str)]` question/answer pairs. The **same** `Content`
fields (`faq_q1..faq_q5`, `faq_a1..faq_a5`) also render a visible
`<section class="feat-faq">` on `/features`, so the schema and the page text
are identical by construction — satisfying Google's visible-content rule with a
single source of truth.

No `Review` or `AggregateRating` is emitted anywhere, ever.

### Ordering invariant

New schemas are appended **after** the existing ones on each page. This
preserves the `first_jsonld_block` ordering that existing tests rely on
(`SoftwareApplication` first on `/`, `HowTo` first on `/features`).

## Alternatives Considered

### A1: `BreadcrumbList` on the home page (single item)

Rejected. The home page is the breadcrumb root; a one-item list pointing to
itself is malformed and Google's validator flags it. Guarded by the
`home_has_no_breadcrumb` test.

### A2: `LearningResource` instead of `SoftwareApplication`

Considered. Rejected — `SoftwareApplication` is the better fit for the
download/install intent (it carries `applicationCategory`, `operatingSystem`).
`LearningResource` is added alongside it, not as a replacement, to surface the
educational metadata without losing the application metadata.

### A3: FAQ rich-result optimization (schema only, hidden text)

Rejected. Google's `FAQPage` policy requires the Q&A to be visible; schema-only
FAQ is a violation. The decision to render a real `<section>` makes the page
better for humans *and* keeps the schema compliant.

### A4: Fabricate `AggregateRating` for rich-result stars

Rejected, unconditionally. Origa has no review system. Inventing ratings is
schema spam and a manual-action risk. This is called out explicitly so no
future contributor adds one to chase a star rich result.

## Consequences

### Positive

- `/features`, `/compare`, `/content`, `/download` carry correct navigation
  context for crawlers and SERP breadcrumbs.
- Origa is typed as a `LearningResource` for EdTech/library discovery.
- The `Organization` entity connects to its GitHub source.
- `/features` answers five high-intent long-tail queries both visibly and in
  machine-readable form.
- The FAQ schema and visible text cannot drift — they share `Content` fields.

### Negative

- Adding a `Content` field requires updating all four locale files (compile-
  enforced) — the FAQ added 10 fields × 4 locales, a one-time cost.
- `BreadcrumbList` and `FAQPage` rich results were de-prioritized by Google in
  2025, so the SERP upside is smaller than it once was. The schema still pays
  for itself via other engines/assistants and via the visible Q&A's effect on
  the page itself.
- The `first_jsonld_block` ordering is now load-bearing across more pages; any
  future schema must be appended, not prepended, or the ordering tests break.

## References

- ADR-011: URL Canonicalization Policy — the slash-canonical URL forms that
  `breadcrumb_schema` item URLs must match.
- schema.org: `BreadcrumbList`, `LearningResource`, `Organization.sameAs`,
  `FAQPage`, `Question`, `Answer`.
- `origa_landing/src/components/seo.rs` — the schema builder functions.
- `origa_landing/tests/seo_meta.rs` — `find_jsonld_block_by_type` helper and
  the per-schema assertions.
