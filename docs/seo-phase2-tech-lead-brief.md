# SEO Phase 2 — Technical Gaps vs kana-dojo (Tech Lead Brief)

> **From:** marketing (SEO strategy)
> **To:** Tech Lead
> **Date:** 2026-06-26
> **Context:** Phase 1 (content-level SEO) landed — keyword-targeted meta ×4 locales, VI «Kanji»→«Hán tự» sweep, README sync. See `marketing/strategies/origa-seo.md`. Technical SEO infra (ADR-007/011, JSON-LD, hreflang, sitemap) is already production-grade.
> **Scope:** This brief covers the **technical gaps** vs competitor `lingdojo/kana-dojo` (2.7k★, Vercel-backed) identified in `marketing/kana-report.md` + the kana-dojo deep-dive. These are backend/infra features, not copy — they need engineering.
> **Severity rationale:** Pre-launch, zero domain authority, 1 GitHub star. Bing/Yandex indexing was a multi-week fight (ADR-007/011). Anything that speeds indexing or broadens rich-results eligibility is high-value.

---

## Background — what's already done (don't redo)

| Area | Status | Where |
| --- | --- | --- |
| DNS / Cloudflare proxy / IPv4+IPv6 | ✅ | ADR-007 |
| URL canonicalization (no trailing slash, 308) / cache-control / soft-404 | ✅ | ADR-011 |
| `<html lang>` per-locale (leptos_meta) | ✅ | `layout.rs` |
| hreflang in head + sitemap + canonical + og:locale | ✅ | `components/seo.rs`, `sitemap.xml.tmpl` |
| JSON-LD: `SoftwareApplication`, `Organization`, `HowTo` (XSS-escaped) | ✅ | `components/seo.rs` |
| Regression tests for SEO (`seo_meta.rs`, `sitemap.rs`) | ✅ | `tests/` |

**Stack:** Rust + Leptos 0.8 SSR + Axum + tower-http + Tauri. Production domain `origa.uwuwu.net` behind Cloudflare → Railway origin. CDN: S3 (T3 Storage).

---

## Tasks (priority-ordered)

### P0 — IndexNow (Bing + Yandex instant indexing)

**Why:** IndexNow (indexnow.org) lets us ping Bing/Yandex/Seznam/Naver on content change → pages indexed in minutes instead of days. Directly attacks the "discovered, not indexed" problem from ADR-007/011. **Google does NOT use IndexNow** (it crawls on its own), but Bing/Yandex are our RU/VI/KO search engines.

**What to build:**

1. Generate an IndexNow key (32+ hex chars). Commit `<key>.txt` to `origa_landing/public/` (must be served at `https://origa.uwuwu.net/<key>.txt`).
2. A way to submit URLs. Two options — pick one:
   - **Build-time** (simplest): a script (Rust binary or Python in `scripts/`) that, after deploy, POSTs the 5 canonical URLs × 4 locales = 20 URLs to `https://api.indexnow.org/IndexNow` with the key. Wire into `docker.yml` / CD after successful deploy.
   - **Runtime**: an axum route `POST /api/indexnow` (auth-gated) that pings IndexNow. More flexible but more surface area.
3. Register the key in **Bing Webmaster Tools** and **Yandex Webmaster** (manual, one-time, by whoever holds the accounts).

**Reference (kana-dojo impl):** `/app/api/indexnow/route.ts`, `/shared/lib/indexnow.ts`, `/docs/INDEXNOW_SETUP.md`, `/TODO_INDEXNOW_SETUP.md`.

**Acceptance:**

- `https://origa.uwuwu.net/<key>.txt` returns 200 with the key as body.
- A POST to `api.indexnow.org/IndexNow` for `https://origa.uwuwu.net/` returns 200.
- Bing Webmaster shows "URL submitted via IndexNow" within 24h.

**Notes:** IndexNow key limit: 10,000 URLs/day per key (we're nowhere near). Submit only canonical URLs (no trailing slash — ADR-011). Don't submit on every build, only on content change (debounce).

---

### P1 — Security headers

**Why:** `X-Content-Type-Options`, `X-Frame-Options` (or CSP `frame-ancestors`), `Referrer-Policy`, `Permissions-Policy`. Bing counts these as trust signals; also just correct hygiene. Very low effort.

**What to build:** An axum middleware (alongside `enforce_cache_policy` / `strip_trailing_slash` from ADR-011) that stamps these on every response. Suggested values:

- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: strict-origin-when-cross-origin`
- `Permissions-Policy: camera=(), microphone=(), geolocation=()` (we have none; if OCR/STT ever need mic/camera in-browser, relax those)
- `X-Frame-Options: SAMEORIGIN` OR drop it in favour of CSP `frame-ancestors 'self'` if/when a CSP is added.

**Reference:** kana-dojo `next.config.ts` headers. Cloudflare can also set these (Transform Rules) — pick one place to avoid duplication; recommend axum so it's in version control.

**Acceptance:** `curl -sI https://origa.uwuwu.net/` shows all four headers. <https://securityheaders.com> grades A or better.

---

### P2 — `llms.txt` (Generative Engine Optimization)

**Why:** `llms.txt` (llmstxt.org) is an emerging convention: a markdown file at `/llms.txt` summarizing the product for AI assistants (ChatGPT, Perplexity, Claude, Gemini). When a user asks an LLM "what's a good app to learn Japanese offline?", a well-written `llms.txt` increases the chance Origa is cited. Forward-looking but cheap.

**What to build:**

1. `origa_landing/public/llms.txt` — concise markdown: what Origa is, the 4-locale native-language wedge, key features, JLPT/FSRS/offline/OCR, link to landing + GitHub. Keep it factual, brand-voice (see `marketing/strategies/origa-seo.md` §10 brand rules; `free` is now permitted, `open source`/`license`/`BSL` are not).
2. Add AI-crawler courtesy note to `robots.txt` (kana-dojo added an "AI-friendly content reference" block). Decide: do we ALLOW AI crawlers (GPTBot, ClaudeBot, PerplexityBot, Google-Extended)? **Recommend allow** — GEO is the upside; the ADR-011 Cloudflare "AI Audit"/"Pay Per Crawl" runbook already notes this is controlled at the CF layer, so confirm it's disabled there.

**Reference:** kana-dojo `llms.txt`, `public/robots.txt` AI block, `/docs/SEO_HEALTH_CHECK_PLAN.md`.

**Acceptance:** `https://origa.uwuwu.net/llms.txt` returns 200 markdown. `robots.txt` has an explicit, intentional stance on AI bots (allow or disallow — documented either way).

---

### P3 — `browserconfig.xml` + IE/Edge tile meta

**Why:** Windows/Edge tile config. Minor Bing/Edge preference, near-zero effort.

**What to build:** `origa_landing/public/browserconfig.xml` referencing existing favicon assets (`favicon.png`, `apple-touch-icon.png`). Add `<meta name="msapplication-TileColor" content="#3d4535">` + `<meta name="msapplication-config" content="/browserconfig.xml">` to the shell `<head>` in `app.rs`.

**Reference:** kana-dojo `public/browserconfig.xml`.

**Acceptance:** `/browserconfig.xml` 200 + valid XML; meta tags present in SSR HTML.

---

### P4 — Meta keywords (Bing-specific, per-locale)

**Why:** Google ignores `<meta name="keywords">`; **Bing still reads it** as a weak signal. Harmless for Google. With our VI/RU/KO markets on Bing/Yandex/Naver, worth the small effort. This needs **per-locale keyword data** — already collected in `docs/keyword-research-report.md`.

**What to build:**

1. Add a `keywords: &'static str` field to the `Content` struct in `origa_landing/src/content/mod.rs`.
2. Populate per-locale (en/ru/ko/vi.rs) using `docs/keyword-research-report.md` — ~10-15 keywords each, comma-separated, localized to the market's actual search terms (e.g. VI must include `hán tự`, RU `кандзи`/`на русском`, KO `일본어 공부`/`입문`).
3. Add `<Meta name="keywords" content=c.keywords/>` to `PageMeta` in `components/seo.rs`. (Optional: per-page keywords — start with a single site-level set per locale, refine later.)
4. Add a test in `tests/seo_meta.rs` asserting keywords meta is present and localized (mirror the existing locale-assertion pattern).

**Reference:** kana-dojo `core/i18n/locales/*/metadata.json` (40+ keywords/page). Source data: `docs/keyword-research-report.md`.

**Acceptance:** Every page has localized `<meta name="keywords">`; test passes; no English keywords leaking into VI/KO/RU sets.

---

### P5 — Richer JSON-LD (broaden rich-results eligibility)

**Why:** We currently emit `SoftwareApplication` + `Organization` + `HowTo`. Adding more schema types increases eligibility for rich results and strengthens E-E-A-T (Experience, Expertise, Authoritativeness, Trustworthiness) signals Bing/Google weigh for educational content. Highest effort — needs real content, not just plumbing.

**What to build (extend `components/seo.rs`):**

- **`BreadcrumbList`** — on all non-home pages (`features` > home, etc.). Cheap, consistent nav signal.
- **`FAQPage`** — on `features` and/or a new `/faq` route. Needs ~5-8 Q&A per locale answering real learner questions from keyword-research long-tails (e.g. EN "how to start kanji", RU "кандзи это", VI "học kanji bắt đầu từ đâu", KO "일본어 공부 순서"). **This is partly a content task** — coordinate with marketing for the Q&A copy.
- **`LearningResource`** — on `features` (educationalContent, educationalLevel = JLPT N5-N1, learningResourceType).
- **`Course`** — optional, if we frame JLPT prep as a course (skillLevel, isAccessibleForFree=true).
- **`Review`/`AggregateRating`** — only when we have real ratings (do NOT fabricate — Google penalizes fake review schema).
- **`Video`** — only if we add demo videos later.
- Enrich existing `Organization`: add `sameAs: ["https://github.com/yurvon-screamo/origa"]`, and `logo`/`slogan` if applicable.

**Reference:** kana-dojo schemas: `CourseSchema`, `AuthorSchema`, `LearningResourceSchema`, `VideoSchema`, `FAQSchema`, `BreadcrumbSchema` (see `SEO_IMPROVEMENTS_SUMMARY.md` Phase 2).

**Acceptance:** Each schema validates in Google Rich Results Test (`search.google.com/test/rich-results`). No warnings about missing required fields. FAQ Q&A is localized and answers real search queries (cross-check `docs/keyword-research-report.md` long-tails).

---

## Suggested order (impact / effort)

1. **P1 Security headers** (1-2h, quick win, hygiene)
2. **P3 browserconfig.xml** (30min, trivial)
3. **P2 llms.txt** (2-3h, GEO future-proofing)
4. **P0 IndexNow** (half day — key gen + submit script + Bing/Yandex Webmaster registration by account holder)
5. **P4 Meta keywords** (half day — struct change + 4 localized keyword sets + test)
6. **P5 Richer JSON-LD** (1-2 days — FAQ content + schema design + per-page wiring)

---

## Constraints / gotchas

- **Brand rules** (`docs/landing-content-plan.md` §10, updated 2026-06-26): landing copy may NOT mention pricing, open source, license, BSL. `free` is now permitted. `llms.txt` and FAQ copy must respect this.
- **Test constraints** (`tests/seo_meta.rs`): RU home description must contain `лексика`/`кандзи`; KO featureList `한자`; VI HowTo name `tiếng Nhật`; default `<title>` unchanged. Any new content must keep these green.
- **Canonical URLs** (ADR-011): no trailing slash everywhere. IndexNow submissions, sitemap, hreflang must all agree.
- **Cache policy** (ADR-011): `llms.txt`, `browserconfig.xml` are static → immutable cache (`public, max-age=31536000`); new static files need an explicit `route_service` entry to get the right header (see ADR-011 negative-consequences note).
- **Cloudflare AI Audit** (ADR-011 runbook): if AI crawlers behave oddly, check the CF "AI Audit"/"Pay Per Crawl" panel — it can inject a managed `robots.txt` block. Document the intended stance so we don't re-enable it by accident.
- **Disk space:** the `D:` build volume was nearly full (LNK1140 on full test link). `cargo check` is fine; if linking many test binaries, clear stale `.rcgu.o` / `target/` first.

---

## Deliverables checklist

- [ ] IndexNow key file + submit mechanism + Bing/Yandex registered
- [ ] Security headers live (securityheaders.com grade A)
- [ ] `llms.txt` published + robots.txt AI stance documented
- [ ] `browserconfig.xml` + Edge tile meta
- [ ] Per-locale `<meta name="keywords">` + test
- [ ] Extended JSON-LD (Breadcrumb, FAQ, LearningResource minimum) + Rich Results Test passes
- [ ] ADR(s) for any non-trivial decision (follow the existing `docs/decisions/ADR-NNN-*.md` pattern)

## References

- Strategy: `marketing/strategies/origa-seo.md`
- Keyword data: `docs/keyword-research-report.md`
- Competitor research: `marketing/kana-report.md` + kana-dojo `SEO_IMPROVEMENTS_SUMMARY.md`
- Prior SEO decisions: `docs/decisions/ADR-007` (DNS), `ADR-011` (canonicalization), `ADR-013` (favicon), `ADR-014` (sitemap lastmod)
- IndexNow spec: <https://www.indexnow.org/>
- llms.txt spec: <https://llmstxt.org/>
- Schema.org validator: <https://validator.schema.org/> / <https://search.google.com/test/rich-results>
