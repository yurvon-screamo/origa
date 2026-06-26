//! Axum router construction for `origa_landing` SSR.
//!
//! Kept in a library module (rather than inline in `main.rs`) so integration
//! tests in `tests/` can build the exact same router the binary serves.

use axum::Router;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Redirect, Response};
use http::header::{CACHE_CONTROL, HeaderName, HeaderValue};
use http::{Method, Request, Uri};
use leptos::config::LeptosOptions;
use leptos_axum::{ErrorHandler, LeptosRoutes};
use tower_http::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

use crate::app::{App, shell};

/// HTML pages: short edge cache so users pick up content/nav changes within
/// 5 minutes, while still letting Cloudflare absorb most origin traffic.
const HTML_CACHE: &str = "public, max-age=300";

/// Hashed/static assets that never change between releases (favicon, OG image,
/// prebuilt CSS, `/images/*`). A year of immutable caching is the standard max
/// for HTTP/1.1 caches.
const IMMUTABLE_CACHE: &str = "public, max-age=31536000, immutable";

/// Crawl-control files (robots.txt, sitemap.xml): search engines must always
/// see the latest copy. Also the forced value on 4xx/5xx error responses so
/// that CDNs never pin a "not found" or server-error page as cacheable —
/// otherwise a later-added file would not be served until the cache expired.
const NO_CACHE: &str = "no-cache";

/// Permanent 308 redirects from `strip_trailing_slash`. A 24h edge cache lets
/// Cloudflare answer the redirect without re-hitting origin on every crawl.
/// Google's SEO guidance for permanent redirects recommends
/// `max-age >= 86400` so link equity transfer is not delayed by re-redirects.
const REDIRECT_CACHE: &str = "public, max-age=86400";

/// Build the production Axum router.
///
/// Layering summary (request flows top → bottom; the LAST `.layer()` call is
/// the outermost, i.e. closest to the client):
///
/// 1. `security_headers` — outermost. Stamps `X-Content-Type-Options`,
///    `X-Frame-Options`, `Referrer-Policy` and `Permissions-Policy` on EVERY
///    response, including the 308 short-circuit from `strip_trailing_slash`
///    and the 404 from the fallback chain. Being outermost is what makes the
///    headers reach redirect/error responses — the inner middlewares
///    short-circuit without calling `next`, so a layer below them would never
///    see the response. See ADR-015.
/// 2. `strip_trailing_slash` — normalises `/ru/`, `/images/`, `/favicon.png/`
///    to their canonical slash-less form. 308 responses produced here carry
///    `REDIRECT_CACHE` directly and short-circuit the inner stack.
/// 3. `enforce_cache_policy` — applies the cache-control contract to every
///    non-redirect response produced by the inner stack:
///    - 2xx without an explicit `Cache-Control` → stamped with `HTML_CACHE`
///      (the default for leptos HTML routes, which set no header themselves).
///    - 2xx with `Cache-Control` → left intact (static assets keep
///      `IMMUTABLE_CACHE`; robots/sitemap keep `NO_CACHE`).
///    - 4xx/5xx → overridden to `NO_CACHE`, regardless of what the inner
///      service set. Without this, `ServeDir`/`ServeFile` would stamp
///      `IMMUTABLE_CACHE` on a 404 via `insert_response_header_if_not_present`
///      (which fires on *all* statuses), and Cloudflare would cache the "not
///      found" for a year — a later-added file would not be served until the
///      cache expired.
///    - 3xx → passed through unchanged. The only 3xx from inner services is
///      `304 Not Modified` on conditional requests, which must keep the same
///      `Cache-Control` the corresponding 200 would have. `308`s from
///      `strip_trailing_slash` never reach this middleware (it is one layer
///      further out and short-circuits the inner stack).
///
/// Fallback chain: when neither explicit static routes nor leptos' registered
/// routes match a path, the request hits `ServeDir(public/)`; if no file is
/// found there either, control falls through to `ErrorHandler`, which renders
/// the App via `leptos_router` (triggering its `<Routes fallback=NotFound>`
/// branch) and stamps HTTP 404 on the response if the App did not override
/// the status itself. This is what makes `/random` return a real 404 with the
/// visible "404" body instead of an empty shell.
///
/// Note on `308 Permanent Redirect`: `Redirect::permanent` in axum 0.8 emits
/// status 308, not 301. Both are treated as permanent by Googlebot/Yandex/Bing
/// and pass full link equity.
pub fn build_router(leptos_options: LeptosOptions) -> Router {
    let pkg_dir = env!("CARGO_MANIFEST_DIR");
    let routes = leptos_axum::generate_route_list(App);

    let public_dir = format!("{pkg_dir}/public");
    let error_handler = ErrorHandler::new(shell, leptos_options.clone());

    Router::new()
        .nest_service(
            "/images",
            ServeDir::new(format!("{public_dir}/images")).insert_response_header_if_not_present(
                CACHE_CONTROL,
                HeaderValue::from_static(IMMUTABLE_CACHE),
            ),
        )
        .route_service(
            "/landing.processed.css",
            ServeFile::new(format!("{pkg_dir}/style/landing.processed.css"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(IMMUTABLE_CACHE),
                ),
        )
        .route_service(
            "/favicon.png",
            ServeFile::new(format!("{public_dir}/favicon.png"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(IMMUTABLE_CACHE),
                ),
        )
        .route_service(
            "/favicon.ico",
            ServeFile::new(format!("{public_dir}/favicon.ico"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(IMMUTABLE_CACHE),
                ),
        )
        .route_service(
            "/apple-touch-icon.png",
            ServeFile::new(format!("{public_dir}/apple-touch-icon.png"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(IMMUTABLE_CACHE),
                ),
        )
        .route_service(
            "/browserconfig.xml",
            ServeFile::new(format!("{public_dir}/browserconfig.xml"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(IMMUTABLE_CACHE),
                ),
        )
        .route_service(
            "/robots.txt",
            ServeFile::new(format!("{public_dir}/robots.txt"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(NO_CACHE),
                ),
        )
        .route_service(
            "/sitemap.xml",
            ServeFile::new(format!("{public_dir}/sitemap.xml"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(NO_CACHE),
                ),
        )
        .route_service(
            "/llms.txt",
            ServeFile::new(format!("{public_dir}/llms.txt")).insert_response_header_if_not_present(
                CACHE_CONTROL,
                HeaderValue::from_static(NO_CACHE),
            ),
        )
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback_service(
            ServeDir::new(&public_dir)
                .append_index_html_on_directories(false)
                .fallback(error_handler),
        )
        // Inner cache-policy layer. Replaces the previous
        // `SetResponseHeaderLayer::if_not_present(HTML_CACHE)` with a function
        // that additionally overrides `Cache-Control` to `NO_CACHE` on 4xx/5xx.
        // See `enforce_cache_policy` below for the full rationale.
        .layer(middleware::from_fn(enforce_cache_policy))
        .layer(middleware::from_fn(strip_trailing_slash))
        .layer(middleware::from_fn(security_headers))
        .with_state(leptos_options)
}

/// Cache-policy middleware applied to every non-redirect response.
///
/// Replaces the old outermost `SetResponseHeaderLayer::if_not_present(HTML)`
/// with status-aware logic. The per-service
/// `insert_response_header_if_not_present` calls on `ServeDir`/`ServeFile`
/// remain in place — they stamp `IMMUTABLE_CACHE`/`NO_CACHE` on successful
/// responses — but they fire on *all* statuses including 404, so this
/// middleware post-processes the result:
///
/// - 4xx/5xx → forced to `NO_CACHE`. This is the fix for the SEO issue where
///   `/images/nonexistent.png` returned 404 with `IMMUTABLE_CACHE`, causing
///   Cloudflare to cache "not found" for a year and block any later-added
///   file from being served until the cache expired.
/// - 2xx without `Cache-Control` → stamped with `HTML_CACHE` (the default for
///   leptos HTML routes, which do not set their own header).
/// - 2xx/3xx with `Cache-Control` already present → left intact, so static
///   assets keep `IMMUTABLE_CACHE`, robots/sitemap keep `NO_CACHE`, and
///   `304 Not Modified` preserves the same `Cache-Control` as the 200 would
///   have (required by HTTP conditional-cache semantics).
///
/// `308 Permanent Redirect` responses from `strip_trailing_slash` never reach
/// this middleware: they are produced by a layer *outside* this one, which
/// short-circuits the inner stack.
async fn enforce_cache_policy(request: Request<axum::body::Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let status = response.status();

    if status.is_client_error() || status.is_server_error() {
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(NO_CACHE));
    } else if status.is_success() && !response.headers().contains_key(CACHE_CONTROL) {
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(HTML_CACHE));
    }

    response
}

/// Middleware: redirect `/path/` to `/path` with HTTP 308 Permanent Redirect.
///
/// Canonical URLs for `origa_landing` have no trailing slash (e.g. `/ru`,
/// `/ru/features`). The site root `/` is the sole exception and keeps its
/// slash by convention.
///
/// - GET/HEAD only: other methods are passed through verbatim so the redirect
///   cannot mask an accidental POST to a slash-suffixed URL. This mirrors
///   RFC 7231 §6.4 redirect semantics.
/// - Query string is preserved (relative `Location`).
/// - Multiple trailing slashes collapse to one canonical form, e.g.
///   `/ru//` → `/ru`.
/// - The 308 response carries `Cache-Control: public, max-age=86400`
///   (`REDIRECT_CACHE`) so edge caches serve the redirect without re-hitting
///   origin on every crawl. This layer is the outermost one and short-circuits
///   the inner stack, so `enforce_cache_policy` cannot apply the header — it
///   must be set here directly.
async fn strip_trailing_slash(
    method: Method,
    uri: Uri,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return next.run(request).await;
    }
    let path = uri.path();
    if path.len() <= 1 || !path.ends_with('/') {
        return next.run(request).await;
    }
    let stripped = path.trim_end_matches('/');
    let stripped = if stripped.is_empty() { "/" } else { stripped };
    let location = match uri.query() {
        Some(q) if !q.is_empty() => format!("{stripped}?{q}"),
        _ => stripped.to_string(),
    };
    let mut response = Redirect::permanent(&location).into_response();
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(REDIRECT_CACHE));
    response
}

/// Outermost security-headers middleware. Stamps four defensive headers on
/// every response the server emits — HTML pages, static assets, the 308
/// redirect from `strip_trailing_slash`, and the 404 from the fallback chain.
///
/// Being outermost is load-bearing: `strip_trailing_slash` short-circuits 308
/// responses without calling `next`, so a layer further in would never see
/// them and the redirect would ship without the headers. See ADR-015 for the
/// header-by-header rationale and the CSP trade-off.
///
/// `Permissions-Policy` locks down the three device capabilities Origa never
/// uses (camera, microphone, geolocation); any future feature needing one
/// must narrow this allowlist explicitly rather than widening it blindly.
async fn security_headers(request: Request<axum::body::Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("SAMEORIGIN"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    response
}
