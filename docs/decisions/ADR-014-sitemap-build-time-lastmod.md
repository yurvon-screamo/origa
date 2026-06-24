# ADR-014: Sitemap Build-Time <lastmod> Generation

## Status

Accepted

## Date

2026-06-24

## Context

`origa_landing/public/sitemap.xml` listed five canonical URLs with `hreflang`
alternates but no `<lastmod>` element. Google largely ignores `lastmod`, but Yandex
uses it to prioritise crawl frequency. Without it, Yandex had no freshness signal and
treated every URL as equally (un)likely to have changed.

The sitemap is served as a static file via `tower_http::ServeFile` with `NO_CACHE`
(per ADR-011), so adding `lastmod` had to preserve the static-serving model rather
than introduce a dynamic Axum handler.

## Decision

Generate `sitemap.xml` at **build time** from a template:

- **Template:** `public/sitemap.xml` was renamed to `public/sitemap.xml.tmpl` with a
  `{{LASTMOD}}` placeholder inserted immediately after each `<loc>`, before the
  `<xhtml:link>` alternates (per sitemaps.org 0.9 element ordering).
- **Generator:** `build.rs::generate_sitemap(manifest_dir)`, wired with
  `cargo:rerun-if-changed=public/sitemap.xml.tmpl` and
  `cargo:rerun-if-env-changed=ORIGA_BUILD_DATE`. Performs
  `template.replace("{{LASTMOD}}", &date)` and writes `public/sitemap.xml`.
- **`lastmod` source — fallback chain:**
  1. `ORIGA_BUILD_DATE` env var (set by CI). Authoritative for Docker/production
     builds, gives bit-for-bit reproducibility.
  2. `git log -1 --format=%cd --date=short --follow -- public/sitemap.xml.tmpl` via
     `std::process::Command`. Local-dev convenience only; the Docker builder stage has
     no git, so this path is never reached in production.
  3. `"1970-01-01"` literal + `cargo:warning=...`. Explicit sentinel meaning "no
     freshness data available" — does not mislead Yandex with a fabricated recent
     date.
- **Output is gitignored:** `origa_landing/.gitignore` ignores
  `public/sitemap.xml` (it is a build artefact, regenerated every build).
- **CI wiring:** `Dockerfile` gained `ARG/ENV ORIGA_BUILD_DATE`; `.github/workflows/
  docker.yml` passes `ORIGA_BUILD_DATE` in the landing image build args, reusing the
  existing `version` job's `build_date` output.
- **Production URLs in the template are intentionally hardcoded** (not env-derived):
  the sitemap must advertise stable canonical public URLs for crawlers independent of
  the build environment. A comment in the template documents this rationale.

## Alternatives Considered

### Static hardcoded `<lastmod>`, updated manually per release

- Rejected: a manual step before every release is fragile and easy to forget. Stale
  `lastmod` is worse than none (signals false freshness).

### Runtime Axum handler generating `sitemap.xml` on each request

- Rejected: would abandon the `ServeFile` static model (ADR-011), add per-request work,
  and require serialising XML in Rust on every hit. The static file with build-time
  substitution keeps serving zero-cost.

### Use the build timestamp always (`Utc::now()` in `build.rs`)

- Rejected: breaks reproducible builds — two builds of the same commit produce
  different `lastmod` values. The `ORIGA_BUILD_DATE` env path gives reproducibility;
  the git path gives commit-accurate dates locally.

## Consequences

- Every build emits a `sitemap.xml` with accurate `<lastmod>` for all five URLs.
- **Production reproducibility** depends on CI always setting `ORIGA_BUILD_DATE`. The
  Docker builder has no git, so if the env var is unset in CI, the sentinel
  `1970-01-01` is emitted with a `cargo:warning` — a visible signal, not a silent
  failure.
- **Local dev** gets commit-accurate dates via git; no env var needed.
- The `.tmpl` file is the source of truth; `public/sitemap.xml` is never edited
  directly (it is gitignored).
- Adding a new URL to the sitemap means editing `sitemap.xml.tmpl` and adding the
  `<lastmod>` placeholder in the correct position.
