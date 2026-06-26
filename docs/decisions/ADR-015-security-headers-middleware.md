# ADR-015: Security Response Headers Middleware

## Status

Accepted

## Date

2026-06-26

## Context

`origa_landing` served no defensive response headers. Every response — HTML
pages, static assets, the 308 trailing-slash redirect and the 404 fallback —
shipped with default headers only. This left the site open to a class of
low-effort browser-based attacks that security scanners (Mozilla Observatory,
securityheaders.com) flag by default:

- **MIME sniffing** — without `X-Content-Type-Options: nosniff`, browsers may
  interpret a response as a type other than the one declared by `Content-Type`.
  A user-uploaded or attacker-influenced file served from the static fallback
  could be executed as HTML/JS.
- **Clickjacking** — without `X-Frame-Options`, any page can be embedded in an
  `<iframe>` on a hostile origin and layered under a transparent UI to trick
  the user into clicking (e.g. a fake "Download" overlay).
- **Referer leakage** — the default `Referrer-Policy` sends the full URL (path
  - query) as `Referer` on every cross-origin navigation. The site links to
  external hosts (fonts, GitHub), so the full landing URL — including any
  campaign query params — would leak.
- **Device API surface** — modern browsers expose `camera`, `microphone` and
  `geolocation` permissions to any origin. The landing never uses any of them,
  but without a `Permissions-Policy` lock-down the capabilities remain
  requestable, widening the impact of any future XSS.

A Content-Security-Policy (CSP) is the strongest defence of this class but is
deferred (see Alternatives) — it requires a full audit of every inline script,
style, font and image source and would currently need `unsafe-inline` for
Leptos SSR hydration, which negates most of its value.

## Decision

Add an outermost axum middleware `security_headers` that stamps four headers
on **every** response via `headers.insert`:

| Header | Value |
| --- | --- |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `SAMEORIGIN` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |
| `Permissions-Policy` | `camera=(), microphone=(), geolocation=()` |

`headers.insert` (not `if_not_present`) is deliberate: the headers are a
policy, not a default. If an inner service ever sets one of them, the policy
wins.

### Layer position: outermost

The middleware is the **last** `.layer()` call on the router, which makes it
the outermost layer (closest to the client). This is load-bearing:

- `strip_trailing_slash` (one layer in) short-circuits 308 responses without
  calling `next.run`. A security-headers layer placed *inside* it would never
  see the redirect, and the 308 would ship without headers — redirects are a
  common clickjacking vector (transparent redirect iframes).
- The 404 fallback (`ErrorHandler` → `enforce_cache_policy`) similarly never
  passes back through an inner-only security layer for short-circuited paths.

Being outermost means the headers reach HTML, static assets, 308 redirects and
404s alike — verified by `tests/security_headers.rs`.

### `Permissions-Policy` allowlist is closed

`camera=(), microphone=(), geolocation=()` denies all three for every origin.
Any future feature needing one of these capabilities must narrow the policy
explicitly (e.g. `camera=(self)`) rather than blindly widening the denylist.

### Why `Strict-Transport-Security` (HSTS) is NOT one of the four headers

HSTS is intentionally **omitted** from the origin middleware. It is owned by
the Cloudflare edge layer: Cloudflare injects `Strict-Transport-Security` for
zones with Full/Full(Strict) SSL, or explicitly via **SSL/TLS → Edge
Certificates → Always Use HTTPS + HSTS**. Duplicating it at the origin would be
redundant and risks a header conflict if the two policies ever disagree on
`max-age`/`preload`/`includeSubDomains` — a browser receiving conflicting HSTS
values treats the response as malformed and may drop the header entirely,
losing the protection the edge provides.

This mirrors how CSP is handled (deferred, see A1): the header lives at exactly
one layer to keep a single source of truth. If Cloudflare ever stops fronting
the origin (e.g. a direct-origin deploy), HSTS must move into this middleware
at that point.

## Alternatives Considered

### A1: Content-Security-Policy now, with `unsafe-inline`

Rejected as the **sole** header. A CSP with `script-src 'unsafe-inline'`
(which Leptos SSR currently requires for hydration bootstrap) provides almost
no XSS protection — `unsafe-inline` permits any inline script. Shipping the
four headers above is strictly additive and does not block a future CSP. CSP
remains tracked as follow-up work once Leptos nonces / hashes are wired.

### A2: `tower_http::SetResponseHeaderLayer` per header

Considered. Rejected because it stamps the header via `if_not_present`, which
an inner service could override, and because composing four layers is more
verbose than a single `from_fn` that is also easy to unit-test through the
router. The `from_fn` form keeps the policy in one readable function.

### A3: `X-Frame-Options: DENY` instead of `SAMEORIGIN`

Considered. `SAMEORIGIN` was chosen because the landing has no embedding use
case today, but a future legitimate same-origin embed (e.g. a docs preview
iframe) should not require a policy change. `DENY` is stricter but offers no
additional protection against cross-origin clickjacking over `SAMEORIGIN`.

### A4: `frame-ancestors` in a future CSP instead of `X-Frame-Options`

`X-Frame-Options` is deprecated in favour of the `frame-ancestors` CSP
directive. We keep `X-Frame-Options` for now because it is the only
clickjacking defence that works in the absence of a CSP (which we do not yet
ship). When a CSP lands, `frame-ancestors 'self'` should supersede it and
`X-Frame-Options` can be dropped.

## Consequences

### Positive

- Mozilla Observatory / securityheaders.com grades move from F to A-range
  (the four headers are the bulk of the score outside CSP).
- 308 redirects and 404s are no longer a clickjacking/MIME-sniffing blind
  spot.
- Cross-origin navigations leak only the origin, not the path/query.

### Negative

- The middleware runs on every response. The cost is four `HeaderMap::insert`
  calls — negligible, but it is a hot path and must stay allocation-free (the
  values are `from_static`).
- The `Permissions-Policy` denylist must be reviewed whenever a feature
  starts using a device capability.
- A future CSP must be reconciled with `X-Frame-Options` (see A4) to avoid
  contradictory signals.

## References

- ADR-011: URL Canonicalization Policy — documents the `strip_trailing_slash`
  short-circuit that makes the outermost layer position mandatory.
- MDN: `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`,
  `Permissions-Policy`.
- `axum::middleware::from_fn` for the middleware form.
- `tests/security_headers.rs` — covers HTML root, the 308 redirect (decisive
  outermost check), the 404 fallback and a static asset.
