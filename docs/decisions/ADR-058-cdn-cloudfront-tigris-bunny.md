# ADR-058: Move Origa delivery to CloudFront + user-owned Tigris origin, DNS to Bunny

## Status

Accepted (cutover 2026-09-19; old Railway bucket retained as backup)

## Date

2026-09-19

## Context

After ADR-057 the delivery chain still routed **every byte of Origa traffic
through one Aeza VPS** (85.192.63.249):

```
Client → Caddy @ Aeza → Railway (API/landing)            [app.origa / origa]
Client → Caddy @ Aeza → docker s3-proxy → Tigris (CDN)   [s3.origa]
```

Facts that motivated the change:

- **App Review (2026-09-18) froze on the loading screen** for 15 minutes with
  "Internet Connection: Active" — the reviewer's traffic died somewhere on the
  single-VPS path; a one-box SPOF in a foreign jurisdiction is a review risk
  and a reliability risk (the same box had already burned us in ADR-049/ADR-057).
- **Aeza DNS panel→NS propagation is broken-slow** (minutes to days,
  documented since July): it stalled ACM validation and every DNS change for
  hours. DNS is a critical path; it must be predictable.
- **No edge caching existed at all** — the VPS proxied every request; Tigris
  paid a Class-B GET per object per cold client.
- The VPS also carries personal services (pass, searxng, VPN) — Origa should
  not share fate with them, and the operator wants the box for his own needs.

Constraints discovered while shopping for the stack:

- **Cloudflare (R2/DNS) is off the table** — TSPU throttling, NS
  unreachability from RF (ADR-046 incident, re-confirmed by a live probe:
  `r2.cloudflarestorage.com` fails TLS from RF with a stale certificate).
- **AWS ACM refuses `.ru`-TLD certificates** (`ADDITIONAL_VERIFICATION_REQUIRED`).
- **Railway's Tigris endpoint cannot serve anonymous reads** (bucket policies
  `NotImplemented`, ACL silently ignored) — a CloudFront origin must be
  publicly readable, so the CDN origin has to move to a user-owned Tigris
  bucket (`origa`, public, `t3.storage.dev`).
- The client application must not be rebuilt: all host names stay identical.

## Decision

1. **One origin for content**: user-owned Tigris bucket `origa`
   (`origa.t3.tigrisfiles.io`), deployed by the same `deploy_cdn.py`
   (profile `[origa-t3]`). The old Railway bucket (`adaptable-foodbox-ucep7wx`
   = Railway `origa-content`) is **retained read-only as a backup**.
2. **CloudFront in front of everything** (three distributions, see
   `infra/cloudfront/`): `s3.origa` (edge cache, immutable years),
   `app.origa` (API, no cache, all methods), `origa` (landing, no cache).
   Hostnames are unchanged — clients and App Review never notice.
3. **DNS zones to Bunny** (`kiki`/`coco.bunny.net`): instant record
   publication, first-class API, free. Both `uwuwu.net` and `uwuwu.ru` zones.
4. **`uwuwu.ru` is a brand redirect only**: `uwuwu.ru`, `www`, `origa.uwuwu.ru`
   → 301 `origa.uwuwu.net` via Caddy on the (kept) VPS. No API/CDN mirrors:
   OIDC redirect URIs are registered for `.net` only, and a separate landing
   host would split Umami analytics.
5. **Aeza VPS stays for personal services**; all Origa routes removed from
   its Caddy, the docker s3-proxy container decommissioned, the Railway
   s3-proxy service deleted.
6. **Deploy transport hardened** (same commit): parallel file upload
   (ThreadPool 24 workers — the 157k small kanji files went from ~6 to
   ~130 files/s), single-PUT up to 8MB (the legacy 24KB single-PUT limit of
   `t3.storageapi.dev` is gone on `t3.storage.dev`, probe-verified).

Cost: ~$0 storage/egress (Tigris free tier 5GB covers the ~4GB corpus),
CloudFront free tier 1TB/mo, Bunny DNS free — the whole delivery stack runs
at effectively $0 plus the existing Railway plan.

## Consequences

- The delivery chain has no RF-jurisdiction box in it: RF users reach
  CloudFront (Frankfurt edge, measured 13–30 MB/s from RF), Apple Review
  reaches US edges, both hit the same origins.
- Edge caching absorbs registration bursts (the Class-B concern): origin GETs
  only happen on cache misses; `manifest.json` stays `no-cache`.
- Railway becomes a pure compute plane (TrailBase + landing); its edge is no
  longer in the data path.
- Rollback path: three DNS records in Bunny back to the VPS A-record
  (TTL 300s) — the VPS routes are gone but can be restored from
  `Caddyfile.bak-20260919`; the backup bucket still holds the full corpus.
- Renewal liability: the `*.uwuwu.ru` ACM import (Let's Encrypt) will expire
  2026-12-18 and is intentionally left to die — nothing serves `.ru` TLS
  anymore. The `.net` certificate is AWS-managed (auto-renew).

## Alternatives considered

- **Cloudflare R2 / DNS** — rejected: RF reachability (live MITM probe).
- **AWS S3 origin + OAC** — viable and canonical, but requires re-uploading
  the corpus to a paid-for bucket (~$0.13/mo) with zero functional gain over
  the existing Tigris bucket; dropped after the Railway→Tigris public-read
  discovery.
- **Gcore CDN** (RF edges in Msk/Spb) — the strongest RF-latency alternative;
  kept as a measured plan B (switch = one CNAME + an evening) if CloudFront
  ever disappoints.
- **HF Storage Buckets as origin** — no anonymous HTTP path (Xet protocol
  only); rejected.
- **Dual-domain client failover** (`.net`/`.ru` with a probe) — rejected:
  touches client code, splits OIDC/analytics domains; the single-CDN route
  with unchanged hostnames achieves the same with zero client changes.
