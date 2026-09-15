# ADR-054: Umami Cloud Web Analytics

## Status

Accepted (2026-09-13)

## Context

Origa had no product analytics: neither the landing (`origa.uwuwu.net`) nor
the web app (`app.origa.uwuwu.net`) measured pageviews, traffic sources, or
conversion from landing to app. The only usage signal is Sentry's anonymous
installation identifier (ADR-036), which counts installs/launches but says
nothing about which screens or journeys users go through.

Requirements for the analytics provider, fixed by the product owner:

- **One website for both surfaces** (landing + app share a free Umami Cloud
  plan website id). Paths do not overlap (`/features`, `/blog/*` vs
  `/words`, `/lesson`), and Umami session hashes include the hostname, so
  landing and app visitors never merge.
- **Desktop is tracked too**: the Tauri build runs the same `origa_ui`
  WASM and its pageviews show which screens desktop users actually visit.
- **CI must be fully muted**: e2e headless runs, the android-smoke emulator,
  and CI-validated desktop builds must never pollute production data.
- Cookieless, no cross-site identifiers — consistent with the "Privacy-first"
  positioning, so no consent banner is required (GDPR: no personal data).
- Basic declarative events (`data-umami-event` attributes) for conversion
  (download CTAs, login, lesson completion, onboarding completion).
  High-frequency events (FSRS card ratings, OCR/STT calls) are deliberately
  NOT tracked: they would burn the free plan's event quota and add noise.

Chosen provider: **Umami Cloud** (`cloud.umami.is`) — privacy-focused,
open-source core, hosted by the creators.

## Decision

### 1. Two injection mechanisms

- **Landing** — a static `<script defer>` tag in `shell()`
  (`origa_landing/src/app.rs`) with `data-domains="origa.uwuwu.net"`
  (local dev is not tracked) and `data-do-not-track="true"`. The landing is
  never executed inside CI, so no mute gate is needed. No CSP header is
  served for the landing (ADR-015 stamps only
  X-Content-Type-Options/X-Frame-Options/Referrer-Policy/Permissions-Policy),
  so a third-party script needs no server changes.
- **Web app / desktop** — the tracker is injected at runtime from WASM
  (`origa_ui/src/core/analytics.rs::inject_umami`, called from `main.rs`
  before mount). A static tag in `index.html` cannot be muted per build, and
  `index.html` is shared by production and CI builds.

### 2. Build-time mute gate (`UMAMI_DISABLED`)

`origa_ui/build.rs` emits `UMAMI_WEBSITE_ID` via `cargo:rustc-env`:

- `UMAMI_DISABLED=1` (or `true`, case-insensitive) → empty value → runtime
  injection is a no-op. Fail-open on typo'd values (`"0"`, `"false"`, `""`
  leave analytics enabled): losing analytics on a release build silently is
  worse than a noisy CI run.
- `UMAMI_WEBSITE_ID` optionally overrides the committed default
  (`DEFAULT_UMAMI_WEBSITE_ID` in `build_config.rs`). Website ids are public
  identifiers (visible in every page's HTML), safe to commit — unlike an API
  key.
- Default (unset) → production website id → analytics ON.

`ci.yml` sets `UMAMI_DISABLED=1` in exactly two steps: the e2e-build
`Build WASM` (its `wasm-dist` artifact also feeds CI-validated
`build-tauri`/`build-macos`) and the android-smoke `Build WASM frontend`.
Release builds (`tauri.yml` `build-frontend`) and the production web Docker
build do NOT set the flag → analytics ON. **No release tag may be cut
between the injector landing and the CSP change (both shipped together).**

### 3. Tauri CSP extension

`https://cloud.umami.is` is allow-listed in `script-src` (tracker) and
`connect-src` in `tauri/build_config.rs::build_csp` and `tauri.conf.json`
(byte-equality is drift-tested). Same pinning discipline as Sentry
(ADR-036 §7): the host executes JS in the WebView where Tauri IPC is in
reach, so no wildcards.

**Post-mortem (2026-09-15):** the cloud tracker script does NOT send to its
own origin — its built-in default endpoint is
`https://gateway.umami.is/api/send` (unless `data-host-url` overrides it),
and the tracker swallows fetch failures silently. The first release
therefore loaded the script on desktop but dropped every beacon to the CSP.
`connect-src` now also pins `https://gateway.umami.is`. Browsers (web app,
landing) were never affected: no CSP header is served there.

**Recurrence vector:** the CSP pins the tracker's *internal* default
endpoint. If Umami changes that default in a future `script.js`, desktop
repeats this exact silent failure (`catch {}` swallows it; e2e is muted and
the wasm tests only assert tag attributes). The desktop Realtime check
below is therefore a standing release-checklist item, not a one-time
post-CSP gate.

**Accepted supply-chain risk:** a compromise of Umami Cloud means hostile JS
running inside the desktop app. Precedent: jsdelivr, sentry-cdn are already
trusted the same way; the host is pinned exactly, not wildcarded.

### 4. `data-domains` for the app build

`app.origa.uwuwu.net,tauri.localhost,localhost` — production web plus the
origins Tauri serves desktop from (Windows/Android `tauri.localhost`,
Linux `http://localhost`, macOS `tauri://localhost` → hostname `localhost`).
The web host is derived from `TRAILBASE_URL` (single source of truth in
`build_defaults.rs`; both deploy on the same host).

Consequence: local dev (`trunk serve`, `cargo tauri dev`) IS tracked — the
same `localhost` origin as Linux/macOS production. Accepted: one developer's
noise; personal browsers can enable Umami's "Exclude my own visits". Local
e2e runs must export `UMAMI_DISABLED=1` (documented in `end2end/README.md`).

### 5. Events (declarative `data-umami-event`)

Landing: `hero_cta_download`, `open_webapp` (hero + download page),
`download_windows`, `download_linux`, `download_android`. The shared
`CtaSection` is deliberately NOT tracked (fires on every page that embeds it).

App: `login_submit` (submit click — an attempt, not a successful login;
Enter-submit is not caught), `login_google` / `login_yandex` /
`login_apple`, `lesson_finish` (both complete-screen exit buttons — a click
on either means the lesson was finished; the Esc path is not caught),
`onboarding_complete` (scoring finish button). Lesson start is NOT an event:
the `/lesson` pageview already covers it. Skip paths (the confirm-modal
"skip onboarding" / "skip scoring" actions that finish onboarding without
the scoring button) do NOT emit `onboarding_complete` — deliberate: the
funnel measures honest completions, and skip traffic stays visible as
pageview drop-off. Programmatic events (`umami.track()` after async
validation) are a future slice if exact semantics are needed.

### 6. Privacy policy

`PR_BODY_EN`/`PR_BODY_RU` gained a "Web analytics" data category, a storage
location (Umami Cloud), and a third-party-services entry. The previous
"we do not collect browsing history" claim was reworded to scope it to
activity outside Origa services. Drive-by fix: the provider lists now
mention Apple sign-in everywhere it is enumerated (authentication row,
storage section, third-party services) — previously only Google/Yandex were
listed although the app ships `OAuthProvider::Apple`.

## Alternatives Considered

### Self-hosted Umami
- Pros: no quota, data ownership, no third-party JS host in the CSP.
- Rejected for now: another always-on service to operate (the project
  already runs Railway services, CDN, Sentry SaaS); the free cloud plan
  covers current volume. Can be migrated later — the tracker contract
  (`data-website-id`, `data-host-url`) makes self-hosting a config change.

### Google Analytics
- Rejected: cookies/consent banner required, contradicts privacy positioning.

### Sentry-based funnel metrics
- Rejected: Sentry is error telemetry (ADR-036); pageview analytics would
  abuse transactions for product metrics and mix concerns.

### Static tag + Playwright route-abort for CI mute
- Rejected: covers only the browser-based e2e job; the android-smoke
  emulator runs a real WebView that Playwright cannot intercept. A build
  gate covers both uniformly.

## Consequences

- CI data stays clean; the mute gate is itself guarded by an e2e scenario
  (`smoke.feature`: "Трекер аналитики Umami отключён в CI-сборке", skipped
  outside CI) and a positive wasm test verifying the injected tag's
  attributes (`analytics_wasm_tests.rs`).
- Web vs desktop app traffic is distinguishable only by hostname in the
  payload (and desktop's empty referrer); Umami's UI may not expose a
  hostname filter on the free plan. Splitting into two websites remains a
  plan upgrade away.
- Ad blockers blocking `cloud.umami.is` cause undercounting (accepted; the
  bypass is a paid cloud feature).
- New env vars to document: `UMAMI_WEBSITE_ID`, `UMAMI_DISABLED` (see root
  AGENTS.md).
- Release checkpoint: after the CSP change, a local release build on ≥1
  platform must show a visit in the Umami Realtime dashboard before the
  next release is tagged (desktop is the only surface whose CSP × WebView ×
  custom-origin combination is not covered by automated tests).
