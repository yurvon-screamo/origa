# ADR-054: Umami Cloud Web Analytics

## Status

Accepted (2026-09-13). Amended (2026-10-22): the provider moved from Umami
Cloud to a project-owned self-hosted instance — see §7. The cloud-vs-crypto
details below (§1–§6) describe how the integration was built and stay
accurate except for the provider, website id, and CSP hosts, which §7
replaces. Amended again (2026-10-29): while Railway's custom-domain
certificate issuance is broken, `umami.origa` routes through the RF VM for
both geo branches — see §7, Domain.

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

#### 5.1. Programmatic onboarding step events (issue #774, 2026-10)

The onboarding funnel (login_submit 370 → onboarding_complete 33 ≈ 8.9%,
Umami 14.09–13.10) had no intermediate measurement: the seven wizard steps
are SPA state inside one `/onboarding` pageview, invisible to the tracker.
The future slice above arrived: `origa_ui/src/core/analytics.rs::track_event`
wraps the tracker's `umami.track(name, data)` contract (js-sys interop,
fire-and-forget, warn-and-drop on a failed call). Events fired before the
tracker script executes (it loads asynchronously; the window covers the
first hundreds of milliseconds of a page — cold start, the resume jump to
Scoring) go into a bounded pending queue (FIFO, 10 s give-up) flushed as
soon as `window.umami` appears — the e2e scenario below caught exactly
this drop before the queue existed.

**Contract** — every step of `OnboardingStep` emits `start` on entry
(single Effect memo-gated on the step value: mount, forward/back
navigation, the resume jump — but NOT intra-step mutations like
level/app picks, which update the same signal without changing the step)
and `complete` on its forward action:

| Step enum | slug | start | complete (fired by) |
|---|---|---|---|
| Intro | `intro` | `onboarding_intro_start` | Next click |
| Load (stepper: "pace") | `load` | `onboarding_load_start` | Next click |
| Jlpt (stepper: "level") | `jlpt` | `onboarding_jlpt_start` | Next click |
| Apps | `apps` | `onboarding_apps_start` | Next click |
| Progress | `progress` | `onboarding_progress_start` | Next click |
| Summary (stepper: "import") | `summary` | `onboarding_summary_start` | successful import checkpoint (both branches) |
| Scoring | `scoring` | `onboarding_scoring_start` | = existing declarative `onboarding_complete` (NOT duplicated) |

Slugs follow the enum identity, not the localized stepper labels — the
table above is the mapping. Every event carries event-data properties
`locale` (`en|ru|ko|vi`, untracked read at fire time) and `device`
(`mobile|desktop`).

**Semantics:**

- *Attempt semantics*, same as `login_submit`: `complete` fires on the
  forward action (steps Intro–Progress are synchronous clicks; Summary
  completes only when the import checkpoint succeeds — a failed import
  leaves the user on the step and emits nothing). Back-navigation re-fires
  `start`; funnel tools match first occurrence in order, so repeats do not
  distort step conversion.
- *Remount dedup*: the ProtectedRoute sync overlay can unmount and remount
  the Onboarding component mid-step (an authenticated `save_sync` — e.g.
  the intro language save) without changing the step. Start events are
  deduped against the last started step in the page session, so a remount
  never double-fires the same step's `start`; navigation (step value
  changed) always emits.
- *Skip paths stay silent* (unchanged from the declarative contract): skip
  is visible as drop-off after the last `start`.
- *Bounce noise*: `intro_start` fires on every `/onboarding` mount,
  including redirects of users who already completed onboarding (→`/home`)
  or have no session (→`/login`) — the funnel top is slightly wider than
  "actually saw the intro".
- *Cold start*: the tracker script loads asynchronously; events fired
  before it executes are queued (FIFO, ≤10 s) and flushed on tracker
  arrival, so no onboarding event is lost to the load race. If the tracker
  never appears (ad blocker, muted build) the queue is discarded.

**Verification**: local-only BDD scenario «Шаги онбординга доставляются в
Umami как события» (`smoke.feature`, issue #774) intercepts the real
`POST /api/send` beacons and asserts the ordered funnel + properties; it
skips on muted builds (runtime probe), so the e2e matrix invariant holds.
The standing release checkpoint (Realtime dashboard check below) applies
unchanged. When reading the funnel in the Umami dashboard, filter out
hostname `localhost` — dev runs and this scenario legitimately inject
synthetic funnels into production analytics.

**Volume**: ~2–3K additional events/month estimated (≈300 onboarding
entries × 7–10 events, abandonment included) — low-frequency relative to
traffic, but monitor event volume in the dashboard after rollout; at quota
pressure the options are a plan upgrade or event pruning.

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
- Revisited 2026-10: the free plan's API/team limits made the "later" real
  — see §7 for the executed migration.

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

## §7. Amendment (2026-10-22): Self-hosted Umami on Railway (issue #733)

### Why

Umami Cloud's free plan stopped being workable for a team:

- **API is Pro-only.** Statistics were scraped with a session-cookie
  workaround that returns only the page shell (`/api/*` = 401; verified
  2026-10-12); the session token is owner-minted and rots.
- **No team access.** One owner account; every dashboard visit goes through
  the owner's browser session.
- **Silent failures.** The tracker swallows errors (the §3 post-mortem: the
  desktop lost 100% of beacons to a CSP mismatch, caught only by a manual
  Realtime check). Cloud endpoints are outside our control, so the
  recurrence vector (§3) is permanent.
- **No desktop/web split** in the cloud UI, opaque free-tier limits.
- **Vendor lock**: data lives on their cloud.

Self-hosting removes the whole class: full API without a subscription, team
users, our own endpoints and limits, our own data.

### Topology

- **Railway** (project `origa-appuru`, production): service `umami` from the
  pinned image `umamisoftware/umami:3.4.0` (v3 is PostgreSQL-only) + a
  Railway PostgreSQL. Env: `DATABASE_URL` (reference to the Postgres
  service), `APP_SECRET` (random), `CLIENT_IP_HEADER=x-forwarded-for` —
  without it every Railway-proxied visit shares one IP.
- **Domain**: `umami.origa.uwuwu.net`. Originally served through the same
  geo-split as every other `origa.*` name (Bunny DNS script 91629,
  `infra/bunny-dns/handleQuery.js` gained a fifth guarded name): world →
  CNAME to the Railway edge name, RF → the RF VM (`193.233.217.243`),
  because Railway subnets are payload-dropped from RF (ADR-049). The RF VM
  nginx got a dedicated cert (`acme.sh` DNS-01 via Bunny — this also proved
  TXT fall-through works for script-attached names, the open question for
  the November renewals of the existing certs) and a dedicated http `:8443`
  server block with `proxy_ssl_verify on`, mirroring `app.origa`.
  **Amendment (2026-10-29):** Railway's custom-domain certificate issuance
  fails persistently with an internal error (4+ retries over two days; the
  domain is Verified and routing works, but the edge serves the wildcard
  `*.up.railway.app` cert, failing world browsers' TLS). Until Railway
  fixes the pipeline, the script routes **both geo branches through the RF
  VM** — it holds a valid Let's Encrypt cert for the name and the
  beacon/script volume is trivial. Flip back to the geo-split (one script
  deploy, TTL 60) once `domain-status` reports a VALID certificate.
- Deploy/verify procedure: `docs/runbooks/umami-self-host.md`.

### Tracker changes

- Script origin and collection endpoint are now the same host, and the
  endpoint is pinned explicitly: `data-host-url="https://umami.origa.uwuwu.net"`
  on all three injection surfaces (landing static tag, WASM injection, and
  the CSP it must match in `tauri/build_config.rs` + `tauri.conf.json`).
  The explicit pin closes the §3 recurrence vector: whatever collection
  default a future tracker bundle carries, the attribute wins.
- CSP: the `cloud.umami.is` + `gateway.umami.is` pair is replaced by the
  single `umami.origa.uwuwu.net` in both `script-src` and `connect-src`
  (byte-equality drift test unchanged).
- New website id: `DEFAULT_UMAMI_WEBSITE_ID` points at the website created
  on the self-hosted instance (same one-website-for-all-surfaces contract
  as §Decision-1; `data-domains` unchanged).
- The wasm tests additionally assert that `data-host-url` equals the script
  origin host.
- **New cross-crate drift guard** (`origa_ui/tests/umami_hosts.rs`): the
  tracker constants, the landing tag, the built CSP and the committed
  `tauri.conf.json` CSP must all name the same analytics host, and the
  landing tag must carry the same website id as `DEFAULT_UMAMI_WEBSITE_ID`.
  Before it, nothing tied the desktop CSP to the tracker's endpoint — the
  exact gap that made the §3 incident silent (a mismatch dropped 100% of
  desktop beacons with every test still green).

### Privacy policy

The EN/RU privacy pages no longer list Umami as a third-party processor:
the instance is operated by Origa (§6 wording updated accordingly). The
data category itself (cookieless, aggregated, IP-discarding, DNT-respecting)
is unchanged — same Umami software, now on our infrastructure.

### Operations

- Team users: view-only accounts (`umami-ru`, `umami-en`, `umami-ko`,
  `umami-vi`) for the marketing bots; the owner keeps admin. Credentials
  follow the access-keys convention (`marketing/.env` + the uwuwu keys
  inventory). Access uses Umami v3 API keys (long-lived) instead of the
  login token, which expires.
- **Default admin**: Umami's `01_init` migration creates `admin`/`umami` —
  a public password. The runbook makes rotating it the first bootstrap
  step, before the custom domain is published.
- History: Umami has no historical-data import. The Cloud CSV export is an
  owner-run archive; the Cloud account stays alive until self-host data is
  verified against the Cloud baseline (Realtime + daily aggregates), then
  the Cloud site is deactivated.
- The §Consequences release checkpoint (desktop Realtime check) applies
  unchanged — now against `umami.origa.uwuwu.net`.
- Accepted operational cost: one more always-on Railway service + Postgres
  (usage-billed), backups now our responsibility (Railway Postgres backups
  enabled).
