# ADR-057: Move CDN s3-proxy from Railway onto the uwuwu.net VPS (Aeza Frankfurt)

## Status

Accepted (cutover 2026-09-17; Railway decommission pending 3–7 day observation window)

## Date

2026-09-17

## Context

After the ADR-049 mitigation the CDN chain for `s3.origa.uwuwu.net` was:

```
Client → Caddy @ Aeza VPS (85.192.63.249) → Railway edge → s3-proxy (pottava/s3-proxy) → Tigris bucket adaptable-foodbox-ucep7wx
```

Facts that motivated the change:

- **Railway service egress is paid** and the s3-proxy was the only meaningful egress
  source (~10.5–13.5 GB per period, measured in ADR-037). Bucket egress (Tigris) and
  Aeza VPS traffic are free. The absolute cost is small (cents to low units of $/mo) —
  the motivation is **decomposition of the Railway dependency, not savings**.
- **Railway edge is fragile for this project**: TSPU blocked the whole edge /24
  (ADR-049), and a Railway custom-domain binding can silently lapse (documented caveat
  in the ADR-037 runbook rollback section).
- There is no real CDN in the chain today: since 2026-09-13 all traffic already flows
  through the single Aeza VPS; Tigris itself is anycast-backed behind the proxy; the
  client-side cache does the heavy lifting via the tiered Cache-Control policy.

### Options considered

| Option | Verdict |
| --- | --- |
| Keep as is | Paid hop + fragile Railway dependency remain |
| Cloudflare proxy in front | Rejected — TSPU throttles CF-fronted routes from RF (ADR-037/046 precedent) |
| Direct public Tigris (no proxy) | Attractive long-term (Tigris anycast, zero egress), but RF DPI behaviour of `storageapi.dev` unverified; the previous direct-Tigris attempt (ADR-037) failed on Cloudflare routing. Deferred as a follow-up ADR |
| **Run the same s3-proxy container on the VPS** | Chosen — zero URL/behaviour change, instant rollback, removes the paid hop |

## Decision

Run the identical `pottava/s3-proxy` container on the uwuwu.net VPS and point the
Caddy site at it:

```
Client → Caddy @ Aeza VPS → s3-proxy (docker, 127.0.0.1:8081) → Tigris (t3.storageapi.dev)
```

Explicitly **not** a revival of the rolled-back ADR-037: the bucket remains the
Railway-managed private Tigris `adaptable-foodbox-ucep7wx`; only the place where the
proxy runs changes. The URL is untouched → zero application changes (URL is baked in
via `build.rs`; e2e downloads from prod via `end2end/cdn-manifest.txt` keep working).

Key implementation facts (all verified in production 2026-09-17):

1. **Env contract snapshot** (with the secret redacted):
   `~/uwuwu-proxy-infra/docs/backups/railway-s3-proxy-envs.2026-09-17.txt`. The Railway
   access key id is identical to `~/.aws/credentials [origa]` (`tid_qYczVK...`, secret
   length verified) — credentials parity is exact. Secrets live in `/root/s3-proxy.env`
   on the VPS (chmod 600, outside git, transferred via ssh stdin — no literals in shell
   history).
2. **Image pinned by digest**: `pottava/s3-proxy@sha256:7e2b4c46dcae...f588`
   (amd64; upstream frozen since 2020-02 — same `latest` tag Railway runs).
3. **The container listens on :80** (not 8080 as first assumed); compose maps
   `127.0.0.1:8081→80`. Port 8081 chosen because searxng occupies 8080 on the host.
4. **No container healthcheck possible**: the image is built `FROM scratch` (no
   sh/wget/curl inside — verified). Failure coverage instead: `restart: unless-stopped`
   (crashed process → restart), Caddy `lb_try_duration 5s` (transient failures retried,
   no 502 to the client), external monitoring of the public URL (runbook README).
5. **`AWS_REGION=auto`** on the VPS (Railway's `$sin` reference variable does not exist
   outside Railway; Tigris accepts any SigV4 region — this is the t3 CLI convention).
6. **CORS contract identical to the previous chain** (baseline-diffed): origin `*`,
   methods `GET, HEAD, OPTIONS`, allow-headers
   `Range, If-None-Match, If-Match, Content-Length, Content-Type, ETag, Last-Modified,
   Cache-Control`, `access-control-max-age: 600`, `Access-Control-Expose-Headers`
   absent (parity — the client worked without it before). Pre-flight and simple-GET
   behaviour unchanged; Range → 206 unchanged.
7. **Baseline header snapshot** before cutover:
   `~/uwuwu-proxy-infra/docs/baseline-s3-headers-20260917.txt`. Post-cutover diff:
   only Railway edge markers disappeared (`server: railway-hikari`, `x-railway-*`,
   `x-hikari-trace`, `x-cache`); `via` changed `2.0 Caddy` → `1.1 Caddy` (HTTP/1.1 to
   the loopback upstream); `alt-svc` is now advertised by Caddy's own HTTP/3.
   Notable baseline fact: **conditional GET returns 200, not 304** — pottava does not
   honour `If-None-Match` through this chain. This is parity, not a regression
   (ADR-037's observed 304 was on direct user-Tigris, a rolled-back setup).
8. **Cutover verification**: SHA256 of manifest-listed assets
   (`dictionaries/sudachidict-20260723/char_def.bin`, `grammar/grammar_v2.json`,
   `phrases/data_bundle_5.json`) match the manifest through the new chain;
   `whisper/onnx/decoder_model.onnx` (not manifest-covered) matches a direct boto3
   download from Tigris. check-host.net: 24/25 nodes 200 (single India timeout,
   non-RU), RU node `ru3` 200 in 0.8 s.
9. **Caddy change**: the `s3.origa.uwuwu.net` site block no longer uses the
   `uwuwu_site` snippet — it is a direct `reverse_proxy 127.0.0.1:8081` with
   `lb_try_duration 5s` / `lb_try_interval 250ms`. No `header_up Host` /
   `tls_server_name` needed: the upstream is plain HTTP on loopback and pottava routes
   by its `AWS_S3_BUCKET` env, not by the Host header. `header_up X-Forwarded-For`
   is unnecessary too — Caddy's default already passes the client IP (caddy validate
   confirms with a warning).

### Risk acceptance — write-scoped credentials on the VPS

s3-proxy needs read-only access, but a Railway-managed Tigris bucket does not offer
scoped/read-only keys through any available CLI (`t3 access-keys assign --role`
manages the user-owned Tigris account only). The VPS therefore holds **write-scoped**
bucket credentials. Accepted consciously: the bucket content is public static data
integrity-verified by the client via manifest SHA256; mitigations are loopback-only
bind, chmod 600 env file, and the fact that the VPS is already in the trust boundary
(`pass.uwuwu.net` TLS terminates there per ADR-049).

## Consequences

**Positive:**

- Railway service egress for CDN drops to ~0; the paid hop and one latency leg are
  gone (VPS → Tigris instead of VPS → Railway → Tigris).
- The CDN no longer depends on the Railway edge subnet (TSPU-blocked before) or on
  Railway custom-domain binding lifetime.
- Until Railway decommission: instant rollback = revert the cutover commit in
  `uwuwu-proxy-infra` + standard deploy → traffic returns via the still-alive Railway
  s3-proxy.

**Negative / accepted risks:**

- The VPS is now the SPOF for the CDN as well — it already was the SPOF for all four
  hosts (ADR-049); structurally nothing new, but the blast radius of a VPS death grows.
- **RTO after the Railway decommission**: a full VPS loss means hours (re-create
  s3-proxy on a new host from the compose + env contract in this repo / re-create on
  Railway from the snapshot) instead of the current minutes. Accepted.
- One more moving part on the VPS (docker service to keep updated; the image is
  frozen upstream, so updates are rare by design).

## Rollback

- **Before Railway decommission**: revert the cutover commit(s) in
  `~/uwuwu-proxy-infra` and run the standard validate-then-replace deploy; the Railway
  s3-proxy stays alive during the whole observation window, so the previous chain is
  restored within a reload. `docker compose down` on the VPS is optional cleanup.
- **After Railway decommission**: the VPS container is primary; fallback is
  re-creating s3-proxy elsewhere from `s3-proxy/docker-compose.yml` +
  `docs/backups/railway-s3-proxy-envs.*.txt` (RTO hours) — see the RTO note above.

## Follow-ups

1. **Railway decommission** after 3–7 stable days (gates: s3-proxy egress ~0 in
   Railway metrics, no CDN-fetch-failure spike in Sentry): `railway service delete`
   (service `a3f10cf6-2d4f-42cf-a176-8ebb4253d734`), then remove the
   `_railway-verify.s3.origa` TXT at Aeza (fresh zone dump first, record ID verified
   against the live API, not the old dump). Then finalize egress numbers in this ADR.
2. Optional hardening: Caddy cache-handler for edge caching of immutable assets
   (reduces Tigris→VPS fetches; traffic is free, so this is latency polish only).
3. Open strategic option: direct public Tigris without any proxy (true anycast edge
   for the world) — requires its own ADR with RF DPI testing of `storageapi.dev`
   (the ADR-037 failure was Cloudflare-routed user-Tigris, not the Railway bucket
   endpoint; conclusions do not transfer automatically).
4. The removed-in-2026-09 monitoring (`uwuwu-check.sh`) is not restored here; external
   checks are manual per the README runbook.
