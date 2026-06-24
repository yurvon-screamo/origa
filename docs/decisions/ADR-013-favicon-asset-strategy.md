# ADR-013: Favicon Asset Strategy

## Status

Accepted

## Date

2026-06-24

## Context

Yandex Webmaster reported "could not load favicon" for `origa.uwuwu.net`. Investigation
showed `/favicon.ico` returned HTTP 404 — the landing served only `/favicon.png`
(configured via `<link rel="icon" type="image/png">`). Yandex (and some browsers)
request `/favicon.ico` by default and treat its absence as a missing favicon, which
also suppresses the favicon in Yandex search snippets.

The landing had no vector source for the logo: a repository-wide search for `*.svg`
returned only `origa_ui/public/external_icons/anki.svg` (a third-party brand icon, not
Origa). The highest-resolution raster source is `tauri/icons/icon-512.png` (512×512 RGBA),
used by the Tauri desktop build.

## Decision

Generate favicon assets from the canonical raster source via a build script:

- **Canonical source:** `tauri/icons/icon-512.png` (single source of truth for all
  raster derivatives).
- **Generator:** `scripts/generate_favicon.py`, run via `uv run --with pillow python
  scripts/generate_favicon.py`. Produces:
  - `origa_landing/public/favicon.ico` — multi-size ICO (16×16, 32×32, 48×48, LANCZOS).
  - `origa_landing/public/apple-touch-icon.png` — 180×180 PNG.
- **Generated artifacts are committed** (not built at CI time) so deployments do not
  require Pillow.
- **Serving:** two explicit `route_service` entries in `server.rs` with
  `IMMUTABLE_CACHE` (public, max-age=31536000, immutable), alongside the existing
  `/favicon.png` route.
- **`<head>` links** (`app.rs` shell):
  - `<link rel="icon" href="/favicon.ico" sizes="16x16 32x32 48x48" />`
  - `<link rel="icon" type="image/png" href="/favicon.png" />` (kept for back-compat)
  - `<link rel="apple-touch-icon" href="/apple-touch-icon.png" />`
- **SVG favicon: not added.** No vector source exists; PNG-traced SVG would defeat the
  purpose of vector scaling. Re-evaluate if a vector logo is produced.

## Alternatives Considered

### SVG favicon traced from PNG

- Rejected: auto-tracing the raster logo produces a large, imprecise SVG that does not
  scale cleanly. A hand-drawn SVG was out of scope (no designer source available).

### Replace `favicon.png` with the new ICO

- Rejected: `favicon.png` is referenced by external systems and keeps a PNG entry in
  the `<head>` for clients that prefer it. Keeping both maximises compatibility.

### Generate ICO at CI time (do not commit artifacts)

- Rejected: would require Pillow in every CI/CD image and add a build dependency for a
  one-time-per-logo operation. Committing the ~8 KB ICO + ~43 KB PNG is cheaper.

## Consequences

- Yandex favicon discovery unblocked (`/favicon.ico` now 200).
- **Regeneration trigger:** whenever `tauri/icons/icon-512.png` changes, re-run
  `generate_favicon.py` and commit the new artifacts. This is a manual step — not
  wired into CI to avoid the Pillow dependency there.
- **CDN cache rotation trade-off:** because favicon routes use `IMMUTABLE_CACHE`
  (1-year edge cache per ADR-011), a logo change will not propagate until the cache
  expires or is purged. If the logo ever rotates, either version the URL
  (`/favicon.v2.ico`) or purge the Cloudflare cache for these paths.
- `favicon.png` (32×32, the previous web favicon) is intentionally left in place for
  backward compatibility.
