# Aeza VPS (85.192.63.249) — legacy delivery front + personal services

> **Status since 2026-09-30 (ADR-061):** this box is TSPU-blocked from RF
> entirely (both IPs TCP-dead from RF vantage points) and is no longer the
> Origa RF front. The RF branch moved to an RF VM — `infra/bunny-dns/` is the
> source of truth for the current topology. Origa `s3/app/origa` blocks below
> are **dead references kept for rollback archaeology**; the box still serves
> `pass.uwuwu.net`, `uwuwu.ru` redirects and the xray VPN inbound for
> world-connected clients (VPN from RF is dead with the host).

The VPS Caddyfile is edited on the box (see the warning in its header about
the second IP owned by xray). Origa blocks (LEGACY, RF-dead since ADR-061):

| Site | Upstream | Notes |
| --- | --- | --- |
| `s3.origa.uwuwu.net` | `https://origa.t3.tigrisfiles.io` | `header_up Host` → bucket domain; public bucket, no signer |
| `app.origa.uwuwu.net` | `https://9v15a3ov.up.railway.app` | `header_up Host` + `tls_server_name` (Railway routes by Host) |
| `origa.uwuwu.net` | `https://vl080mt6.up.railway.app` | same |
| `uwuwu.ru` / `www` / `origa.uwuwu.ru` | 301 | `redir https://origa.uwuwu.net{uri} permanent` |

- every block: `bind 85.192.63.249` (xray owns 138.124.61.86:443)
- TLS: acme.sh DNS-01 via `dns_bunny` (BUNNY_API_KEY on the box), certs in
  `/etc/ssl/origa/`, renewal by the acme.sh crontab
- restart flow: `caddy validate` → `systemctl restart` (reload hangs on
  eternal grace with live connections; restart is safe once every block binds)

Bunny zone uwuwu.net (873317): the SCR geo-split script in `../bunny-dns/` is
**live and authoritative** for `origa`/`app.origa`/`s3.origa`/
`content.origa` (NS-attached). RF → A 193.233.217.243 (RF VM), world → direct
CNAMEs (Railway / CloudFront). The "platform A-query bug" note that used to
live here was a misdiagnosis — refuted by Bunny support and ADR-059 §5; the
root cause was attaching records to the wrong in-zone names.
