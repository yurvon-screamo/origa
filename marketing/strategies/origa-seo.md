# Origa — SEO Strategy

> **Status:** Draft → HUMAN GATE
> **Date:** 2026-06-26
> **Stage:** Pre-launch, zero domain authority, foundational SEO
> **Scope:** 4 markets (EN / RU / VI / KO), repo + README + landing content
> **Sources:** `docs/keyword-research-report.md`, `marketing/kana-report.md`, kana-dojo deep-dive, ADR-007/011/013/014, `docs/landing-content-plan.md`, landing source (`origa_landing/`)

---

## 1. Executive Summary

Origa's **technical SEO infrastructure is production-grade** — the team already shipped DNS fix (ADR-007), URL canonicalization + cache-control + soft-404 (ADR-011), localized JSON-LD, hreflang, sitemap with `lastmod`, favicon strategy, and OG tags, all covered by regression tests (`tests/seo_meta.rs`, `tests/sitemap.rs`). PRs #177/#184/#185/#186 landed 2026-06-22..24.

The remaining gap is **content-level keyword optimization per market**. Current meta is grammatically correct and brand-consistent but does not target actual search intent — most critically, the Vietnamese locale uses "Kanji" where Vietnamese learners search "Hán tự", and RU/KO miss high-intent zero-competition modifiers ("на русском", "с нуля", "입문", "공부법").

This strategy defines: (a) per-market keyword targets, (b) concrete content rewrites that respect brand red lines and test constraints, (c) repo-meta / README SEO, and (d) gaps vs competitor `kana-dojo` (2.7k★, Vercel-backed) that are out of scope here but tracked for future phases.

**Primary wedge (cross-market):** Origa is the only all-in-one Japanese learning app with **native-language interfaces in Russian, Vietnamese and Korean** — markets where Anki / WaniKani / Bunpro / kana-dojo are English-only. This is the defensible SEO position for a zero-DA pre-launch site.

---

## 2. Market & Product Context

- **Product:** Origa — all-in-one Japanese learning (vocabulary, kanji, grammar, listening, 200K+ native phrases, JLPT prep). FSRS, OCR/STT, offline-first, local AI, content-driven learning. Stack: Rust + Leptos 0.8 + Tauri v2.
- **Stage:** Pre-launch. GitHub: 1 star, 0 forks, 0 backlinks. Domain `origa.uwuwu.net` behind Cloudflare (ADR-007), indexable by Google/Yandex/Bing.
- **Constraint:** Zero domain authority → cannot compete for head terms ("learn japanese", "японский язык", "học tiếng nhật", "일본어"). Must win **long-tail + branded-adjacent + wedge keywords** where intent matches Origa's unique features.

### Why these 4 markets (and not others)

| Market | Why it matters for Japanese learning | Origa's localization |
| --- | --- | --- |
| **EN (global/US/EU)** | Largest learner base; saturated; Reddit-driven discovery | `Locale::En` (root `/`) |
| **RU (CIS)** | High demand, few quality RU-UI apps; "на русском" = zero competition | `Locale::Ru` (`/ru`) |
| **VI (Vietnam)** | **Huge** — many Vietnamese work/study in Japan (tokutei ginou); JLPT = visa requirement | `Locale::Vi` (`/vi`) |
| **KO (Korea)** | Major market; textbook/PDF culture; DC Inside = community hub; Naver ~50% search share | `Locale::Ko` (`/ko`) |

Markets NOT covered (ID, ES) — explicit per user scope. Existing `README` mentions them as "planned"; this is desynced from the live landing (which has VI/KO) and will be corrected.

---

## 3. Current SEO State Assessment

### Technical — elite level (NO changes needed)

| Area | Status | Evidence |
| --- | --- | --- |
| DNS / indexing | ✅ Fixed | ADR-007: Cloudflare proxy, IPv4+IPv6, Yandex/Bing crawl 200 |
| URL canonicalization | ✅ Done | ADR-011: no trailing slash, 308 redirect, soft-404 with body |
| Cache-Control | ✅ Done | ADR-011: HTML 5min, static immutable, robots/sitemap no-cache |
| `<html lang>` per-locale | ✅ Done | `layout.rs`: `<Html lang=locale.as_str() />` via leptos_meta |
| hreflang (head + sitemap) | ✅ Done | `seo.rs` + `sitemap.xml.tmpl`: 4 locales + x-default |
| Canonical per-locale | ✅ Done | `seo.rs`: `<link rel="canonical">` per page |
| JSON-LD | ✅ Done | `SoftwareApplication` + `Organization` + `HowTo` (XSS-escaped) |
| OG / Twitter | ✅ Done | `seo.rs`: og:locale + og:locale:alternate, twitter:card |
| Site verification | ✅ Done | Google / Yandex / Bing in `app.rs` |
| Regression tests | ✅ Done | `tests/seo_meta.rs` (12 cases), `tests/sitemap.rs` (4 cases) |

### Content — needs optimization (THIS strategy's scope)

- Meta titles/descriptions are grammatically correct but **do not front-load search keywords**.
- **VI locale uses "Kanji"** — Vietnamese learners search "Hán tự". Direct SEO miss.
- RU/KO miss zero-competition intent modifiers ("на русском", "с нуля", "입문", "공부법").
- README "Interface Languages" table lists VI/KO as "planned" — desynced from live landing.

---

## 4. Differentiation Framework

### Origa vs competitor landscape

| Competitor | Stack / type | Locale coverage | Origa's wedge |
| --- | --- | --- | --- |
| **kana-dojo** (2.7k★) | Next.js, Vercel, web-only PWA | en/es/de/fr/zh — **NO vi/ko** | Origa covers VI+KO (top JP-learner markets); native desktop/mobile; content-driven |
| **Anki** | Desktop+mobile, SRS | English UI (RU community decks) | All-in-one (vocab+kanji+grammar+listening), native RU/VI/KO UI, zero-config |
| **WaniKani** | Web, radical-based kanji | English only | Learn kanji you encounter in your content; not a fixed radical order |
| **Bunpro** | Web, grammar SRS | English only | Grammar examples use vocab you already know; unified with vocabulary |
| **Duolingo** | Mobile, gamified | Many langs (incl. RU/VI/KO UI) but shallow JP | Depth: JLPT, content-driven, 200K phrases; Duolingo = starter, Origa = growth |
| **Mazii** (VI) | Dictionary | VI dictionary | Origa = learning app (not dictionary); JLPT + offline + content |
| **HeyJapan** (VI) | Gamified beginner | VI UI | Depth vs gamified surface; JLPT/work-visa prep |

### Defensible wedges (SEO messaging anchors)

1. **Native-language learning beyond English** — "Learn Japanese in your own language" / "японский без английского посредника" / "Học tiếng Nhật bằng tiếng Việt" / "한국어로 일본어". kana-dojo and the Western stack cannot follow here without rebuilding their i18n.
2. **Content-driven learning** — learn from manga/anime/textbooks you actually consume (OCR/STT). Matches "learn japanese from manga", "học tiếng nhật qua anime".
3. **Offline-first + local AI** — matches "learn japanese offline", "app học tiếng nhật offline", "오프라인". Privacy is not a search demand (no autocomplete) but reinforces the persona.
4. **Anki modernization** — "anki alternative japanese", "карточки японский", "anki японский". Import + integrated ecosystem.

---

## 5. Competitor Deep-Dive: kana-dojo (primary reference)

**What they do that Origa does NOT** (tracked for future phases, out of scope here):

| Feature | kana-dojo | Origa | Priority |
| --- | --- | --- | --- |
| IndexNow API (Bing instant index) | ✅ | ❌ | Phase 2 |
| `browserconfig.xml` (Edge tiles) | ✅ | ❌ | Phase 2 |
| Meta keywords (Bing values) | ✅ 40+/page | ❌ | Phase 2 (Bing only) |
| `llms.txt` (GEO/AI SEO) | ✅ | ❌ | Phase 2 |
| FAQ / Breadcrumb / LearningResource / Author(E-E-A-T) / Video schema | ✅ | ❌ (only SoftwareApp/Org/HowTo) | Phase 2 |
| Security headers (X-Content-Type-Options etc.) | ✅ | ❌ | Phase 2 |
| Community-driven topics (good-first-issue, hacktoberfest) | ✅ 21 topics | ❌ 9 tech topics | Now (repo meta) |

**Why Origa does not blindly copy:** kana-dojo is contributor-driven OSS (AGPL, Vercel sponsorship, hacktoberfest topics drive star growth). Origa is BSL (not pure OSS) and product-led — contributor-discovery topics would be misleading. Repo-meta optimization uses **domain + tech** topics instead (see §7).

---

## 6. Per-Market Keyword Strategy

> Full data: `docs/keyword-research-report.md`. Constraint: pre-launch zero DA → prioritize **low-difficulty long-tail + wedge terms**, not head terms.

### Brand rules (content constraints)

`docs/landing-content-plan.md` §10 forbids in landing copy: **pricing, open source, license, BSL**. `free` / `бесплатно` / `miễn phí` / `무료` is **permitted** — the app is currently free to use; plan §10 updated 2026-06-26 per product-owner decision. These "free" keywords are now used in home + download meta across all 4 locales (strong transactional/commercial-intent keywords per `keyword-research-report.md`: `japanese learning app free`, `изучение японского бесплатно`, `học tiếng nhật miễn phí`, `무료 일본어 공부 앱 추천`).

> README and GitHub repo description: "free" avoided there for consistency with the BSL license framing (README has a License section explaining BSL restrictions).

### Market 1 — English (`/`)

**Target keywords (low-med difficulty, Origa-fit):** `japanese learning app`, `learn japanese offline`, `anki alternative japanese`, `japanese ocr app`, `japanese ai tutor`, `jlpt study tracker`, `japanese immersion app`, `learn japanese from manga`.

**Recommended homepage meta:**

- Title (~50 char): `Origa — Learn Japanese: Kanji, Grammar, JLPT, Offline`
- Description (~125 char): front-load "Learn Japanese", include offline + features + "in your own language".

**Cultural notes:** "learn" >> "study" (5-10x volume). Reddit is the primary discovery channel (r/LearnJapanese) — channel insight, not on-page SEO. Privacy/local-AI is a persona differentiator, not a search demand.

### Market 2 — Russian (`/ru`)

**Target keywords:** `приложение для изучения японского на русском` (zero competition), `изучение японского языка с нуля`, `карточки японский язык`, `anki японский`, `кандзи учить`, `японская грамматика n5`, `хирагана катакана тренажер`.

**Recommended homepage meta:**

- Title (~50 char): `Origa — Японский язык: кандзи, грамматика, JLPT`
- Description (~125 char): include "на русском" (wedge), "с нуля", offline, features.

**Test constraint:** `home_meta_description` MUST contain `лексика` OR `кандзи` (`tests/seo_meta.rs::ru_home_schema_description_is_russian_not_english`).

**Cultural notes:** "с нуля" and "самостоятельно" are dominant modifiers. Yandex Wordstat shows 2-5x higher volumes than Google RU — Yandex SEO (already verified in `app.rs`) matters. Geographic demand in Russian Far East (Vladivostok, Khabarovsk).

### Market 3 — Vietnamese (`/vi`) — HIGHEST priority correction

**Target keywords:** `app học tiếng nhật offline`, `app học tiếng nhật hiệu quả`, `học kanji bắt đầu từ đâu`, `hán tự n5`, `từ vựng tiếng nhật n5`, `ngữ pháp tiếng nhật n5`, `học tiếng nhật cho người mới bắt đầu`.

**CRITICAL correction:** Replace **"Kanji" → "Hán tự"** across the entire VI locale. Vietnamese learners search "hán tự", not "kanji". This is the single highest-impact content fix in this strategy.

**Recommended homepage meta:**

- Title (~50 char): `Origa — Học tiếng Nhật: Hán tự, Ngữ pháp, JLPT`
- Description (~125 char): include "người mới bắt đầu", "offline", "hán tự", JLPT.

**Test constraint:** `features_schema_how_to_name` MUST contain `tiếng Nhật` (`tests/seo_meta.rs::vi_how_to_schema_name_is_vietnamese`).

**Cultural notes:** Mazii dominates dictionaries — do NOT compete on dictionary features, position as a learning app. Work-visa motivation (tokutei ginou) drives JLPT demand. Minna no Nihongo textbook alignment matters.

### Market 4 — Korean (`/ko`)

**Target keywords:** `일본어 공부 앱 추천`, `일본어 한자 공부 앱`, `무료 일본어 공부 앱 추천` (note: "무료" = free, brand-red-line), `일본어 입문`, `일본어 공부법`, `일본어 공부 순서`, `일본어 단어 앱`.

**Recommended homepage meta:**

- Title (~40 char): `Origa — 일본어 공부: 한자, 문법, JLPT`
- Description (~125 char): include "입문", "오프라인", "한자", JLPT.

**Test constraint:** `home_schema_feature_list` MUST contain `한자` (`tests/seo_meta.rs::software_application_schema_has_feature_list`).

**Cultural notes:** Naver ~50%+ search share — Google-only research undercounts; supplement with Naver Search Advisor later. DC Inside (디시) is the community hub (Korea's Reddit). Koreans already study Hanja (~1800 chars in school) — kanji concept is not foreign; content should acknowledge this baseline. "입문" > "초보자". Textbook/PDF culture.

---

## 7. Repo SEO (GitHub)

> GitHub repo meta drives repo discoverability (GitHub search + topic pages). Different from web-search SEO. Implementation: `gh repo edit` commands (paste-ready, user runs — credential isolation).

### Topics: 9 → 15 (max)

Current: `anki, desktop, duolingo, flashcards, fsrs, japanese, jlpt, language-learning, leptos`

Proposed (15): `anki, desktop, duolingo, flashcards, fsrs, japanese, jlpt, kanji, language-learning, leptos, learning-japanese, ocr, rust, spaced-repetition, tauri, vocabulary`

Rationale: keep existing (they index well), add domain keywords users actually filter by (`kanji`, `vocabulary`, `spaced-repetition`, `learning-japanese`) and stack signals for the Rust/Tauri ecosystem (`rust`, `tauri`, `ocr`). **Not** adding contributor-discovery topics (`good-first-issue`, `hacktoberfest`) — Origa is BSL/product-led; those would mislead contributors.

### Description: 213 → ~130 chars (keyword-optimized)

Current (213): `Learn Japanese from what you read, watch, and listen to. Built-in dictionaries, kanji trainer, grammar reference, and 200K+ native phrases with voice acting — everything you need for JLPT, all in one place.`

Proposed (~135): `Learn Japanese in your own language — vocabulary, kanji, grammar & 200K+ native phrases. JLPT prep, offline-first, FSRS spaced repetition. Rust + Tauri.`

Rationale: front-load "Learn Japanese" (head keyword), include feature keywords, mention stack for the Rust/Tauri dev audience, drop the long subordinate clause. No "free"/"open source" (brand red line).

### Other repo signals (recommendations, not auto-applied)

- **Discussions:** currently `has_discussions: false`, but README invites to Discussions. Either enable Discussions or remove the README mention. (Recommend: enable — community signal + SEO surface.)
- **Social preview image:** API cannot confirm; recommend setting repo Settings → Social Preview to `en.og.png` (1200×630) so GitHub link previews are branded.
- **License:** `NOASSERTION` is expected for BSL 1.1 (GitHub doesn't auto-detect BSL). No action needed.

---

## 8. README SEO

Current READMEs are product-complete but not keyword-targeted. Changes:

1. **Synchronize "Interface Languages" table** — VI and KO are live on the landing (`/vi`, `/ko`), not "planned". Update both README.md and README.ru.md tables to reflect reality (VI/KO = available, ID/ES = planned).
2. **Keyword-optimize the opening paragraph** — front-load "Learn Japanese" / "изучение японского языка" + feature keywords, keep brand voice (measured, no superlatives).
3. **Keep "Comparison with Other Apps"** — strong SEO section (competitor-name keywords: Anki, WaniKani, Bunpro, Duolingo, Migii).
4. **Add a one-line SEO subheading** under H1 capturing primary keywords (e.g., "Japanese learning app with vocabulary, kanji, grammar, JLPT prep — offline-first, in your own language").

No structural rewrite — README content is solid. Keyword-density tuning + table sync only.

---

## 9. Landing Content SEO — per-page rewrite plan

> Implementation = edit `src/content/{en,ru,ko,vi}.rs` string constants. Constraints: respect test assertions, brand red lines, length budgets (title ≤55, description ≤130 — modern SEO while staying close to the ≤50/≤125 brand plan).

### Rewrite scope (20 meta sets = 4 locales × 5 pages)

For each locale, rewrite `home_*`, `features_*`, `compare_*`, `download_*`, `integrations_*` meta title + description. H1/H2 body copy adjusted only where it conflicts with cultural terms (notably VI "Kanji"→"Hán tự").

### VI full "Kanji"→"Hán tự" sweep

Beyond meta: every VI string referencing "Kanji"/"kanji" in feature names, insights, dictionary labels → "Hán tự"/"hán tự". This is the highest-impact single change.

### What is NOT touched

- `components/seo.rs` (technical — works correctly)
- `app.rs` default `<Title text="Origa — Japanese Learning App">` (test-pinned for 404)
- sitemap / robots / JSON-LD structure
- Test files (`tests/*.rs`) — I edit content to satisfy existing assertions, not weaken them

---

## 10. Brand Constraints & Red Lines (do not violate)

From `docs/landing-content-plan.md` §10 + DESIGN.md + AGENTS.md:

1. **No mentions of:** pricing, free, open source, license, BSL (in landing copy).
2. Respect competitors — never disparage (no "X is bad"; only "X is best for Y, Origa for Z").
3. No exaggerations / superlatives / forbidden words (see `@marketer` brand voice: no "revolutionary", "seamless", "cutting-edge", etc.).
4. No emoji in technical/release writing (README, release notes). README currently uses emoji headers — that is a pre-existing repo convention and is retained.
5. Fonts: Cormorant Garamond + DM Mono only (DESIGN.md) — not affected by content SEO.
6. RU: "вы", slightly more detailed, keep technical terms (JLPT, FSRS, N+1).
7. Test assertions (hard): RU desc contains `лексика`/`кандзи`; KO featureList contains `한자`; VI HowTo name contains `tiếng Nhật`; default title unchanged.

---

## 11. Implementation Roadmap

### Phase 1 — This session (content SEO, in-scope)

1. Landing meta rewrite ×4 locales ×5 pages (`content/{en,ru,ko,vi}.rs`).
2. VI "Kanji"→"Hán tự" full sweep.
3. README.md + README.ru.md keyword tune + language-table sync.
4. Repo meta paste-ready `gh repo edit` commands (topics + description).
5. Factcheck `.factcheck.json` + HUMAN GATE.

### Phase 2 — Future (tracked, out of scope now)

- IndexNow API (Bing instant indexing) — backend route + key.
- `browserconfig.xml` + meta keywords (Bing-specific).
- `llms.txt` + AI-crawler directives in robots.txt (GEO).
- Richer JSON-LD: FAQ, BreadcrumbList, LearningResource, Author (E-E-A-T), Video.
- Security headers (X-Content-Type-Options, X-Frame-Options, Referrer-Policy).
- Free web tools as traffic magnets (hiragana/katakana trainer, grammar checker) — per keyword-research content-marketing priorities.
- Blog/content marketing for long-tail educational keywords ("study order guide", "how to start kanji").
- App Store Optimization (Google Play / App Store) — separate from web SEO.

---

## 12. Factcheck status

Claims in this doc are verified against: repo source (`content/*.rs`, `components/seo.rs`, `tests/`), ADR-007/011, `docs/landing-content-plan.md`, `docs/keyword-research-report.md`, `marketing/kana-report.md`, and the kana-dojo GitHub API data. Detailed per-claim confidence in `.factcheck.json` (generated after content rewrites land).

External/unverified signals (marked, not asserted as fact):

- Naver search-share estimate (~50%) — from keyword-research-report, not independently verified.
- kana-dojo star count (2,754) — GitHub API snapshot 2026-06-26, recheck before any public claim.
