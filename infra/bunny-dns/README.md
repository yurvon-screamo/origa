# Bunny Scriptable DNS: geo-steering for RF (script ID 91629)

`handleQuery.js` is deployed as Bunny DNS script `origa-s3-geo` (ID 91629) and
attached (via NS records, TTL 60) to **four** names in the `uwuwu.net` zone
(873317):

- `origa`         → RF: A 193.233.217.243 (RF VM, nginx), world: CNAME vl080mt6.up.railway.app
- `app.origa`     → RF: A 193.233.217.243, world: CNAME 9v15a3ov.up.railway.app
- `s3.origa`      → RF: A 193.233.217.243, world: CNAME d3gbi3wo8j4c2w.cloudfront.net
- `content.origa` → CNAME origa.t3.tigrisbucket.io for everyone (diagnostic name, no RF branch)

`pass` is a plain A record (85.192.63.249, TTL 300) and is **not** attached to
the script. The script answers only the four names above (hostname guard) and
returns `undefined` for anything else — a newly attached name without a branch
fails loudly instead of silently resolving to the RF IP. (Semantics note:
Bunny treats a script answer of `undefined` as "no record" — grey answer.
The terminal `return undefined` is unreachable today: every guarded name has
a branch; it exists so a future branch-less addition cannot inherit another
name's record.)

There is **no health gating** in the script: RF always gets the RF VM IP,
world always gets the direct CNAME. Fallback decisions are made by editing and
redeploying the script.

## Why

During TSPU/provider throttling windows (observed 2026-09-19 evening)
CloudFront body transfers from RF degrade to hundreds of B/s, and the Railway
edge subnet 69.46.46.0/24 is payload-dropped by TSPU outright (ADR-049). The
geo-split routes RF users through an RF-terminating front (ADR-061); world
users keep the direct CloudFront/Railway paths.

## RF VM side (nginx on 193.233.217.243, ADR-061)

nginx stream SNI-demultiplexer on :443 routes RF TLS to either the http proxy
blocks (loopback :8443 → Railway/Tigris, PROXY protocol carries the real
client IP) or the xray Reality VPN (loopback :4430 via a PROXY-header-stripper
bridge on :4432). TLS certificates are issued by acme.sh **DNS-01 via Bunny**
on the VM (HTTP-01 cannot pass: world resolvers land on CloudFront/Railway
directly); renewal via the acme.sh crontab on the VM. Upstream hostnames are
resolved once at start/reload — after any Railway edge change run `dig` first,
then `nginx -s reload` (reload fails closed with "host not found in upstream"
if DNS is unreachable).

Legacy reference (superseded 2026-09-30, host TSPU-blocked from RF): the old
Aeza Frankfurt VPS (85.192.63.249) served the same RF branch via Caddy with
`bind 85.192.63.249`, proxying to `origa.t3.tigrisfiles.io` /
`vl080mt6.up.railway.app`. Caddy on that box remains authoritative for
`pass.uwuwu.net`, `uwuwu.ru` redirects and personal services.

## Change procedure

```bash
# edit handleQuery.js, then:
bunny dns scripts deploy infra/bunny-dns/handleQuery.js 91629
```

Rollback: `git revert` the cutover commit, redeploy — RF clients fall back to
the previous answer within TTL 60.
