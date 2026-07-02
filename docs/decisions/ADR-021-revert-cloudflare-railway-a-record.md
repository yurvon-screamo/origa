# ADR-021: Revert Cloudflare Proxy — Return to Aeza NS + Railway Edge A-record

## Status

Accepted

## Date

2026-06-28

## Context

ADR-007 (2026-06-13) resolved the landing DNS indexing failure by replacing a CNAME → Railway with a plain A-record → Railway edge IP `69.46.46.46` on Aeza authoritative NS. A subsequent update (ADR-007, 2026-06-24) replaced that workaround with a full Cloudflare proxy: the `uwuwu.net` zone was migrated to Cloudflare authoritative NS (`ali.ns.cloudflare.com`, `clay.ns.cloudflare.com`), and `origa.uwuwu.net` became a proxied CNAME → `c2qj368z.up.railway.app`. The intent was to add DDoS protection, automatic SSL, and IPv6 reachability while continuing to hide Railway's AAAA SERVFAIL bug behind the Cloudflare proxy.

Despite the Cloudflare migration:

- **Yandex Webmaster** continued to report *"Не удалось подключиться к серверу из-за ошибки DNS"* and *"изменений нет"*, with only 1 page in the search index, for ~1.5 months after the migration.
- **Google Search Console** showed `Discovered — currently not indexed` for `/features`, `/compare`, `/content`, `/download`.

Independent diagnosis (2026-06-28) confirmed that, from the public internet, all major crawlers received `200 OK` with full SSR HTML through Cloudflare:

- `Googlebot/2.1`, `YandexBot/3.0`, `bingbot/2.0` → `200 OK`, real Leptos SSR body, `cf-cache-status: DYNAMIC`, no `cf-mitigated`, no challenge page.
- `robots.txt`, `sitemap.xml` — 200, crawlable.
- Direct bypass to Railway origin (`69.46.46.46`) — `200 OK`, `server: railway-hikari`.

This created a contradiction: crawlers receive 200 SSR HTML from public probes, yet Yandex Webmaster reports a DNS-layer failure. Two non-mutually-exclusive hypotheses were considered:

1. YandexBot's internal recursive resolver cached the pre-2026-06-24 SERVFAIL and held it via RFC 2308 negative caching. The 1.5-month duration makes this weak but not impossible.
2. Cloudflare applies bot-management logic to real crawler IPs (verified-bot allowlist, IP reputation, bot fight mode) that differs from what a synthetic `curl -A YandexBot` probe sees. Cloudflare's public docs confirm it distinguishes verified bots by reverse-DNS/IP, not by User-Agent alone.

The product is dev-stage with no production users, so downtime risk is acceptable. The user values a simpler stack over defensive layers that do not appear to be helping.

## Decision

Revert to the original ADR-007 (2026-06-13) solution: Aeza as authoritative NS for `uwuwu.net`, with `origa.uwuwu.net` as a plain **A-record → `69.46.46.46`** (Railway edge IP, TTL 300). Cloudflare is removed from the DNS path entirely.

Concrete actions taken (2026-06-28, autonomous):

1. Backed up the Cloudflare zone (8 records) to `docs/backups/uwuwu.net.cloudflare.2026-06-28.zone`.
2. Verified the Aeza zone was already populated with all 8 records (preserved from the pre-Cloudflare configuration).
3. Toggled "Use custom NS servers" OFF in the Aeza panel (`my.aeza.net/services/1095525/ns`), returning delegation to `ns1–ns4.aeza-dns.net`.
4. Independent verification confirmed: `.net` TLD nameservers now delegate to Aeza; `origa.uwuwu.net` A = `69.46.46.46`; AAAA = empty NOERROR (SERVFAIL eliminated at the authoritative level).

## Alternatives Considered

### Keep Cloudflare (status quo before this ADR)

- Pros: DDoS protection, edge caching, automatic SSL, IPv6 reachability.
- Cons: Did not resolve the Yandex indexing failure; adds an extra hop and a non-trivial bot-management layer that we cannot fully inspect; user explicitly wants the stack simplified.
- Rejected: 1.5 months of no observable improvement invalidated the original rationale.

### Migrate to Vercel (original user request)

- The landing is a long-running Rust Axum + Leptos SSR server (`#[tokio::main]`, `axum::serve`). Vercel does not natively host long-running Rust binaries; only community `vercel-rust` serverless wrappers exist.
- A serverless rewrite would introduce cold-start latency on every cold SSR request, which is itself hostile to crawler indexing.
- Rejected: architecturally mismatched; the existing Docker image cannot be deployed to Vercel as-is.

### Fly.io / Render (Docker-friendly PaaS)

- Would accept the existing Docker image with no code changes and provide a simpler stack than Cloudflare + Railway.
- Deferred: out of scope for this round. The user chose to stay on Railway and simplify the DNS layer first. Reconsider if Railway becomes unreliable.

### Use Railway as authoritative NS for `uwuwu.net`

- Not possible. Railway is not a DNS provider for custom zones; it exposes only CNAME targets (`*.up.railway.app`) and an edge IP. Authoritative NS must live on a real DNS host (Aeza / Cloudflare / other).
- Rejected: technically impossible.

## Consequences

**Positive:**

- SERVFAIL on `origa.uwuwu.net` AAAA is eliminated at the authoritative level (verified independently via `ns1/ns2.aeza-dns.net` and `.net` TLD delegation). Strict resolvers (Yandex, Bing per ADR-007) no longer see a CNAME chain to Railway's broken AAAA for the landing host.
- Cloudflare is removed as a variable. If Yandex indexing recovers, we have isolated the cause. If it does not, we have eliminated DNS/hosting as the root cause and can focus on SEO factors.
- Simpler stack: one fewer vendor, one fewer proxy hop, one fewer place where bot-management can silently interfere with crawlers.

**Negative / risk:**

- No Cloudflare DDoS protection or WAF. Acceptable while dev-stage.
- No Cloudflare edge cache: every request hits Railway origin. The Axum server's own `Cache-Control` headers (`HTML_CACHE`, `IMMUTABLE_CACHE`, `NO_CACHE`) still apply, but there is no shared edge cache between clients.
- No IPv6 reachability for the landing (no AAAA record). Acceptable per ADR-007 — the previous workaround had the same limitation.
- Single point of failure: if Railway changes its edge IP away from `69.46.46.46`, the Aeza A-record must be updated manually. Same risk as the original 2026-06-13 workaround.

**Known non-regression:** `app.origa.uwuwu.net`, `s3.origa.uwuwu.net`, and `pass.uwuwu.net` remain CNAMEs to Railway and continue to exhibit AAAA SERVFAIL via Google Public DNS. This is the same Railway DNS protocol bug documented in ADR-007 and pre-dates this ADR. It does not affect the landing host and does not affect Tauri desktop clients (which connect over IPv4).

**Railway-side Cloudflare note:** The Railway edge IP `69.46.46.46` itself sits behind a Cloudflare CDN front-end (HTTP responses carry `server: cloudflare` and `X-Railway-Edge: osl1`). This is internal to Railway's infrastructure and not under our control; our domain is no longer delegated to Cloudflare, which was the goal.

## Verification (2026-06-28)

| Check | Method | Result |
| --- | --- | --- |
| TLD registry delegation | `nslookup -type=NS uwuwu.net a.gtld-servers.net` and `m.gtld-servers.net` | `ns1–ns4.aeza-dns.net` ✅ |
| Aeza NS: `origa` A | `nslookup -type=A origa.uwuwu.net ns1.aeza-dns.net` | `69.46.46.46` ✅ |
| Aeza NS: `origa` AAAA | `nslookup -type=AAAA origa.uwuwu.net ns1.aeza-dns.net` | Empty NOERROR ✅ (no SERVFAIL) |
| Backup integrity | File listing | 3/3 files present (zone, public snapshot, runbook) |

Public-resolver propagation was partial at verification time (Cloudflare DoH updated, Google DoH still caching old NS for up to 6 h) — this is expected and does not affect the authoritative-level correctness.

## Open Questions (not resolved by this ADR)

1. **Did this actually fix Yandex indexing?** Not yet known. Requires Yandex Webmaster re-crawl request + 1–2 weeks of observation. ADR to be updated with outcome.
2. **Google "Discovered — currently not indexed"** is unchanged by this ADR — it is an algorithmic decision unrelated to DNS. Resolution requires backlinks and time, not infrastructure changes. See `marketing/strategies/origa-seo.md` (status: 0 backlinks, 1 GitHub star).

## References

- ADR-007 (superseded in part — the 2026-06-24 Cloudflare update is reverted by this ADR; the original 2026-06-13 A-record workaround is reinstated)
- `docs/backups/uwuwu.net.cloudflare.2026-06-28.zone` — Cloudflare zone backup
- `docs/runbooks/teardown-cloudflare-to-railway.md` — execution runbook + rollback
