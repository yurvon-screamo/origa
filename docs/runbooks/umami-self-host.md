# Runbook: Umami self-host на Railway (ADR-054 §7, issue #733)

Топология: трекеры (лендинг / веб-приложение / десктоп Tauri) шлют биконы на
`umami.origa.uwuwu.net`. Имя резолвится через Bunny scriptable DNS
(скрипт 91629): мир → CNAME на edge-имя Railway `07wyrdxw.up.railway.app`
(кастом-домен `umami.origa.uwuwu.net` приаттачен к сервису), РФ → RF VM nginx
(Railway-подсети из РФ дропаются, ADR-049). Сервис — Umami v3.4.0
(PostgreSQL-only) + Railway Postgres + Valkey (Redis из шаблона, опционален).

Развёрнуто 2026-10-22. Фактические значения:

| Что | Значение |
| --- | --- |
| Проект Railway | `origa-appuru` (e7a27898), environment production |
| Сервисы | `umami` (c468ab89, образ `umamisoftware/umami:3.4.0` — запинен, шаблон ставит `postgresql-latest`), `Postgres`, `Valkey (Redis)` |
| Сервисный домен | `umami-production-5769.up.railway.app` |
| Кастом-домен | `umami.origa.uwuwu.net` (id 987565d2), edge-CNAME `07wyrdxw.up.railway.app` |
| Website id | `57c7d57f-3bd9-48cf-b9ee-ea02c95d967a` (website «Origa», domain `origa.uwuwu.net`) |
| Переменные umami | `APP_SECRET` (шаблон), `DATABASE_URL` (ссылка на Postgres), `CLIENT_IP_HEADER=x-forwarded-for` |

## 1. Railway: деплой

Шаблон Railway «Umami [Updated Oct ’26]» (`railway deploy -t umami`) создаёт
umami + Postgres + Valkey одной командой. После деплоя:

```bash
railway deploy -t umami                              # из каталога с линком на проект
railway variables --service umami --set "CLIENT_IP_HEADER=x-forwarded-for"
railway service <service-id> && railway domain umami.origa.uwuwu.net   # кастом-домен
# запинить образ (шаблон ставит rolling-тег):
railway api ... # или MCP connect-service-source: umamisoftware/umami:3.4.0
```

Проверка: `curl -fsS https://umami-production-5769.up.railway.app/script.js | head -3`.

Bootstrap (первый заход — обязательно, дефолтный `admin`/`umami` — публичный
пароль из миграции `01_init`; ротация до публикации домена):
`POST /api/auth/login` → `POST /api/users/{userId} {"password": ...}` →
website `POST /api/websites {"name":"Origa","domain":"origa.uwuwu.net"}`
(domain в v3 обязателен и валидируется регекспом хостнейма) → юзеры
`POST /api/users {"username":"umami-ru","password":"...","role":"view-only"}`
→ API-ключи `POST /api/me/api-keys {"name":"marketing"}` токеном юзера
(значение ключа отдаётся один раз). Пароли/ключи → `marketing/.env` +
инвентарь ключей (uwuwu-вики `origa/access-keys-inventory`).

## 2. RF VM: серт + nginx-ветка (ssh root@193.233.217.243)

Серт (DNS-01 через Bunny; TXT fall-through через scriptable DNS работает —
проверено 2026-10-22 на приаттаченном имени):

```bash
/root/.acme.sh/acme.sh --issue --dns dns_bunny --keylength ec-256 --server letsencrypt -d umami.origa.uwuwu.net
mkdir -p /etc/nginx/ssl/umami.origa.uwuwu.net
/root/.acme.sh/acme.sh --install-cert -d umami.origa.uwuwu.net --ecc \
  --fullchain-file /etc/nginx/ssl/umami.origa.uwuwu.net/fullchain.pem \
  --key-file /etc/nginx/ssl/umami.origa.uwuwu.net/key.pem \
  --reloadcmd "systemctl reload nginx"
```

nginx (IaC: `~/rf-proxy-infra` на ноуте и на ВМ, правки через git;
бэкап 2026-10-22 — `/root/nginx-backup-20261022/`):

- `rf-proxy-stream.conf` — строка `umami.origa.uwuwu.net → 127.0.0.1:8443` в
  `map $ssl_preread_server_name`;
- `rf-proxy-common-umami.conf` — копия common-сниппета с путями к серту
  umami (nginx не умеет переменные в `ssl_certificate`, поэтому сниппет
  дублируется);
- `rf-proxy-http.conf` — upstream `railway_umami` (сервисный домен Railway)
  - server-блок `:8443`. **`proxy_ssl_name` — сервисный домен Railway
  (`umami-production-5769.up.railway.app`), НЕ кастомный**: серт Railway
  выдаёт на сервисное имя, verify ON проверяет его; приложение получает
  кастомный хост через `Host`. (У app.origa иначе — там кастом-домен
  приаттачен давно и серт Railway покрывает его.)

```bash
nginx -t && systemctl reload nginx
# проба РФ-ветки (SNI руками, DNS не нужен):
curl -s --resolve umami.origa.uwuwu.net:443:193.233.217.243 -o /dev/null -w '%{http_code}\n' https://umami.origa.uwuwu.net/script.js
```

## 3. Bunny DNS: кастом-домен + ветка umami в скрипте 91629

Порядок (важен — Railway верифицирует кастом-домен по DNS и только потом
начинает маршрутизировать хост на edge):

1. Кастом-домен приаттачен к сервису (`railway domain umami.origa.uwuwu.net`),
   Railway даёт edge-CNAME: `umami.origa → 07wyrdxw.up.railway.app`.
2. В зоне — обычная CNAME-запись `umami.origa → 07wyrdxw.up.railway.app`
   (до деплоя скрипта ответ идёт через fallthrough: скрипт возвращает
   undefined, Bunny отдаёт записи зоны). Railway верифицирует домен,
   выпускает серт, edge начинает маршрутизацию. Без этой записи мир-ветка
   отдаёт `x-railway-fallback` 404, а РФ-ветка — 502 с edge.
3. Ветка в скрипте (репо `infra/bunny-dns/handleQuery.js`): имя в
   guard-список + мир-ветка `CnameRecord("07wyrdxw.up.railway.app", 300)`
   (РФ обслуживается общим `isRF`-блоком). Деплой:
   `bunny dns scripts deploy infra/bunny-dns/handleQuery.js 91629`.
   После деплоя скрипт отвечает раньше записей зоны — зоновую CNAME можно
   оставить как страховку (не конфликтует).
4. Проверка обеих веток:
   `curl -fsS https://umami.origa.uwuwu.net/script.js -o /dev/null -w '%{http_code}'`
   (мир) и через `--resolve ...:193.233.217.243` (РФ).
5. Rollback скрипта: `git revert` + редеплой (TTL 60).

## 4. Переключение и проверка

1. Обе geo-ветки отвечают 200 на `/script.js` (шаг 3).
2. Релиз с новым трекером (код репо, ADR-054 §7).
3. Realtime-чек (постоянный пункт релизного чеклиста): десктоп-сборка с
   `tauri.localhost` + веб с `app.origa.uwuwu.net` + лендинг — по визиту в
   Realtime на `umami.origa.uwuwu.net`.
4. Параллельная сверка с Cloud (1–2 суток агрегатов), затем деактивация
   website в Cloud за владельцем.

## Откат

- Скрипт Bunny: revert + редеплой (TTL 60) — трафик уходит с имени.
- Railway: MCP/CLI → удалить сервисы `umami`/`Postgres`/`Valkey`.
- Код: revert релизного коммита — трекер возвращается на Cloud (website id
  Cloud `0b8b69aa-9c94-41ef-b27a-f928011b797b` хранится в git-истории до
  деактивации аккаунта).
