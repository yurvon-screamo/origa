# ADR-049: RF TSPU Block of Railway Edge Subnet — Temporary VPS Reverse Proxy

## Status

Accepted (temporary mitigation; to be superseded by migration off Railway or unblocking)

## Date

2026-09-13

## Context

On 2026-09-12 all `uwuwu.net` hosts on Railway became unreachable from Russia again — this time
**with a healthy DNS layer** (Aeza NS answered, RU resolvers resolved the CNAME chain fine,
including Yandex DNS 77.88.8.8). This **confirms the alternative hypothesis recorded in
ADR-046** ("the real RU blocker may be an IP-range block on Railway edge") — its falsification
criterion fired: zone resolves from RU, HTTPS does not.

Diagnosis (live, from an MGTS vantage point and 55 check-host.net nodes):

- ICMP and TCP handshakes to Railway edge IPs `69.46.46.46`/`69.46.46.55`/`69.46.46.59`
  **succeed**; the entire application payload (TLS ClientHello with any SNI, HTTP :80) is
  **silently dropped** → browsers see `ERR_TIMED_OUT`, no RKN stub page. This is TSPU
  "extended-list" behavior outside the public registry.
- Foreign vantage points: HTTP 200 in <1 s for all hosts (services healthy).
- The block **rolled out progressively per ISP** during the observation window: initially only
  `origa.uwuwu.net` (.46) was dead while `landing-production-c0a2.up.railway.app` (.59, same
  /24, different edge IP) still worked; ~30 minutes later .59 died too. Some RU networks
  (check-host `ru3`) still reached Railway directly at that moment.
- Conclusion: the whole Railway edge subnet `69.46.46.0/24` is being dropped by TSPU,
  ISP by ISP. Nothing fixable on the Railway side (single shared anycast edge).

Affected: `origa.uwuwu.net`, `app.origa.uwuwu.net`, `s3.origa.uwuwu.net`,
`pass.uwuwu.net` (all CNAME → `*.up.railway.app` per ADR-046). `cdn.origa` (Tigris,
ADR-037/046 leftover) is **not** affected and was deliberately left as is.

### Options considered

| Option | Verdict |
| --- | --- |
| Wait / do nothing | RU audience (the primary SEO target) lost |
| Cloudflare proxy | Rejected — TSPU throttles CF-fronted routes from RU since 2025 (ADR-037/046 precedent) |
| RU CDN (DDoS-Guard, StormWall, Yandex Cloud CDN) | Viable but RU-jurisdiction TLS termination for `pass.uwuwu.net` (password manager) is unacceptable; RU-edge → Railway egress crosses TSPU itself |
| bunny.net / foreign CDN | Architecturally sound, but RU-unfriendly billing; deferred |
| **Own VPS reverse proxy (EU)** | Chosen — VPS already owned (Aeza Frankfurt, `138.124.61.86`), free, fully controlled, RU users reach EU IPs fine |
| Migrate all services off Railway | The end-state; too heavy for an incident window — this ADR is the bridge |

## Decision

Route all four public hosts through a **temporary L7 reverse proxy on the user's existing
VPS** (Aeza Frankfurt, Ubuntu 24.04, `138.124.61.86`) and switch DNS to it. Railway remains
the origin; rollback is a DNS change (TTL 300).

Architecture:

```
RU user ──→ Caddy @ 85.192.63.249:443 (VPS, Frankfurt, LE certs) ──→ Railway edge (*.up.railway.app)
             L7 reverse_proxy                                    DE→DE leg, never crosses RF border
```

Key implementation facts (all verified in production 2026-09-13):

1. **Second IP, not a second server.** The VPS already had a second IPv4 `85.192.63.249`;
   xray (VLESS+Reality VPN, inbound `vless-reality-tcp-443`) held `*:443`. Its `listen` was
   rebound `0.0.0.0 → 138.124.61.86` in the x-ui DB (backup
   `/etc/x-ui/x-ui.db.bak.20260912-230218`) — VPN clients (server address `138.124.61.86`)
   unaffected, verified via Reality handshake post-change. Caddy binds `85.192.63.249:443/:80`
   (auto HTTP→HTTPS redirect inherits the site `bind`; wildcard :80 not taken).
2. **Caddy → Railway requires two non-obvious directives** (root cause of a long debugging
   session; both are now in the `uwuwu_site` snippet in the Caddyfile):
   - `header_up Host {host}` — **Railway routes by the Host header**, while Caddy v2.11
     *replaces* Host with the upstream dial address (original goes to `X-Forwarded-Host`).
     Without this, Railway answers `x-railway-fallback: 404 "Application not found"`
     regardless of correct SNI.
   - `transport http { tls_server_name <site domain> }` — edge must see the site's SNI to
     serve its cert and route; with the upstream hostname as SNI you get the
     `*.up.railway.app` cert + the same fallback 404.
   - `header_up X-Forwarded-For {remote_host}` — overwrite, not append: Caddy is the only
     trusted hop; client-supplied XFF must not reach the apps (matters for `pass` audit).
3. **DNS cutover via Aeza API v2** (X-API-Key; domain ID 2776): four records CNAME → A
   `85.192.63.249`, TTL 300, canary order (`origa` first, then the rest). Zone dumped before
   the change (`zone-backup-20260913-000457.json`). Propagation on the new
   `a`/`b.aeza-dns.net` fleet: ~1.5 min (notably faster than the 48 h feared in ADR-046 —
   that figure was observed on the old `ns1–4` fleet).
4. **LE certificates are issued only after DNS points at the proxy** (http-01/tls-alpn-01
   otherwise hit Railway). Rate limit: 5 failed authorizations/hour/domain — do not
   restart/reload Caddy in a loop while DNS still points to Railway; canary + single restart.
5. **Monitoring must probe from behind TSPU** (the DE-hosted proxy sees nothing when RU is
   blocked): `uwuwu-check.sh` on the VPS polls check-host.net RU nodes every 15 min (cron),
   OK = ≥2 RU nodes answering 200/404(S3 root)/401; alerts via configurable webhook
   (`/etc/uwuwu-check.conf`, OK→FAILED and back), plus a dead-man-switch ping
   (`HEALTHCHECK_URL`) because a monitor living on the monitored host cannot detect the
   host's own death.
6. Infrastructure-as-code artifacts live in the `uwuwu-proxy-infra` git repo (working copy
   `~/uwuwu-proxy-infra/`, offsite bare copy `/root/uwuwu-proxy-infra.git` on the VPS):
   Caddyfile, runbook (deploy/rollback/plan-B), monitor script, logrotate, zone backup.

## Consequences

**Positive:**

- All four hosts verified 200 OK from RU (MGTS — the most aggressive TSPU node observed) and
  from 55 worldwide check-host nodes, 0.25–0.9 s via the proxy. `pass.uwuwu.net` (Bitwarden)
  is reachable from RU again; VPN (xray) and searxng on the same VPS untouched and verified.
- Railway stays the origin: zero application changes; world traffic also flows through the
  proxy (acceptable at current volumes) — one DNS flip (TTL 300) restores the previous
  state completely.

**Negative / accepted risks:**

- **Single VPS = single point of failure for the whole mitigation** (and for the world now,
  not just RU). Aeza subnets periodically end up on TSPU lists (hoster popular with VPNs).
  Plan-B (new-IP pre-switch validation from RU nodes; hoster migration with the same
  Caddyfile) is documented in the runbook.
- **TLS for `pass.uwuwu.net` is terminated on the VPS** (hoster-visible session cookies /
  metadata on that hop; vault contents remain client-side encrypted). Consciously accepted
  for the temporary period; alternative (VPN-only access) documented as rollback.
- Monitoring coverage is check-host RU nodes only (hosting networks): a full per-ISP block
  is detected (all RU nodes fail), a single-ISP partial block may not be.
- Added latency for non-RU users (EU hop) and one more moving part to renew/maintain
  (certs auto-renew via Caddy; monitor/alert config is manual).
- `cdn.origa` remains a dangling ADR-037/ADR-046 leftover on Tigris (unaffected by this
  incident); its cleanup decision (ADR-046 Follow-up 3) stays open.

**Explicitly not done (scope discipline):** no migration off Railway, no RU CDN contracts,
no changes to Railway custom-domain registration (SNI/Host routes there remain valid for
rollback), no changes to `cdn.origa`.

## Rollback

Full procedure in `uwuwu-proxy-infra/README.md` (canonical; bare repo mirrors it on the VPS):
revert commit in the repo → deploy via the validate-then-replace procedure → Aeza API PATCH
each record back to CNAME `*.up.railway.app` (record ids in the runbook; API key location:
`uwuwu-cli read origa/aeza-dns-api`). Caddy site blocks for reverted domains must be removed
in the same change, otherwise LE renewals burn authorizations against Railway-fronted DNS.

## Follow-ups

1. **Strategic decision required:** migrate public hosting off Railway (the /24 block is
   unlikely to be lifted) vs keep the proxy long-term. The proxy was built as a bridge, not
   a destination.
2. Enable `ALERT_WEBHOOK` (and optionally `HEALTHCHECK_URL`) in `/etc/uwuwu-check.conf` on
   the VPS; verify with `uwuwu-check.sh --test-alert`.
3. ADR-046 Follow-up 5 (watch Yandex/GSC for DNS or indexing regressions) now also covers
   the proxy hop; re-baseline after 2–4 weeks.
4. Rotate the VPS root password if it was shared over insecure channels during the incident
   (done once during setup 2026-09-13; owner should store the current one in the password
   manager).
5. Wiki article `experience/hosting/railway-rf-block.md` (pending approval queue) covers the
   TSPU diagnosis methodology; cross-check after publication.
