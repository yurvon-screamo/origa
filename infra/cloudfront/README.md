# CloudFront distributions (2026-09-19 migration)

Live configuration of the three production distributions (account
310356785854, `us-east-1` for ACM + distributions). The JSONs are the exact
`distribution-config` payloads used to create them — reusable for
`aws cloudfront create-distribution` after swapping `CallerReference`.

| File | Distribution ID | CNAME | Origin |
|---|---|---|---|
| `cf-dist-cdn-net.json` | E21MVKWQP036OX | `s3.origa.uwuwu.net` | `origa.t3.tigrisfiles.io` (Tigris bucket `origa`, public) |
| `cf-dist-api-net.json` | E246LJ2I68OZBT | `app.origa.uwuwu.net` | `9fmm6y4e.up.railway.app` (TrailBase) |
| `cf-dist-landing-net.json` | E1PUUG8XMRV8EV | `origa.uwuwu.net` | `c2qj368z.up.railway.app` (landing) |

## Shared building blocks (account-level, not in these files)

- Origin request policy `b2abc271-2c2f-4d18-866a-2be80778061f`
  (`origa-forward-host`) — forwards the viewer `Host` to the origin.
  **Required** for Railway origins: they route by `Host` and answer 404
  "Application not found" when CloudFront substitutes its own origin domain.
- Cache policy `87c6b7bc-79dd-4aae-aabe-c2645659ff20`
  (`origa-manifest-nocache`) — TTL 0, for `manifest.json` and API paths.
- Default behavior of the CDN distribution uses the managed
  `CachingOptimized` policy (`658327ea-...`); the origin serves the tiered
  `Cache-Control` (immutable / 300s / no-cache) set by `deploy_cdn.py`.

## Gotchas baked into these configs (learned the hard way, see ADR-058)

- `OriginSslProtocols` accepts up to TLSv1.2 only (TLSv1.3 is not in the enum).
- Min/Default/Max TTL cannot be set on a behavior that has a cache policy.
- The managed `CachingDisabled` policy is not attachable via API — use the
  custom `origa-manifest-nocache` instead.
- One certificate per distribution; `*.uwuwu.net` does NOT cover
  `s3.origa.uwuwu.net` (wildcard matches one level) — the live cert is
  `*.origa.uwuwu.net` + `origa.uwuwu.net`.
- Origin read timeout max is 60s.

## 2026-09-19 hotfix

- `HttpVersion` is `http2` (not `http2and3`): TSPU blocks QUIC/UDP-443 in RF and
  Chrome fails with `ERR_QUIC_PROTOCOL_ERROR` without falling back fast. curl
  never catches this (no HTTP/3 support) — browser-only symptom.
