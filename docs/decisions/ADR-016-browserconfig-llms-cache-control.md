# ADR-016: Cache-Control for browserconfig.xml and llms.txt

## Status

Accepted

## Date

2026-06-26

## Context

Two new static files were added to `origa_landing/public/`:

- **`browserconfig.xml`** — Windows Edge/IE tile configuration. Points the
  `square150x150logo` at the existing 180×180 `apple-touch-icon.png` and pins
  the `TileColor` to the theme colour (`#3d4535`).
- **`llms.txt`** — a factual, brand-voice summary of Origa for AI assistants
  and crawlers (GEO / Generative Engine Optimization). Covers what the product
  does, the four interface languages, offline/local-AI capabilities and that
  it is currently free.

Per ADR-011, static files served via `tower_http::ServeFile` must carry an
explicit `Cache-Control` header set through
`insert_response_header_if_not_present`. The two files have **different**
change cadences, so a single policy does not fit both:

- `browserconfig.xml` changes only when the logo or theme colour changes —
  the same cadence as the favicons (ADR-013), which are immutable.
- `llms.txt` is **release-updated** copy. Its wording tracks product changes
  (new features, new locales, positioning) and is expected to drift every few
  releases, exactly like the `grammar/`, `dictionary/` and `phrases/` JSON on
  the app CDN.

PR #182 documented the failure mode for release-updated content under
immutable caching: S3 gets the new object, but the Cloudflare edge holds the
year-long `immutable` cache and keeps serving the stale version until manually
purged (CDN edge-cache poisoning). The same class of bug applies to `llms.txt`
if it were served immutable.

## Decision

Serve the two files with **different** `Cache-Control` policies, matching
their change cadence:

| File | `Cache-Control` | Rationale |
| --- | --- | --- |
| `/browserconfig.xml` | `public, max-age=31536000, immutable` | Changes only with the logo; same cadence as favicons (ADR-013). Reuses the existing 180×180 apple-touch-icon, so no separate msstile artefact set is generated — a single-tile scope. |
| `/llms.txt` | `no-cache` | Release-updated copy. `no-cache` keeps it always-fresh at the edge while still allowing conditional (`304`) revalidation. |

Both are wired as explicit `.route_service` entries in `server.rs`, so they
pick up their configured header rather than the fallback `ServeDir` default
(`HTML_CACHE`). The `enforce_cache_policy` middleware (ADR-011) still overrides
both to `no-cache` on any 4xx/5xx, so a transient 404 on either path cannot be
pinned at the edge.

### `browserconfig.xml`: single-tile scope, no msstile set

A full Windows tile set (70×70, 150×150, 310×150, 310×310) was **not**
generated. The `square150x150logo` reuses `apple-touch-icon.png` (180×180),
which is large enough to downsample cleanly for the 150×150 tile slot. The
file exists primarily so that `msapplication-config` resolves (some Edge/IE
versions request it unconditionally); investing in a dedicated tile pipeline
is not justified for the share of that traffic.

## Alternatives Considered

### A1: Serve `llms.txt` immutable (max-age=31536000)

Rejected. `llms.txt` is release-updated; immutable caching would reproduce the
PR #182 edge-poisoning bug — AI crawlers, which cache aggressively, would keep
quoting a stale product summary until the edge cache was manually purged.
`no-cache` costs one revalidation round-trip per crawl but guarantees
freshness, which is the entire point of the file.

### A2: Serve `browserconfig.xml` with `no-cache`

Considered for symmetry. Rejected because the file is effectively immutable
between logo changes (its content is two hardcoded references), and the logo
itself already rotates via `IMMUTABLE_CACHE` (ADR-013) with the same
cache-rotation trade-off documented there. Treating `browserconfig.xml`
differently from the icon it references would be inconsistent.

### A3: Generate a full msstile set

Rejected. Windows tiles are a negligible share of landing traffic, and the
180×180 apple-touch-icon downsamples acceptably into the 150×150 slot. A full
pipeline would add a Pillow dependency and committed artefacts for no
measurable benefit (mirrors ADR-013's reasoning on SVG favicons).

## Consequences

### Positive

- `llms.txt` is always fresh for AI crawlers without manual cache purges.
- `browserconfig.xml` caches as aggressively as the favicons, consistent with
  ADR-013.
- The 4xx/5xx `no-cache` override (ADR-011) protects both paths from
  edge-pinned "not found" responses.

### Negative

- The two files have different policies, so a future contributor adding
  another static file must consciously pick a cadence rather than copy-paste
  one value. This is the right trade-off (it forces the cadence question) but
  is a small documentation burden.
- If `llms.txt` copy ever needs to change within a release window faster than
  crawlers revalidate, a manual Cloudflare purge is still required — `no-cache`
  guarantees freshness on the next request, not instant propagation.

## References

- ADR-011: URL Canonicalization Policy — `Cache-Control` policy framework and
  the `enforce_cache_policy` middleware that overrides both files on 4xx/5xx.
- ADR-013: Favicon Asset Strategy — the immutable-caching precedent for
  logo-derivative assets, and the CDN cache-rotation trade-off.
- PR #182: the CDN edge-cache poisoning regression for release-updated content
  served as immutable.
- `tests/cache_headers.rs` — `browserconfig_xml_has_immutable_cache` and
  `llms_txt_has_no_cache`.
