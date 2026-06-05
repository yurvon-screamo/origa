# Origa Landing — Technical Architecture Plan

> **Status:** Draft (post code-quality-review)  
> **Crate:** `origa_landing`  
> **Scope:** 6 pages × 2 languages = 12 localized pages  
> **Prerequisite:** Content plan at `docs/landing-content-plan.md`

---

## ADR-1: Pure SSR without client-side WASM

**Decision:** The landing site renders HTML on the server via Axum + Leptos SSR. No WASM is sent to the browser.

**Rationale:**

- Landing pages are static content — no reactive state, no interactive components
- All visual effects achieved via CSS (hover, transitions, animations, paper texture)
- Zero WASM bundle → instant page load → perfect Core Web Vitals
- Simpler build pipeline (native binary only, no WASM compilation)
- Search engines receive complete HTML with all content

**Consequences:**

- ✅ Optimal SEO (pre-rendered HTML)
- ✅ Minimal client-side resources (CSS only)
- ✅ Simple deployment (single binary + static assets)
- ❌ No client-side navigation (full page reloads between pages)
- ❌ No interactive components that require JavaScript

**Future optimization:** SSG (Static Site Generation) — pre-render at build time, deploy static HTML to CDN. Leptos supports this via `generate_route_list_with_ssg`. This eliminates the need for a running server. Can be added as Phase 2 without architectural changes.

---

## ADR-2: CSS reuse via symlink, new SSR-safe Rust components

**Decision:** Share CSS by symlinking `input.css` from `origa_ui`. Create new SSR-safe Leptos components that use the same CSS class names.

**Rationale:**

- `origa_ui` components depend on browser-only APIs (web_sys, gloo, idb, wasm-bindgen) — cannot be imported in SSR context
- CSS class names (.btn, .card, .heading-h1, .tag, etc.) ARE the design system contract
- Rust components are just HTML generators — creating new ones that output the same classes is NOT duplication
- Landing-specific components (hero, comparison table, CTA) are different from app components anyway
- Symlink ensures CSS stays in sync; landing-specific overrides go in separate file

**CSS structure:**

```
origa_landing/style/
├── input.css         ← SYMLINK to ../../origa_ui/input.css
└── landing.css       ← Landing-specific overrides and trim
```

**Build process:** Tailwind CLI processes `input.css` + scans landing source files. Landing-specific `landing.css` adds only what's needed (hero section, comparison table, etc.) and can use `@import "input.css"` to include the full design system.

**Maintenance:** When `origa_ui/input.css` changes, symlink automatically picks up changes. Build script (`build.rs`) verifies symlink integrity.

---

## ADR-3: Simple enum-based i18n (no leptos_i18n)

**Decision:** Use a `Locale` enum with `&'static str` content constants. No runtime i18n framework.

**Rationale:**

- Landing content is hand-written and static — no need for compile-time key generation
- `leptos_i18n` adds build complexity (leptos_i18n_build, YAML files)
- Enum approach is type-safe, compile-time checked, zero-cost
- Easy to add new languages: just add a new variant and content module

---

## Crate Structure

```
origa_landing/                        ← NEW workspace member
├── Cargo.toml                        ← features: ssr only
├── build.rs                          ← CSS processing + symlink verification
├── src/
│   ├── main.rs                       ← Axum server (#[cfg(feature = "ssr")])
│   ├── lib.rs                        ← Public exports
│   ├── app.rs                        ← App component + shell() + router
│   ├── state.rs                      ← Locale parsing from URL :lang param
│   ├── pages/
│   │   ├── mod.rs
│   │   ├── home.rs                   ← Homepage (7 blocks)
│   │   ├── features.rs               ← Features detail (4 sections)
│   │   ├── compare.rs                ← Comparison table + per-competitor
│   │   ├── download.rs               ← Platform download cards
│   │   ├── privacy.rs                ← Privacy Policy (legal)
│   │   └── terms.rs                  ← Terms of Service (legal)
│   ├── components/
│   │   ├── mod.rs
│   │   ├── layout.rs                 ← Header, Footer, PageShell
│   │   ├── hero.rs                   ← Hero section
│   │   ├── feature_card.rs           ← Feature preview card
│   │   ├── compare_table.rs          ← Comparison table component
│   │   ├── download_card.rs          ← Platform download card
│   │   ├── cta_section.rs            ← Call-to-action block
│   │   ├── principle_item.rs         ← Principle bullet item
│   │   └── seo.rs                    ← Schema.org JSON-LD injection
│   └── content/
│       ├── mod.rs                    ← Locale enum + Content struct + accessors
│       ├── en.rs                     ← EN content (pub static Content)
│       └── ru.rs                     ← RU content (pub static Content)
├── style/
│   ├── input.css                     ← SYMLINK to ../../origa_ui/input.css
│   └── landing.css                   ← Landing-specific styles
├── public/
│   ├── favicon.png                   ← Copied from origa_ui/public/
│   ├── og-image.png                  ← NEW: 1200×630 OG image
│   └── robots.txt                    ← Static
├── tailwind.config.js                ← Copy from origa_ui, update content paths
└── deploy/
    └── Dockerfile                    ← Multi-stage build
```

---

## Cargo.toml

```toml
[package]
name = "origa_landing"
version.workspace = true
edition.workspace = true
authors.workspace = true
license-file.workspace = true

[[bin]]
name = "origa_landing"
path = "src/main.rs"

[lib]
crate-type = ["rlib"]

[dependencies]
# Leptos SSR
leptos = { version = "0.8", features = ["ssr"] }
leptos_meta = { version = "0.8", features = ["ssr"] }
leptos_router = { version = "0.8", features = ["ssr"] }
leptos_axum = "0.8"

# Server
axum = "0.8"
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
tower-http = { version = "0.6", features = ["fs"] }

# Utilities
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[features]
default = ["ssr"]
ssr = []

[build-dependencies]
# Tailwind CSS processing via build script if needed
```

## Workspace Integration

Add to root `Cargo.toml`:

```toml
[workspace]
members = ["origa", "utils", "origa_ui", "tauri", "origa_landing"]
```

---

## Router Design

```rust
// app.rs

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    
    view! {
        <Router>
            <Routes fallback=|| view! { <NotFound/> }>
                // EN default (root, no prefix)
                <ParentRoute path=path!("/") view=move || view! { <Layout locale=Locale::En/> }>
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/features") view=FeaturesPage/>
                    <Route path=path!("/compare") view=ComparePage/>
                    <Route path=path!("/download") view=DownloadPage/>
                    <Route path=path!("/privacy") view=PrivacyPage/>
                    <Route path=path!("/terms") view=TermsPage/>
                </ParentRoute>
                // RU (prefixed)
                <ParentRoute path=path!("/ru") view=move || view! { <Layout locale=Locale::Ru/> }>
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/features") view=FeaturesPage/>
                    <Route path=path!("/compare") view=ComparePage/>
                    <Route path=path!("/download") view=DownloadPage/>
                    <Route path=path!("/privacy") view=PrivacyPage/>
                    <Route path=path!("/terms") view=TermsPage/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
```

Each page component reads `Locale` from context (provided by `Layout`).

---

## shell() — HTML Shell (Pure SSR)

```rust
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="theme-color" content="#3d4535"/>
                <link rel="stylesheet" href="/style.css"/>
                <link rel="icon" type="image/png" href="/favicon.png"/>
                <MetaTags/>
            </head>
            <body class="min-h-screen paper-texture">
                <App/>
            </body>
        </html>
    }
}
```

Key: No `<AutoReload>`, no `<HydrationScripts>`. Pure HTML output.

---

## i18n Implementation

```rust
// content/mod.rs

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Locale {
    En,
    Ru,
}

pub struct LandingContent {
    // Homepage
    pub hero_title: &'static str,
    pub hero_subtitle: &'static str,
    pub hero_cta_primary: &'static str,
    pub hero_cta_secondary: &'static str,
    pub problem_h2: &'static str,
    pub problem_text: &'static str,
    pub features_h2: &'static str,
    // ... all text content for all pages
    
    // Meta
    pub meta_title: &'static str,
    pub meta_description: &'static str,
    pub html_lang: &'static str,
    pub og_locale: &'static str,
}

impl Locale {
    pub fn content(&self) -> &'static LandingContent {
        match self {
            Locale::En => &en::CONTENT,
            Locale::Ru => &ru::CONTENT,
        }
    }
    
    pub fn path_prefix(&self) -> &'static str {
        match self {
            Locale::En => "",
            Locale::Ru => "/ru",
        }
    }
}
```

```rust
// content/en.rs

pub static CONTENT: LandingContent = LandingContent {
    hero_title: "Learn Japanese in your own language",
    hero_subtitle: "Vocabulary, kanji, grammar, listening and 200,000+ native phrases — all in one app. No English required.",
    hero_cta_primary: "Get started",
    hero_cta_secondary: "Open web app",
    // ...
    meta_title: "Origa — All-in-One Japanese Learning App",
    meta_description: "All-in-one Japanese learning app: vocabulary, kanji, grammar, 200K+ native phrases and JLPT analytics. In your language.",
    html_lang: "en",
    og_locale: "en_US",
};
```

---

## SEO Components

### Schema.org JSON-LD (seo.rs)

```rust
#[component]
pub fn SoftwareApplicationSchema() -> impl IntoView {
    let schema = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": "Origa",
        "applicationCategory": "EducationalApplication",
        "operatingSystem": "Windows, Linux, macOS, Android, Web",
        "description": "All-in-one Japanese learning app with vocabulary, kanji, grammar and 200K+ native phrases.",
        "featureList": "Vocabulary, Kanji, Grammar, Listening, JLPT Analytics, Offline Mode",
        "inLanguage": ["en", "ru"]
    });
    
    view! {
        <script type="application/ld+json">{schema.to_string()}</script>
    }
}
```

### Page Meta Tags

Each page component uses `leptos_meta`:

```rust
#[component]
pub fn HomePage() -> impl IntoView {
    let locale = use_context::<Locale>().unwrap_or(Locale::En);
    let content = locale.content();
    
    view! {
        <Title text=content.meta_title/>
        <Meta name="description" content=content.meta_description/>
        <Meta name="og:title" content=content.meta_title/>
        <Meta name="og:description" content=content.meta_description/>
        // ... canonical, hreflang, OG tags
        <SoftwareApplicationSchema/>
        // Page content...
    }
}
```

---

## Build Pipeline

### Development

```powershell
# Terminal 1: Watch CSS
npx tailwindcss --input style/landing.css --output target/landing.css --watch

# Terminal 2: Run server
cargo run --bin origa_landing
```

### Production Build

```powershell
# 1. Process CSS
npx tailwindcss --input style/landing.css --output target/landing.css --minify

# 2. Build binary
cargo build --release --bin origa_landing

# 3. Deploy binary + static assets
```

### Docker

```dockerfile
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN npx tailwindcss --input style/landing.css --output target/landing.css --minify
RUN cargo build --release --bin origa_landing

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/origa_landing /app/origa_landing
COPY --from=builder /app/target/landing.css /app/style.css
COPY --from=builder /app/public /app/public
EXPOSE 3000
CMD ["/app/origa_landing"]
```

---

## Verification Strategy

- **SEO:** Lighthouse SEO score ≥ 90
- **Performance:** LCP < 2.5s, CLS < 0.1, INP < 200ms
- **Meta tags:** All titles ≤ 50 chars, descriptions ≤ 125 chars
- **hreflang:** Validated via Google International Targeting report
- **sitemap.xml:** Submitted to Google Search Console
- **robots.txt:** Accessible at /robots.txt
- **Semantic HTML:** header, main, section, footer on every page
- **Images:** All have alt attributes with keywords
- **CSS:** Shared with origa_ui, verified via symlink

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| CSS symlink breaks on Windows | Medium | build.rs verifies symlink; fallback to copy |
| leptos_router SSR edge cases | Low | Test all 12 routes render correctly |
| Tailwind CLI version mismatch | Low | Pin tailwindcss version in package.json |
| Server downtime | Medium | Docker health check + Railway auto-restart |
| Content updates require rebuild | Low | Content is compile-time; acceptable for landing |

---

## Implementation Order (Slices)

### Phase 1: Foundation

1. **Scaffold crate** — Cargo.toml, main.rs, shell(), basic Axum server
2. **CSS pipeline** — symlink + Tailwind + build.rs
3. **i18n system** — Locale enum + content structs (EN only initially)

### Phase 2: Pages (EN)

4. **Layout** — Header, Footer, navigation
2. **Homepage** — All 7 blocks
3. **Features page** — 4 feature sections
4. **Compare page** — Table + per-competitor sections
5. **Download page** — Platform cards

### Phase 3: RU + Legal

9. **RU content** — All pages
2. **Privacy + Terms** — Legal pages (both languages)

### Phase 4: SEO + Deploy

11. **SEO infrastructure** — Schema.org, meta tags, hreflang, canonical
2. **robots.txt + sitemap.xml**
3. **Docker + deployment**

### Checkpoint: Ready

- [ ] All 12 URLs render correct HTML
- [ ] Lighthouse SEO ≥ 90
- [ ] Core Web Vitals green
- [ ] hreflang validated
- [ ] CSS consistent with origa_ui design system
