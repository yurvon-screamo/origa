# Aeza VPS (85.192.63.249) — Origa delivery (ADR-059)

The VPS Caddyfile is the source of truth (edited on the box; see the warning
in its header about the second IP owned by xray). Origa blocks:

| Site | Upstream | Notes |
|---|---|---|
| `s3.origa.uwuwu.net` | `https://origa.t3.tigrisfiles.io` | `header_up Host` → bucket domain; public bucket, no signer |
| `app.origa.uwuwu.net` | `https://9v15a3ov.up.railway.app` | `header_up Host` + `tls_server_name` (Railway routes by Host) |
| `origa.uwuwu.net` | `https://vl080mt6.up.railway.app` | same |
| `uwuwu.ru` / `www` / `origa.uwuwu.ru` | 301 | `redir https://origa.uwuwu.net{uri} permanent` |

- every block: `bind 85.192.63.249` (xray owns 138.124.61.86:443)
- TLS: acme.sh DNS-01 via `dns_bunny` (BUNNY_API_KEY on the box), certs in
  `/etc/ssl/origa/`, renewal by the acme.sh crontab
- restart flow: `caddy validate` → `systemctl restart` (reload hangs on
  eternal grace with live connections; restart is safe once every block binds)

Bunny zone uwuwu.net (873317): plain A-records → 85.192.63.249. The SCR
geo-split script lives in `../bunny-dns/` — blocked on the platform A-query
bug (ticket filed 2026-09-19, `_probe`/`probe2` left as reproducers).
