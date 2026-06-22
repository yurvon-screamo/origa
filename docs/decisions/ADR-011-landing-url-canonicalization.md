# ADR-011: URL Canonicalization Policy for origa_landing

## Status

Accepted

## Date

2026-06-22

## Context

Following ADR-007 (DNS layer fix via plain A record), `origa.uwuwu.net` became
resolvable for all major search engine crawlers. However, none of the three
target search engines (Google, Yandex, Bing) proceeded to actually index the
site beyond the homepage, and Google Search Console / Yandex Webmaster kept
reporting generic "discovered current page, not indexed" / "crawling issues"
statuses.

Investigation at the HTTP layer revealed several distinct, independent root
causes — none of which were DNS-related:

1. **Dead URLs in `sitemap.xml`**. Three hreflang alternates were published
   with trailing slashes (`/ru/`, `/ko/`, `/vi/`). The `leptos_router`
   configuration registers locale routes without trailing slashes (`/ru`,
   `/ko`, `/vi`), so slash-suffixed URLs were not matched by the router and
   fell through to the static-file fallback service, which returned 404 with
   an empty body. Sitemaps containing 4xx URLs are a strong negative signal
   in Google's quality pipeline.
2. **Missing `Cache-Control` on HTML**. Every SSR HTML response was emitted
   without a `Cache-Control` header, so Cloudflare classified the responses
   as `Cf-Cache-Status: DYNAMIC` and forwarded every request to the Railway
   origin. This produced inconsistent crawl latencies and gave crawlers no
   revalidation hints.
3. **Soft-404**. The Leptos `<Routes fallback=NotFound>` branch was never
   reached for unmatched paths because the `ServeDir` fallback service
   intercepted them first and returned a bare HTTP 404 with an empty body.
   Worse, requests that *did* match a `ParentRoute` but missed every child
   route (e.g. `/ru/typo`) silently rendered the layout shell with HTTP 200,
   producing a genuine soft-404.
4. **Hard-coded `<html lang="en">`** in the SSR shell. Pages under `/ru`,
   `/ko`, `/vi` were rendered with `lang="en"` at the document root, with
   the correct locale only applied by a client-side `<script>` after
   hydration. Crawlers saw an English document containing non-English text.
5. **Cloudflare-managed `robots.txt`** (RC-2). The Cloudflare "AI Audit"
   / "Pay Per Crawl" feature injected its own `robots.txt` block ahead of
   the user-managed file, including `Disallow: /` for several AI crawlers
   and a `Content-Signal` header. This is handled out of code, via the
   Cloudflare dashboard — see the runbook section below.

This ADR documents the canonicalisation policy that addresses (1)–(4).
Item (5) is operational and tracked separately.

## Decision

### Trailing slash canonicalisation: NO trailing slash

All URLs are canonical **without** trailing slash:

- `/`, `/features`, `/compare`, `/content`, `/download`
- `/ru`, `/ru/features`, `/ko`, `/vi`, etc.

The only exception is the site root `/` itself, which keeps its slash by
HTTP convention (an empty path is semantically equivalent to `/`).

### Redirect strategy: axum middleware, HTTP 308

Requests with a trailing slash are redirected to the slash-less canonical
form by an axum middleware (`strip_trailing_slash`):

- **Status code**: 308 Permanent Redirect (`Redirect::permanent` in axum
  0.8). 308 is semantically equivalent to 301 for GET/HEAD and additionally
  preserves the request method for non-GET traffic; Google, Yandex and Bing
  all treat 308 as a permanent redirect that passes full link equity.
- **Methods**: only GET and HEAD are redirected. POST/PUT/DELETE are passed
  through unchanged so the middleware cannot mask an accidental state-changing
  request against a slash-suffixed URL.
- **Layer position**: outermost (the last `.layer()` call on the Router) so
  it runs before routing and normalises `/ru/`, `/images/logo.png/`,
  `/favicon.png/` alike.
- **Query string**: preserved in the `Location` header (e.g.
  `/ru/?ref=twitter` → `/ru?ref=twitter`).
- **Multiple slashes**: collapsed (`/ru///` → `/ru`).

### Sitemap policy

`public/sitemap.xml` must mirror the canonical URLs: locale-prefixed paths
have no trailing slash. The site root keeps its trailing slash. Verification:

```bash
rg 'hreflang="(ru|ko|vi)"' origa_landing/public/sitemap.xml
# all results end without "/"
```

### Cache-Control policy

- **HTML (leptos routes)** — `public, max-age=300`: 5-minute edge cache;
  users pick up content changes within 5 min.
- **Static assets** — `public, max-age=31536000, immutable`: hashed/immutable;
  bumped only by redeploy.
- **`robots.txt`, `sitemap.xml`** — `no-cache`: crawlers must always see the
  latest copy.
- **308 redirects** — `public, max-age=86400`: 24h edge cache; CDN serves the
  redirect without re-hitting origin. Google recommends `max-age >= 86400` for
  permanent redirects so link equity transfer is not delayed.
- **4xx/5xx error responses** — `no-cache`: CDNs must not pin "not found" or
  server errors; a later-added file must be served immediately.
  `ServeDir`/`ServeFile` stamp their configured value (`IMMUTABLE_CACHE` for
  assets, `NO_CACHE` for robots/sitemap) on *all* statuses via
  `insert_response_header_if_not_present`, so a status-aware middleware
  overrides every one of them to `no-cache` on errors.

Implementation uses `tower_http::ServiceExt::insert_response_header_if_not_present`
on each per-route static service to set the success-path value, then a single
`enforce_cache_policy` axum middleware post-processes every non-redirect
response:

- 2xx without `Cache-Control` → stamped with `HTML_CACHE` (the leptos-route
  default).
- 4xx/5xx → overridden to `no-cache`, regardless of what the inner service
  set (fixes the 404-with-immutable regression).
- 3xx (e.g. `304 Not Modified`) → passed through unchanged, preserving
  conditional-cache semantics.

`308 Permanent Redirect` responses from `strip_trailing_slash` never reach
`enforce_cache_policy` (the redirect layer is outermost and short-circuits
the inner stack), so they stamp `REDIRECT_CACHE` on themselves directly.

### Soft-404 policy

Unmatched paths must return HTTP 404 with a visible "404" body. This is
implemented by composing `tower_http::ServeDir::fallback` with
`leptos_axum::ErrorHandler`:

1. A request like `/random` is not matched by any explicit route, nor by any
   `leptos_router`-registered path.
2. It reaches the `fallback_service`, which is a `ServeDir(public/)`
   chained into `ErrorHandler`.
3. `ServeDir` cannot find `public/random` and delegates to `ErrorHandler`.
4. `ErrorHandler` renders the App via the leptos router. The App's
   `<Routes fallback=NotFound>` branch fires, which calls
   `leptos_axum::ResponseOptions::set_status(NOT_FOUND)` via context.
5. `ErrorHandler` post-checks: if the response status is still 200, it
   overrides to 404. `ErrorHandler` is the primary 404 mechanism;
   `ResponseOptions::set_status` inside `NotFound` is a defence-in-depth
   layer that also documents intent at the component level.

### `<html lang>` policy

The SSR shell no longer hardcodes `lang="en"` on the `<html>` tag. Instead,
`<html>` is emitted without a `lang` attribute, and `leptos_meta::Html`
inside the `Layout` component is the single source of truth:

```rust
view! {
    <Html {..} lang=locale.as_str() />
    // ...
}
```

`leptos_meta` registers the attribute with `ServerMetaContext` during render
and injects it into the `<html>` tag of the streamed response. This ensures
search engines see the correct `lang` for each locale (en/ru/ko/vi) in the
raw SSR HTML, without relying on client-side JS to patch
`document.documentElement.lang`.

The previous runtime JS patch has been removed.

## Alternatives Considered

### A1: Trailing slash AS canonical + redirect `/ru` → `/ru/`

Rejected. The existing `leptos_router` configuration already matches
`path!("ru")` without a slash; switching canonical to `/ru/` would require
either router refactoring or a `*` matcher, both of which are higher-risk
than the chosen approach. No-slash canonical is also shorter and matches
the convention used by the `hreflang` alternates already emitted in the
HTML `<head>`.

### A2: Sitemap fix only, no redirect middleware

Rejected as defence-in-depth failure. The sitemap is one of many entry
points for crawlers; external inbound links are not under our control. A
permanent redirect ensures that any future `/ru/` request — whether from a
stale bookmark, a misconfigured campaign URL, or a crawler that appended a
slash itself — reaches the canonical URL and passes link equity.

### A3: `Cache-Control: no-cache` for HTML

Considered (max freshness, simplest semantics). Rejected because it forces
Cloudflare to forward every HTML request to the Railway origin, which
defeats the purpose of having a CDN and inflates crawl latency. A 5-minute
edge cache with `public` visibility is the standard trade-off for
marketing/landing HTML that changes at most a few times per week.

### A4: axum `fallback(leptos_axum::file_and_error_handler(shell))`

Considered as the soft-404 fix. `file_and_error_handler` serves static files
from `LeptosOptions::site_root` (default `target/site`) and renders the App
for everything else. Rejected because our static files live in
`public/` (not `target/site`), and switching `site_root` would also affect
the trunk output path. The chosen composition
(`ServeDir::new(public).fallback(ErrorHandler)`) keeps the existing static
directory layout and achieves the same 404 behaviour.

## Consequences

### Positive

- Every URL listed in `sitemap.xml` returns 200 (directly or via 308
  redirect). Sitemap health is "green" in Google Search Console.
- Cloudflare can cache HTML for 5 minutes, eliminating the
  `Cf-Cache-Status: DYNAMIC` warning and stabilising crawl latency.
- Search engines see consistent canonical, locale, hreflang and
  cache-control signals for every page.
- `/random` (and any other unmatched path) returns HTTP 404 with a
  visible body, so it is treated as a real 404 rather than indexed as a
  soft-404 page.

### Negative

- An extra 308 redirect hop is incurred for slash-suffixed URLs (one
  additional round-trip). This is a one-time cost per inbound link and is
  cached by the CDN.
- Adding a new static asset to `public/` requires an explicit
  `route_service` entry to get the correct `Cache-Control` header; files
  served by the fallback `ServeDir` get the default HTML cache policy
  (`public, max-age=300`) rather than immutable caching. This is acceptable
  for now — the set of static assets is small and stable.

## Cloudflare Runbook (RC-2)

The Cloudflare "AI Audit" / "Pay Per Crawl" feature, when enabled on the
`uwuwu.net` zone, injects a managed `robots.txt` block that prepends
`Content-Signal: search=yes,ai-train=no` and `Disallow: /` directives for
specific AI user-agents *before* the user-managed `public/robots.txt`
content. This causes some crawlers that fetch `robots.txt` to see a partial
or contradictory file.

### Fix (dashboard)

1. Cloudflare dashboard → zone `uwuwu.net`.
2. Navigate to **AI** → **AI Audit** (or **Pay Per Crawl**, depending on
   the account tier).
3. Disable **AI Audit** and **Pay Per Crawl**.
4. Save.

### Verification

```bash
curl -s https://origa.uwuwu.net/robots.txt
# Expected body (exactly 4 lines, no AI crawler block):
# User-agent: *
# Allow: /
#
# Sitemap: https://origa.uwuwu.net/sitemap.xml
```

### Rollback

Re-enable **AI Audit** / **Pay Per Crawl** in the same dashboard panel.

## Production Verification

End-to-end smoke checks against production (after deploy):

```bash
# Trailing-slash redirect
curl -o /dev/null -s -w '%{http_code}\n' https://origa.uwuwu.net/ru/
# Expected: 308

# HTML cache header
curl -s -D - -o /dev/null https://origa.uwuwu.net/ | rg -i 'cache-control:'
# Expected: public, max-age=300

# Static asset immutable cache
curl -s -D - -o /dev/null https://origa.uwuwu.net/favicon.png | rg -i 'cache-control:'
# Expected: public, max-age=31536000, immutable

# Soft-404
curl -o /dev/null -s -w '%{http_code}\n' https://origa.uwuwu.net/random
# Expected: 404

# Sitemap contains no slash-suffixed locale URLs
curl -s https://origa.uwuwu.net/sitemap.xml | rg 'hreflang="(ru|ko|vi)"'
# Expected: every href ends without "/"
```

## References

- ADR-007: Landing DNS — Replace CNAME with A Record to Fix Search Engine
  Indexing (Railway AAAA SERVFAIL). DNS-layer predecessor of this ADR.
- axum 0.8 `Redirect::permanent` → HTTP 308.
- `leptos_axum::ResponseOptions::set_status` for SSR-only HTTP status.
- `leptos_axum::ErrorHandler` as a tower `Service` for compositional
  fallback chains.
- `tower_http::ServiceExt::insert_response_header_if_not_present` for
  per-route `Cache-Control` defaults on static services.
- `axum::middleware::from_fn` for the `enforce_cache_policy` and
  `strip_trailing_slash` middlewares (status-aware `Cache-Control` on
  non-redirect responses; `REDIRECT_CACHE` stamped directly on 308s).
