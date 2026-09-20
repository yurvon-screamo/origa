# ADR-059: Delivery via the Aeza VPS (Caddy direct-proxy), AWS decommissioned

## Status

Accepted (cutover 2026-09-19; supersedes the delivery part of ADR-058)

## Date

2026-09-19

## Context

ADR-058 moved Origa's delivery to CloudFront + user-owned Tigris behind Bunny
DNS. Within hours, live traffic from RF residential networks exposed two
platform-level failures of that plan:

1. **HTTP/3 + IPv6**: Chrome from RF failed with `ERR_QUIC_PROTOCOL_ERROR`
   against `http2and3` distributions and hung on AAAA-preferring connections
   (broken v6 routing on RF ISPs). Fixed by `http2`-only and IPv6 off — the
   site worked again, but only until:
2. **TSPU throttling windows**: during evening windows the RF→CloudFront body
   transfer rate degraded to hundreds of B/s while the RF→Aeza(Frankfurt)
   route kept multi-MB/s throughput (measured simultaneously: 263 B/s via CF
   vs 5.5 MB/s via Aeza for the same object). The same window slowed direct
   Tigris too — the throttling targets routes, not vendors.

The attempted fix — Bunny Scriptable DNS geo-steering (RF → Aeza A-record,
world → direct Railway/Tigris CNAMEs) — could not be deployed: Bunny's
production NS never execute SCR records on **A queries** (NXDOMAIN for A,
NOERROR-empty for every other type; script valid in their dashboard sandbox;
record recreated via CLI/REST/UI; confirmed from RU and DE networks and DoH).
A support ticket with a minimal reproducer was filed; `_probe`/`probe2` SCR
records are left in the zone for their engineers.

Meanwhile: the App Review freeze that motivated ADR-058 happened on the
**Aeza-path era** chain (US→Aeza→s3-proxy→Tigris), not on direct
Railway/Tigris — "the world must not touch Aeza" was a misread of that
incident. Tigris geo-replicates on access and the client caches immutable
content for a year, so a CF-class edge cache adds little. Railway has its own
edge network. The owner accepted single-box risk ("infra failures are
out-of-scope for now").

## Decision

1. **All three hostnames** (`s3.origa` / `app.origa` / `origa.uwuwu.net`)
   resolve to a plain A record on the Aeza VPS. Caddy site blocks proxy:
   - `s3.origa` → `https://origa.t3.tigrisfiles.io` (public bucket, no
     s3-proxy container needed — it was decommissioned)
   - `app.origa` / `origa` → Railway (`9fmm6y4e` / `vl080mt6`
     `.up.railway.app`, `header_up Host` is mandatory)
2. **TLS on the VPS**: acme.sh DNS-01 via the Bunny API (`dns_bunny` hook) —
   HTTP-01 cannot pass while Bunny NS also serve the zone; renewal via the
   acme.sh crontab on the VPS. Every site block carries
   `bind 85.192.63.249` (xray owns the second IP, see the Caddyfile header).
3. **AWS is fully decommissioned**: distributions, ACM certificates, IAM
   users/policies removed. The account holds no Origa resources.
4. **DNS stays on Bunny** (instant publication, API, free); `uwuwu.ru` hosts
   the brand 301-redirects via the same Caddy.
5. **Geo-split is LIVE** (same night, after Bunny support pointed out the
   attach-argument mistake — there was no platform bug): SCR records answer
   `s3.origa` / `app.origa` / `origa` with an A-record (Aeza) for RF clients
   and a CNAME for the world (`origa.t3.tigrisbucket.io` for `s3.origa` —
   Tigris custom domain, cert auto-issued by Tigris; `*.up.railway.app` for
   API/landing). No client changes.
   NOTE the CLI footgun: `bunny dns scripts attach <zoneId> <name>` creates
   the record under `<name>` **inside the zone** (i.e. `attach 873317 _probe`
   serves `_probe.uwuwu.net`, NOT `_probe.origa.uwuwu.net`). Verifying the
   wrong hostname cost hours and a false platform-bug conclusion.

## Consequences

- One box (Aeza VPS) is the single delivery path for RF *and* world,
  including App Store review traffic. Accepted by the owner; the box also
  carries personal services and has been stable for years.
- No CDN edge cache anywhere: world users hit Tigris/Railway through the VPS.
  Tigris access-replication + client immutable caching absorb most of it.
- Rollback/geo upgrade path: Bunny records → SCR (script ready), or A-record
  flip anywhere else. TTL 60 on the hot names.
- Cost: $0 beyond the existing VPS and Railway plans.

## Alternatives considered

- Keep CloudFront as world-front / RF-fallback — rejected by the owner: the
  RF-fallback role is fictitious (CF is the thing being throttled), a second
  platform buys nothing while it works, and decommissioning removes a whole
  vendor from the stack.
- Gcore CDN with RF edges — the real fix for RF throttling windows, deferred
  with the geo-split (needs the same DNS steering or client changes).
- Client-side dual-domain failover — rejected earlier (ADR-058): OIDC
  redirect URIs and analytics are domain-bound; zero client changes is a hard
  requirement of this migration.
