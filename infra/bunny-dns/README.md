# Bunny Scriptable DNS: s3 geo-steering + RF failover (script ID 91629)

`handleQuery.js` is deployed as Bunny DNS script `origa-s3-geo` (ID 91629) and
attached (SCRIPT records, TTL 60) to **two** names in the `uwuwu.net` zone
(873317):

- `s3.origa`      → RF: A 193.233.217.243 while the RF VM is online (nginx
  SNI-demux → Tigris); world — and RF during VM death — CNAME
  `d3gbi3wo8j4c2w.cloudfront.net` (CloudFront in front of the public Tigris
  bucket)
- `content.origa` → CNAME `origa.t3.tigrisbucket.io` for everyone
  (diagnostic name, no RF branch)

Everything else in the zone is plain records and stays out of the script:
`origa`, `app.origa`, `umami.origa` are plain CNAMEs to the Railway edge
since ADR-072 (2026-11-06) — script-attached names break LE-issuance paths
that resolve through Google resolvers, so never re-attach Railway-LE names;
`pass` is a plain A record (85.192.63.249) and was never in the blast
radius. The script answers only the two attached names (hostname guard) and
returns `undefined` for anything else — a newly attached name without a
branch fails loudly (grey answer) instead of silently resolving elsewhere.

## RF failover gate (`Monitoring.getStatus`)

On the RF geo branch the script consults Bunny's built-in background
monitoring: `Monitoring.getStatus("193.233.217.243").isOnline`.

- VM online → RF resolvers get the A record, TTL 60.
- VM offline → RF resolvers get the same CloudFront CNAME the world gets,
  TTL 300: degraded but working, instead of silently dead.
- Any non-`true` status (offline, unknown shape, `Monitoring` global absent)
  is treated as offline — **fail-closed**: degrade to the working world path,
  never grey-answer.

Monitoring semantics (Bunny docs): ICMP ping only (700 ms × 3 retries, one
reply = online); actively queried IPs are re-checked roughly every 10 s; the
status is per DNS server — "reachable from the PoP answering the query".
The first query for an IP synchronously waits for the initial check.

**What the gate does NOT cover:** an nginx crash on a live VM keeps
answering ping, so no failover happens. That failure class belongs to the
uptime poller's pinned HTTP probe (alert to the owner, monitoring task
20261104060619), not to DNS. A VM-side watchdog that blinds ping when nginx
dies is a possible future task, deliberately out of scope here.

## Convergence (asymmetric by design)

- **Failover (VM dies):** the RF A answer carries TTL 60 and Bunny re-checks
  ~every 10 s, so RF clients land on CloudFront within roughly a minute of
  the actual death.
- **Recovery (VM returns):** clients that already received the failover
  CNAME hold it up to TTL 300, so the return to the A record converges up
  to ~5 minutes after Bunny sees the VM alive again. The slower recovery is
  the cost of the stable CNAME answer — not a bug. Measured numbers from
  the drills live in the Bunny-failover task body and the wiki Deployment
  doc.

## Drill (VM-death rehearsal)

The gate triggers on ICMP loss, so the drill is an ICMP black hole on the
RF VM — not an nginx stop:

```bash
# on the RF VM — drop ping, auto-revert after 4 minutes
iptables -I INPUT -p icmp --icmp-type echo-request -j DROP
nohup sh -c 'sleep 240 && iptables -D INPUT -p icmp --icmp-type echo-request -j DROP' \
  >/dev/null 2>&1 &
```

Then watch, from anywhere:

```bash
watch -n 15 'dig +short s3.origa.uwuwu.net @77.88.8.8; echo ---; dig +short s3.origa.uwuwu.net @1.1.1.1'
```

Expected: the RU resolver flips to the CloudFront CNAME within the failover
window, and back to the A record after the revert; the world answer never
changes. Dropping ping does not touch nginx or the xray VPN (TCP).

Coordination: if the uptime poller (monitoring task 20261104060619) is
already live, its DNS assert "s3 RU → strictly A VM" fires during the drill
— warn the owner about the expected Telegram alert before opening the
window.

## RF VM side (nginx on 193.233.217.243, ADR-061)

nginx stream SNI-demultiplexer on :443 routes RF TLS to either the http
proxy blocks (loopback :8443 → Tigris, PROXY protocol carries the real
client IP) or the xray Reality VPN (loopback :4430 via a
PROXY-header-stripper bridge on :4432). TLS certificates are issued by
acme.sh **DNS-01 via Bunny** on the VM (HTTP-01 cannot pass: world resolvers
land on CloudFront directly); renewal via the acme.sh crontab on the VM.
Upstream hostnames are resolved once at start/reload — after any edge
change run `dig` first, then `nginx -s reload` (reload fails closed with
"host not found in upstream" if DNS is unreachable).

`umami.origa` keeps its own certificate and nginx snippet on the VM even
though the name is now a plain CNAME (ADR-072) — the config is dormant, not
removed, so a future geo-split return for a specific name is one Bunny
operation (nginx configs alive, ADR-072).

Legacy reference (superseded 2026-09-30, host TSPU-blocked from RF): the
old Aeza Frankfurt VPS (85.192.63.249) served the same RF branch via Caddy.
Caddy on that box remains authoritative for `pass.uwuwu.net`, `uwuwu.ru`
redirects and personal services.

## Change procedure

```bash
# edit handleQuery.js, then:
node infra/bunny-dns/smoke.test.mjs                 # branch matrix must stay green
node --check infra/bunny-dns/handleQuery.js         # syntax gate
bunny dns scripts deploy infra/bunny-dns/handleQuery.js 91629   # upload + publish
```

Deploy is a production change: record the live dig matrix
(RU `@77.88.8.8` / world `@1.1.1.1` × `s3`/`content`) before and after — it
must be identical while the VM is alive.

Rollback: `git revert` the change commit, redeploy — RF clients fall back
to the previous answer within TTL 60. The same rollback treats a misbehaving
gate: the previous script revision has no monitoring dependency. The smoke
test runs outside CI on purpose (`infra/**` is not in ci.yml path filters);
`node infra/bunny-dns/smoke.test.mjs` is part of this procedure for the same
reason.
