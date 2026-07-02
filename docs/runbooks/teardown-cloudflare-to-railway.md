# Teardown Cloudflare → Railway-only (Aeza NS)

**Status:** COMPLETED
**Date:** 2026-06-28
**Executed by:** Autonomous agent (browser automation via active sessions in Cloudflare dashboard and Aeza panel)

## What was done

1. Backed up Cloudflare zone (8 records, see `docs/backups/uwuwu.net.cloudflare.2026-06-28.zone`).
2. Verified Aeza DNS zone already populated with all required records (from ADR-007 pre-Cloudflare config).
3. Switched "Use custom NS servers" toggle OFF in Aeza panel (`my.aeza.net/services/1095525/ns`).
4. Authoritative NS now: `ns1/ns2/ns3/ns4.aeza-dns.net` (confirmed via .net TLD root).

## Current DNS configuration (Aeza authoritative)

| Name | Type | Value | TTL |
| ------ | ------ | ------- | ----- |
| `origa` | A | `69.46.46.46` | 300 |
| `app.origa` | CNAME | `9fmm6y4e.up.railway.app` | 300 |
| `s3.origa` | CNAME | `sltxm1ip.up.railway.app` | 300 |
| `pass` | CNAME | `vcce37wa.up.railway.app` | 300 |
| `_railway-verify.origa` | TXT | `railway-verify=8434...` | 300 |
| `_railway-verify.app.origa` | TXT | `railway-verify=2e3a...` | 300 |
| `_railway-verify.s3.origa` | TXT | `railway-verify=6fd8...` | 300 |
| `_railway-verify.pass` | TXT | `railway-verify=0568...` | 300 |

## What changes after propagation

- `origa.uwuwu.net` resolves to `69.46.46.46` directly (no Cloudflare IPs).
- AAAA for `origa.uwuwu.net` → clean NOERROR (empty, no SERVFAIL).
- HTTP responses may still carry `server: cloudflare` and `X-Railway-Edge: osl1` because Railway's own edge infrastructure uses Cloudflare CDN internally — this is Railway-side and not under our control. The important signal is `X-Railway-Edge` (direct origin), not `server`.
- `app`/`s3`/`pass` remain CNAME to Railway (unchanged behaviour; same AAAA SERVFAIL on Google resolver as before — pre-existing Railway DNS bug, not a regression).

## Post-migration verification commands

```bash
# NS switched (via public resolver)
curl -s "https://dns.google/resolve?name=uwuwu.net&type=NS"
# Expected: ns1-4.aeza-dns.net

# origa resolves to Railway edge, not Cloudflare IPs
curl -s "https://dns.google/resolve?name=origa.uwuwu.net&type=A"
# Expected: Answer with data "69.46.46.46" (not 104.21.x.x / 172.67.x.x)

# AAAA should be empty NOERROR (not SERVFAIL)
curl -s "https://dns.google/resolve?name=origa.uwuwu.net&type=AAAA"
# Expected: Status 0, no Answer section

# Authoritative check (bypass cache) — query Aeza NS directly
nslookup -type=A origa.uwuwu.net ns1.aeza-dns.net
nslookup -type=AAAA origa.uwuwu.net ns1.aeza-dns.net
nslookup -type=NS uwuwu.net a.gtld-servers.net

# Site still serving
curl -sSI https://origa.uwuwu.net/
# Expected: 200 OK (server may say cloudflare — see note above about Railway edge)

# Bot SSR
curl -sSI -A "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)" https://origa.uwuwu.net/
curl -sSI -A "Mozilla/5.0 (compatible; YandexBot/3.0; +http://yandex.com/bots)" https://origa.uwuwu.net/
```

## Known regressions (accepted)

1. **No Cloudflare DDoS protection** — site is directly exposed.
2. **No Cloudflare caching** — all requests hit Railway origin.
3. **No Cloudflare IPv6 proxy** — no AAAA record (acceptable per ADR-007).
4. **app/s3/pass SERVFAIL on AAAA** — Railway DNS bug still present for CNAME subdomains (NOT new, existed before Cloudflare).
5. **Single IP point of failure** — if Railway changes `69.46.46.46`, must update Aeza A record.

## Rollback

If anything breaks, re-enable Cloudflare:

1. Go to `https://my.aeza.net/services/1095525/ns`.
2. Toggle "Use custom NS servers" ON.
3. Set NS1 = `ali.ns.cloudflare.com`, NS2 = `clay.ns.cloudflare.com`.
4. Save.

This reverts to the pre-migration state within NS TTL (up to 48h).

## Cloudflare zone cleanup (optional)

The `uwuwu.net` zone still exists in the Cloudflare dashboard. Since we're not deleting it, it serves as a backup reference. If you want to fully remove Cloudflare:

1. Log in to `https://dash.cloudflare.com`.
2. Go to `uwuwu.net` → Overview → scroll to bottom.
3. Click "Remove site from Cloudflare".
4. Confirm.

**WARNING:** This is irreversible. Only do this after confirming Aeza NS is fully propagated and the site works.

## SEO re-crawl submissions (2026-06-28)

After the DNS switch, re-crawl / URL submission was performed across all three search consoles (browser automation, sessions active). Submissions:

- **Yandex Webmaster** — 12/12 URLs queued (root + 4 EN pages + 4 RU pages + KO/VI roots). "Переобход страниц". Limit: 150/day.
- **Google Search Console** — 11/12 URLs requested (1 rejected: `/ru` live-test gave transient "Server connection error"; needs retry after DNS propagation). Limit: ~10/day.
- **Bing Webmaster Tools** — 12/12 URLs submitted via "Submit URLs" + Fetch as Bingbot for root. Limit: 100/day.

**Note:** Browser screenshots of these submissions were captured during execution but were lost from the working tree in a later session. The submission counts above are reconstructed from session logs and are accurate, but no visual evidence is committed.
