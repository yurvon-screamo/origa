# Bunny Scriptable DNS: geo-steering for RF (script ID 91629)

`handleQuery.js` is deployed as Bunny DNS script `origa-s3-geo` (ID 91629) and
attached to two records in the `uwuwu.net` zone (873317):

- `s3.origa` → RF: A 85.192.63.249 (Aeza VPS), world: CNAME dsl8eedfp23ee.cloudfront.net
- `origa`    → RF: A 85.192.63.249, world: CNAME d3o6p7y31je36k.cloudfront.net

Health-gated via `Monitoring.getStatus()`: if the VPS goes down, RF falls back
to CloudFront automatically (TTL 60s).

## Why

During TSPU/provider throttling windows (observed 2026-09-19 evening) CloudFront
body transfers from RF degrade to hundreds of B/s while the RF→Frankfurt(Aeza)
route keeps multi-MB/s throughput. The geo-split routes RF users around the
throttled CF path; world users keep the CF edge cache.

## VPS side (Caddy on 85.192.63.249)

Both hostnames have Caddy site blocks (bind 85.192.63.249) proxying to
`origa.t3.tigrisfiles.io` / `vl080mt6.up.railway.app`. TLS certificates are
issued by acme.sh **DNS-01 via Bunny** on the VPS (HTTP-01 cannot pass: world
resolvers land on CloudFront); renewal via the acme.sh crontab on the VPS.
Client SNI is always the original hostname regardless of which DNS branch
answered — that is why the VPS must hold certs for the exact names.

## Change procedure

```bash
# edit handleQuery.js, then:
bunny dns scripts deploy infra/bunny-dns/handleQuery.js 91629
```
