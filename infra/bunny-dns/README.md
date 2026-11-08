# Bunny Scriptable DNS: geo-steering for RF (script ID 91629)

`handleQuery.js` is deployed as Bunny DNS script `origa-s3-geo` (ID 91629) and
attached (via Script records, TTL 60) to **two** names in the `uwuwu.net` zone
(873317):

- `s3.origa`      → RF: A 193.233.217.243 (RF VM, nginx), world: CNAME d3gbi3wo8j4c2w.cloudfront.net
- `content.origa` → CNAME origa.t3.tigrisbucket.io for everyone (diagnostic name, no RF branch)

Three more names are plain CNAME records in the zone — straight to Railway for
everyone, RF included (variant A, ADR-072 / task 202611060809: RF clients of
these three names take the TSPU risk on the direct path):

- `origa`       → c2qj368z.up.railway.app
- `app.origa`   → 9fmm6y4e.up.railway.app
- `umami.origa` → 07wyrdxw.up.railway.app (self-hosted Umami, ADR-054 §7)

The script does **not** answer those three while they are not attached.
`handleQuery.js` keeps a **dormant pre-staged geo-split branch** for
`umami.origa` (RF → the RF VM via the common `isRF` block, world → the Railway
edge CNAME) so that the post-fix return stays a single record operation.

**Do not re-attach cert-bearing names while the Bunny NODATA bug stands:**
Scriptable DNS answers NODATA through Google/8.8.8.8 resolvers (verified
2026-11-08), and LE validators / Railway resolvers sit on Google DNS —
script-attached names fail domain validation, breaking cert issuance and
renewal. This is the diagnosed root cause of the "internal error" issuance
outage (task 202611060809; plain-CNAME cutover 2026-11-06 restored issuance:
app/landing certs re-issued VALID on 2026-11-06, umami on 2026-11-08 after a
manual `retry-domain-certificate` nudge of a cycle stuck since 2026-11-06).

`pass` is a plain A record (85.192.63.249, TTL 300) and is **not** attached to
the script. The script answers only the guarded names (hostname guard) and
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

`umami.origa` has its own certificate (SAN `umami.origa.uwuwu.net`, acme.sh
DNS-01; renewed 2026-11-08, valid into February 2027) and its own
`rf-proxy-common-umami.conf` snippet — nginx does not support variables in
`ssl_certificate`, so the shared snippet (which pins the `origa.uwuwu.net`
cert) cannot be reused for a second host. DNS-01 TXT records created by
acme.sh resolve regardless of how the name itself is routed (verified
2026-10-22 when the name was script-attached, and again 2026-11-08 when it
was already a plain zone CNAME). Since 2026-11-06 the name itself is a plain
CNAME straight to Railway (variant A), so this nginx branch is dormant
fallback: it serves traffic only if the zone record is pointed back at the
VM.

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
