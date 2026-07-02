# ADR-007: Landing DNS — Replace CNAME with A Record to Fix Search Engine Indexing (Railway AAAA SERVFAIL)

## Status

Accepted

## Date

2026-06-13

## Context

The landing site `origa.uwuwu.net` (deployed on Railway via custom domain, DNS managed on Aeza at `ns1/ns2.aezadns.com`) was completely failing to index in Yandex and Bing:

- **Yandex**: "Failed to connect to server due to DNS error" — the bot could not resolve the domain.
- **Bing**: "Discovered but not crawled", "Crawl Allowed: No", "Page Fetch: Unsuccessful".

### Root Cause

Railway's authoritative nameservers for `*.up.railway.app` violate the DNS protocol. When queried for an `AAAA` (IPv6) record, they return an `A` (IPv4) record instead of a proper AAAA response or an empty NOERROR. Strict recursive resolvers (Google Public DNS, Yandex DNS, Bing) detect this as a bogus/suspicious response and return `SERVFAIL` (rcode 2).

The domain was configured as a CNAME: `origa.uwuwu.net` → `c2qj368z.up.railway.app`. When a resolver follows this CNAME and queries Railway for AAAA, it gets SERVFAIL. Some resolvers (notably Yandex's, and per RFC 6724 IPv6-first behavior) do not fall back to the A record after a SERVFAIL on AAAA, so for them the domain appears completely unresolvable.

Evidence (DNS-over-HTTPS via Google):

- `A` query → CNAME → `69.46.46.46` (Status 0, NOERROR) ✅
- `AAAA` query → Status 2 SERVFAIL, EDE: "Unexpected c2qj368z.up.railway.app/a in received ANSWER at up.railway.app for c2qj368z.up.railway.app/aaaa" ❌

### What Was NOT the Problem

All of the following were verified correct and ruled out:

- `robots.txt` — `User-agent: * / Allow: /` with sitemap reference ✅
- `sitemap.xml` — valid, 5 pages × 5 languages (en/ru/ko/vi/x-default) ✅
- Meta tags — no `noindex`, canonical present, OG/Twitter/JSON-LD structured data, Yandex/Google/Bing verification tags ✅
- HTTP responses — 200 OK on `/`, `/robots.txt`, `/sitemap.xml` ✅
- SSL certificate — valid ✅
- Aeza nameservers — correctly serving the CNAME ✅

## Decision

Replace the CNAME record `origa` → `c2qj368z.up.railway.app` with a plain **A record**: `origa` → `69.46.46.46` (Railway edge IP, TTL 300).

### Rationale

A plain A record on the authoritative nameserver (Aeza) means recursive resolvers never follow a CNAME chain to Railway's DNS. The AAAA query returns a clean empty NOERROR (no IPv6 configured) instead of SERVFAIL. This eliminates the broken Railway DNS from the resolution path entirely.

Railway routes custom domains by SNI/Host header at the edge, not by the DNS CNAME chain. This was verified: a direct connection to `69.46.46.46` with `SNI: origa.uwuwu.net` returns `200 OK` with valid SSL and full SSR HTML content. So a plain A record works correctly.

### Note on Aeza ALIAS

Aeza offers an "ALIAS" record type, but testing confirmed it does NOT perform CNAME flattening for subdomains — it continues to serve the record as a regular CNAME, so it does not solve the problem.

## Alternatives Considered

### CNAME Flattening via ALIAS/ANAME (Aeza)

- Tried first: changed CNAME → ALIAS in the Aeza panel.
- Result: Aeza's ALIAS for subdomains still serves a CNAME to clients. The authoritative NS continued returning `canonical name = c2qj368z.up.railway.app`. SERVFAIL persisted.
- Rejected: does not actually flatten for subdomains.

### Cloudflare Proxy

Put Cloudflare in front of the domain; Cloudflare serves correct A/AAAA records and proxies to Railway.

- **Pros**: most robust — Cloudflare's anycast network, automatic SSL, caching, DDoS protection; Railway edge IP changes are handled automatically.
- **Cons**: requires either migrating the entire zone's NS to Cloudflare (risky — affects all subdomains: app, s3, etc.) or setting up Cloudflare for SaaS for a single subdomain (complex, may require paid plan).
- **Deferred**: viable long-term upgrade, but higher operational cost. The A record is sufficient for now.

### Bug Report to Railway + Wait

Report that Railway's authoritative NS return an A record in response to AAAA/MX/SOA queries for `*.up.railway.app`, violating DNS protocol.

- Rejected as primary fix: too slow, no guarantee of resolution timeline. Should still be reported separately.

## Consequences

- **Indexing unblocked**: AAAA queries now return clean NOERROR instead of SERVFAIL. Yandex and Bing can resolve and crawl the site.
- **Single point of IP**: The A record points to one Railway edge IP (`69.46.46.46`). If Railway changes their edge IP, the record must be updated manually. Railway edge IPs are relatively stable (TTL 60s on their side), but this requires monitoring.
- **No IPv6**: The site has no AAAA record. IPv6-only clients cannot reach it. Acceptable for now — Railway edge is IPv4.
- **Future migration to Cloudflare recommended**: If the A-record maintenance burden grows or Railway edge IPs change frequently, migrate DNS to Cloudflare for automatic CNAME flattening and anycast resilience.
- **Monitoring**: Periodically verify that `69.46.46.46` still resolves for `c2qj368z.up.railway.app` and that the site responds 200 OK. If the IP changes, update the A record in the Aeza panel.

## Update (2026-06-24): Cloudflare Proxy Implemented

The "Cloudflare Proxy" alternative listed above as **Deferred** (long-term upgrade)
has since been **implemented**. The `uwuwu.net` zone was migrated from Aeza
nameservers to Cloudflare authoritative NS (`ali.ns.cloudflare.com`,
`clay.ns.cloudflare.com`). `origa.uwuwu.net` now resolves to Cloudflare proxy IPs
(`104.21.39.177`, `172.67.147.175`) with valid AAAA records, and Cloudflare forwards
to the Railway origin over IPv4.

This supersedes the plain-A-record workaround as the active solution:

- AAAA queries now return clean NOERROR with real Cloudflare IPv6 — SERVFAIL is no
  longer reachable for `origa.uwuwu.net` by any resolver.
- Railway's broken AAAA behaviour is still present (`c2qj368z.up.railway.app` AAAA →
  SERVFAIL, no IPv6 on Railway origin), but it is **hidden behind the Cloudflare
  proxy** and no longer affects the public domain.

**⚠️ Operational warning:** Cloudflare is now the **only** layer shielding the domain
from the Railway DNS protocol bug. If the zone is ever moved back off Cloudflare, or
`origa.uwuwu.net` is re-pointed directly at Railway via CNAME, the SERVFAIL regression
returns immediately. Keep the domain behind Cloudflare.

Verified 2026-06-24: A and AAAA queries via Yandex DNS (77.88.8.8), Google (8.8.8.8),
Cloudflare (1.1.1.1), Quad9 (9.9.9.9), and the authoritative Cloudflare NS all return
NOERROR with valid records. `robots.txt` is clean (no AI-Audit block), and
`Google-Extended`/`YandexBot` user-agents receive HTTP 200 with full SSR HTML.

## Update (2026-06-28): Cloudflare Proxy Reverted — see ADR-021

The 2026-06-24 Cloudflare Proxy implementation documented above was reverted on
2026-06-28. After ~1.5 months with Cloudflare in front of the domain, Yandex
Webmaster continued to report *"Не удалось подключиться к серверу из-за ошибки DNS"*
and Google Search Console showed `Discovered — currently not indexed` — neither
was resolved by the Cloudflare migration.

The original 2026-06-13 workaround (Aeza authoritative NS + plain A-record →
`69.46.46.46`) is reinstated. Authoritative NS is again `ns1–ns4.aeza-dns.net`.
The 2026-06-24 "Operational warning" above no longer applies — Cloudflare is no
longer in the path, and the A-record approach independently eliminates SERVFAIL
for `origa.uwuwu.net` (verified via Aeza NS direct queries on 2026-06-28).

See **ADR-021** for the full rationale, alternatives considered (including Vercel,
rejected as architecturally mismatched for the long-running Rust Axum SSR server),
and verification details.
