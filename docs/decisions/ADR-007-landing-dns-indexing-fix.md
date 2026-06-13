# ADR-007: Landing DNS — Cloudflare Migration to Fix Search Engine Indexing (Railway AAAA SERVFAIL)

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

### Aeza DNS Provider Failure

The initial fix attempt was to replace the CNAME with a plain A record on Aeza. During that attempt we discovered that Aeza's DNS zone management is broken: the control panel accepts record changes but does **not** push them to the authoritative nameservers. The SOA serial was stuck at `2026060923` (June 9 — 4 days stale). Neither the panel UI, the Aeza API, nor a delete-and-recreate operation triggered a zone reload. A support ticket (#620117) was opened, but Aeza's estimated response time was ~8 hours, which was unacceptable for unblocking search indexing. This made any Aeza-based workaround non-viable.

## Decision

**Migrate the entire DNS zone from Aeza to Cloudflare.**

Migrate nameservers from Aeza (`ns1-4.aeza-dns.net`) to Cloudflare (`ali.ns.cloudflare.com`, `clay.ns.cloudflare.com`). On Cloudflare, the `origa` record is a CNAME → `c2qj368z.up.railway.app` with **Cloudflare proxy enabled (orange cloud)**. Cloudflare serves its own correct A/AAAA records (anycast IPs), so recursive resolvers never query Railway's broken DNS for AAAA. The SERVFAIL is eliminated permanently.

### Implementation (Cloudflare API)

A scoped Cloudflare API token (Zone DNS Edit + Zone Edit) was used to:

1. Create the zone `uwuwu.net`.
2. Create 8 DNS records (4 CNAME → Railway + 4 TXT `railway-verify`).
3. Delete all legacy records (apex A, smtp, MX, SPF, mail subdomains, DKIM — all confirmed unused).

The nameserver change was made at the registrar (OnlineNIC, Inc.).

### Records on Cloudflare

| Name | Type | Content | Proxy |
|------|------|---------|-------|
| origa | CNAME | c2qj368z.up.railway.app | Proxied (orange) |
| app.origa | CNAME | 9fmm6y4e.up.railway.app | DNS only (grey) |
| s3.origa | CNAME | sltxm1ip.up.railway.app | DNS only (grey) |
| pass | CNAME | vcce37wa.up.railway.app | DNS only (grey) |
| _railway-verify.{origa,app.origa,s3.origa,pass} | TXT | railway-verify=... | — |

## Alternatives Considered

### Plain A Record on Aeza (initial attempt)

- Replace the CNAME with an A record `origa` → `69.46.46.46` on Aeza.
- **Rejected / Failed**: Aeza's zone reload was broken (SOA serial stuck for 4+ days). Changes made in the panel never propagated to the authoritative NS. Not viable until Aeza fixes their infrastructure.

### Aeza ALIAS / CNAME Flattening

- Aeza offers an "ALIAS" record type, but testing confirmed it does **not** flatten CNAME for subdomains — it continues serving a regular CNAME.
- Rejected: does not solve the problem.

### Bug Report to Railway + Wait

- Railway's authoritative NS return an A record in response to AAAA/MX/SOA queries, violating DNS protocol.
- Rejected as primary fix: too slow, no guarantee of resolution timeline. Should still be reported separately.

## Consequences

- **Indexing unblocked**: Cloudflare serves correct A/AAAA (anycast). SERVFAIL eliminated. Yandex and Bing can resolve and crawl the site.
- **No dependence on Railway DNS**: Resolvers query Cloudflare, never Railway. Railway's AAAA bug is irrelevant.
- **No dependence on Aeza**: Fully migrated away. Aeza DNS service no longer needed.
- **Automatic IP tracking**: The Cloudflare proxy resolves the Railway CNAME on its side, automatically tracking Railway edge IP changes. No manual A-record updates required.
- **Bonus**: Cloudflare provides SSL termination, DDoS protection, and caching for the `origa` subdomain (proxied). Other Railway subdomains (app, s3, pass) remain DNS-only to avoid proxy overhead on API/static traffic.
- **SSL mode**: Cloudflare "Full" or "Full (strict)" recommended (Railway already has valid SSL).
- **Token cleanup**: The scoped Cloudflare API token should be revoked after migration is complete.
- **Legacy removed**: All unused records (mail, apex server `31.58.85.8`, MX, SPF, DKIM) were not migrated — confirmed unused by the owner.
