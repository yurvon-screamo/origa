# ADR-061: RF Delivery Branch Moves to an RF VM (nginx SNI-Demux)

## Status

Accepted (cutover 2026-09-30; supersedes the RF-branch part of ADR-049/ADR-059 delivery chain)

## Date

2026-09-30

## Context

The ADR-049 mitigation (RF users → Caddy on Aeza Frankfurt → Railway/Tigris)
failed on 2026-09-29: RKN blocked the box for RF **entirely** — both the front
IP `85.192.63.249` and the host IP `138.124.61.86` fail TCP from RF vantage
points while remaining reachable from the rest of the world (check-host.net
ru-nodes TCP timeout, non-RU nodes fine). The world geo-branch (Bunny SCR →
direct Railway/Tigris CNAMEs) was unaffected. `pass.uwuwu.net` (password
manager) was deliberately left world-only per owner decision, which also
removed the RU-jurisdiction TLS concern from ADR-049 for that host.

A new RF VM was available: Aeza-RU Saint Petersburg `193.233.217.243`
(AS216246 Aeza Group LLC, SPBs-1 shared, Ubuntu 26.04, 1 vCPU / 2 GB).
Live verification before the cut (2026-09-30, from the VM):

- Railway edge `69.46.46.4` / `.31`: TCP + TLS (any SNI) + HTTP 200 in
  ~0.3 s — the ADR-049 payload-drop of `69.46.46.0/24` does **not** apply to
  this hoster's transit (daytime).
- Tigris origin `origa.t3.tigrisfiles.io`: 28–39 MB/s (118 MB model in 4 s).
- The VM already ran xray VLESS+Reality on `0.0.0.0:443`
  (masking `www.samsung.com:443`) — a personal VPN that had to keep working.

## Decision

RF clients (Bunny SCR geo-branch) resolve `origa` / `app.origa` /
`s3.origa.uwuwu.net` to `193.233.217.243` (A record, TTL 60). The VM fronts
them with nginx; the world branch keeps its direct CNAMEs and is untouched.

```
RF user ──> Bunny SCR (geo=RU) ──> A 193.233.217.243 ──> nginx :443
            (stream, ssl_preread, PROXY protocol on every branch)
              ├─ SNI origa/app/s3 ──> http :8443 (TLS termination,
              │    real client IP via PROXY protocol + realip,
              │    upstream TLS verify ON) ──> Railway (origa/app)
              │                                 Tigris (s3)
              └─ SNI www.samsung.com / no-SNI / anything else
                   ──> bridge :4432 (strips PROXY header)
                   ──> xray Reality :4430 (loopback) — VPN unchanged
World ──> Bunny SCR (geo≠RU) ──> direct CNAMEs: vl080mt6 / 9v15a3ov
        (up.railway.app) / d3gbi3wo8j4c2w.cloudfront.net — untouched
```

Key implementation facts (all verified in production):

1. **nginx stream SNI-demultiplex** shares :443 between the proxy and the
   pre-existing xray VPN. `proxy_protocol on` is set on the public stream
   server so the http blocks recover the real client IP
   (`listen 127.0.0.1:8443 ssl proxy_protocol` + `set_real_ip_from 127.0.0.1`
   + `real_ip_header proxy_protocol` → `$remote_addr` is the client, and
   `proxy_set_header X-Forwarded-For $remote_addr` keeps the ADR-049 contract
   "client-supplied XFF never reaches the apps"). xray does not speak PROXY
   protocol, so its branch detours through a loopback bridge (:4432) that
   strips the header; VPN clients (SNI `www.samsung.com`) notice nothing.
2. **Upstream TLS verification is ON** (`proxy_ssl_verify on` against the
   system CA bundle, `proxy_ssl_server_name on` + `proxy_ssl_name <site>`) —
   the VM→edge leg crosses TSPU and carries auth traffic; silent MITM must
   not be possible. Caddy on Frankfurt verified upstream certs too.
3. **Certs**: acme.sh DNS-01 (`dns_bunny` hook), one SAN cert for the three
   names, `--reloadcmd "systemctl reload nginx"`, renewal crontab. Fresh keys
   were issued on the RF VM — nothing was copied from the blocked box.
4. **Static upstream resolution** (`upstream { server name:443; keepalive 16 }`
   without `resolve`): nginx resolves hostnames at start/reload only.
   Railway edge IPs are stable (ADR-049/059); on an edge change the runbook is
   `dig` first, then `nginx -s reload` (reload fails closed with "host not
   found in upstream" when DNS is down — no half-applied states).
5. **Bunny SCR cutover** (commit `c479c8ed`): RF answer switched
   `85.192.63.249 → 193.233.217.243`, plus a hostname guard (the script
   answers only the four attached names; unknown names get `undefined`
   instead of silently falling through to the RF IP), dead-branch dedupe and
   header/parked-comment fixes. Deploy:
   `bunny dns scripts deploy infra/bunny-dns/handleQuery.js 91629`.
   Pre-flight verified: script attached (NS records) exactly to `origa`,
   `app.origa`, `s3.origa`, `content.origa`; `pass` is a plain A record and
   outside the blast radius; deployed script behavior was identical to the
   repo version (no drift).
6. **Verification at cutover** (evening window, 19–23 MSK): Yandex DNS
   (RU resolver) returns `193.233.217.243` for all three names; full RF chain
   `app` 200 in 0.31 s, Tigris 24.5 MB/s; world branch pinned CNAMEs unchanged
   (vl080mt6 / 9v15a3ov / d3gbi3wo8j4c2w), non-RU check-host nodes 200 via
   Railway/CloudFront; a check-host node that resolved the RF branch got 200
   from the nginx too. Hourly probe continues (`/root/rf-branch-tests/
   probe.log`, cron) covering RF full chain + world CF.
7. **IaC**: `~/rf-proxy-infra/` on the VM (git), offsite mirror
   `~/rf-proxy-infra` on the operator laptop; `xray-config.json` snapshot
   included. Old Aeza box keeps Caddy for `pass.uwuwu.net`, `uwuwu.ru`
   redirects, searxng and the xray VPN inbound — its Caddyfile RF-site blocks
   are now dead references (documented as legacy in the README).

## Compromises (accepted by the owner)

- **Auth transit**: `app.origa` RF-branch TLS terminates on the RF VM —
  logins/session tokens of RF users transit RF jurisdiction. Accepted
  (short-lived tokens; the alternative was no RF availability at all).
- **Bunny API key on the RF VM** (renewals): with Bunny's non-scoped keys
  this is full DNS control of `uwuwu.net` including `pass`. Accepted; rotate
  if Bunny ships per-zone keys.
- **Lost DE→DE leg** (ADR-049): the VM→upstream egress now crosses TSPU, and
  `69.46.46.0/24` is a documented TSPU target. The assumption "datacenter
  egress of AS216246 is not filtered like residential" is validated by the
  live probes and the evening window; it may break on a future TSPU rollout
  wave.
- **SPOF**: one small VM is the whole RF branch (world unaffected). Plan B —
  RF→EU chain (RF VM passthrough + new EU VM) — is ~30 minutes of work if
  the direct route dies.
- **A-only RF branch**: never add AAAA (broken v6 on RF ISPs, ADR-058).
- **No upstream connect retries** (nginx, single upstream per block):
  transient Railway edge 400/502s are covered by the client-side retry
  (PR #626), not by the proxy.

## Alternatives considered

- **Second IPv4 on the RF VM instead of the stream demux** (the ADR-049
  "second IP, not a second server" pattern): rejected — extra monthly cost,
  unknown availability of extra IPv4s at the RF hoster, and a fresh IP could
  land in a different TSPU segment than the verified `193.233.217.243`. The
  stream demux adds zero cost and no new public ports.
- **Keep CloudFront for RF** — rejected: ADR-059 measured evening TSPU
  throttling of the CF route down to hundreds of B/s.
- **RF→EU chain immediately** — deferred as plan B: not needed while the
  direct route is clean, and every extra hop is another TSPU surface.

## Consequences

- RF users are served again (~0.3 s API latency, tens of MB/s CDN from RF).
- The RF branch is now behind RF jurisdiction and RF hosting weather; the
  world branch is fully independent of the RF VM.
- SSH:22 from outside RF degrades during evening TSPU windows (banner
  exchange timeouts) — production ports are unaffected; use the Aeza panel
  console if SSH is unavailable.
- The blocked Aeza Frankfurt box loses its delivery role but keeps
  `pass`, `uwuwu.ru`, searxng and the xray VPN inbound (world-reachable).
