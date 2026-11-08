# Runbook: Umami self-host на Railway (ADR-054 §7, issue #733)

Топология: трекеры (лендинг / веб-приложение / десктоп Tauri) шлют биконы на
`umami.origa.uwuwu.net`. **С 2026-11-06 имя — простая CNAME-запись зоны →
`07wyrdxw.up.railway.app` для всех гео-веток** (вариант A, ADR-072 / задача
202611060809: RF-клиенты идут на Railway напрямую, ТСПУ-риск этого имени
принят; RF-ветка через ВМ — спящий фоллбэк). Railway-серт на имя —
`VALID` с 2026-11-08 (LE, до 06.02.27; цикл issuance застрял в ISSUING с
06.11 — снялся ручным `retry-domain-certificate`). Root cause всего среза
issuance 2026-10..11 — Bunny Scriptable DNS отдаёт NODATA через Google DNS
(8.8.8.8), на котором сидят валидаторы LE/Railway; скрипт-имена валидацию не
проходят, поэтому cert-bearing имена приаттачивать к скрипту 91629 нельзя до
платформенного фикса Bunny (warning в шапке `handleQuery.js`). Ветка umami в
скрипте оставлена pre-staged (ниже isRF-блока, мир → CNAME 07wyrdxw) —
спящая, сработает только при ре-аттаче имени. Сервис — Umami v3.4.0
(PostgreSQL-only) + Railway Postgres

- Valkey (Redis из шаблона, опционален).

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
  и server-блок `:8443`. **`proxy_ssl_name` — сервисный домен Railway
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

**Текущее состояние (2026-11-08):** имя — plain CNAME →
`07wyrdxw.up.railway.app`, TTL 300, сертификат Railway VALID. Скриптовая
ветка umami в `handleQuery.js` — pre-staged, спящая (имя не приаттачено,
запросы до скрипта не доходят). Порядок исходной настройки (для истории и
возврата):

1. Кастом-домен приаттачен к сервису (`railway domain umami.origa.uwuwu.net`),
   Railway даёт edge-CNAME: `umami.origa → 07wyrdxw.up.railway.app`.
2. Верификация — TXT-записью `_railway-verify.umami.origa` (токен виден в
   `domain-status` MCP/CLI). Зоновая CNAME и NS-делегация скрипта на одном
   имени не уживаются (Bunny отказывает) — при возврате на скрипт зоновую
   CNAME придётся удалить, при откате — вернуть. С 2026-11-06 в зоне живёт
   именно зоновая CNAME (вариант A), делегации нет.
3. Гоча (root cause инцидента 2026-10..11): приаттаченное к скрипту имя
   отдаёт NODATA через Google DNS (8.8.8.8) — валидаторы LE и резолверы
   Railway на нём сидят, domain validation и реньюал ломаются. Проверка:
   `dig +short <имя> @8.8.8.8` — пусто при живом ответе от 1.1.1.1 и NS
   Bunny. Cert-bearing имена к скрипту не приаттачивать до фикса Bunny;
  `retry-domain-certificate` (Railway MCP/CLI) — штатное лечение застрявшего
   цикла при исправном DNS (umami: ISSUING с 06.11 → VALID за минуты после
   ретрая 08.11).
4. Ветка в скрипте (репо `infra/bunny-dns/handleQuery.js`): имя в
   guard-список + мир-ветка `CnameRecord("07wyrdxw.up.railway.app", 300)`
   ниже isRF-блока (РФ обслуживается общим `isRF`-блоком). Сейчас ветка
   спящая; при возврате на geo-split после платформенного фикса Bunny —
   удалить зоновую CNAME, создать Script-запись на имя (ссылка на 91629,
   как у `s3.origa`), деплой не нужен (ветка уже в целевом виде). Деплой
   скрипта при правках: `bunny dns scripts deploy
   infra/bunny-dns/handleQuery.js 91629`.
5. Проверка обеих веток (актуальна при приаттаченном имени):
   `curl -fsS https://umami.origa.uwuwu.net/script.js -o /dev/null -w '%{http_code}'`
   (мир, ждём 200) и через `--resolve umami.origa.uwuwu.net:443:193.233.217.243`
   (РФ-ветка через ВМ). Сейчас мир ходит напрямую на Railway, RF — тоже
   (вариант A); VM-путь жив как фоллбэк.
6. Rollback текущей схемы — к приаттаченному скрипту НЕ откатывать (вернёт
   NODATA-ломку issuance). Фоллбэк при деградации прямого Railway-маршрута
   из РФ — вернуть зоновую A/CNAME на ВМ (nginx-ветка и серт ВМ живы,
   продлеваются acme.sh-кроном; серт обновлён 2026-11-08).

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
