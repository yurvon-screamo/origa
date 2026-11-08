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
скрипте — удалена целиком (PR #898: umami не в guard-списке, доктрина
ADR-072 запрещает ре-аттач Railway-LE имён). Сервис — Umami v3.4.0
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
(domain в v3 обязателен и валидируется регекспом хостнейма). **С 2026-11-09
доп. юзеры не создаются** (view-only `umami-*` удалены — см. секцию 6);
если снова понадобятся — секция 6 описывает полный порядок с ловушкой
доступа к сайту. Пароли/ключи → `marketing/.env` + инвентарь ключей
(uwuwu-вики `origa/access-keys-inventory`). Дашборд-просмотр — share-линк
(секция 5).

## 2. RF VM: серт + nginx-ветка (ретировано 2026-11-06 — инструкция на случай возврата geo-split после фикса Bunny; ssh root@193.233.217.243)

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

**Текущее состояние (2026-11-09):** имя — plain CNAME →
`07wyrdxw.up.railway.app`, TTL 300, сертификат Railway VALID. Ветки umami в
`handleQuery.js` НЕТ (PR #898 выкинул её вместе с guard-записью: ре-аттач
Railway-LE имён запрещён доктриной ADR-072 — см. п.3). Порядок исходной
настройки (для истории и возврата):

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
4. Ветки в скрипте больше нет (PR #898 убрал umami из guard вместе с
   веткой; script 91629 отвечает только на s3/content, RF-ветка s3 —
   с failover-гейтом `Monitoring.getStatus`). Возврат на geo-split (если
   когда-нибудь понадобится после фикса Bunny и осознанного решения
   владельца) = добавить имя в guard + мир-ветка
   `CnameRecord("07wyrdxw.up.railway.app", 300)` + RF через общий
   isRF-блок, деплой скрипта (`bunny dns scripts deploy
   infra/bunny-dns/handleQuery.js 91629`), затем удалить зоновую CNAME и
   создать Script-запись на имя (ссылка на 91629, как у `s3.origa`).
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

## 5. Аналитика: как смотреть (настроено 2026-11-09)

### Read-only просмотр — share-линк

Публичный read-only дашборд без логина: share-URL ведётся в uwuwu-Деплойменте
(секция umami) — сознательно не дублируется в public-репо: ссылка открывает
неаутентифицированный просмотр аналитики (вкл. realtime) любому читателю репо.
ShareId хранится в поле `shareId` website; отозвать —
`POST /api/websites/{id} {"shareId": null}` токеном admin. Проверка анон-доступа:
страница рендерит цифры дашборда без авторизации (проверено headless-браузером).

### Отчёты (Reports)

| Имя | Тип | Вопрос, на который отвечает |
| --- | --- | --- |
| Onboarding funnel | funnel | Где отваливаются в онбординге (шаги в порядке `OnboardingStep` enum: intro → load → jlpt → apps → progress → summary → scoring → `onboarding_complete`) |
| Landing to Web app | funnel | Лендинг: `pageview /` → `hero_cta_download` → `open_webapp` |
| User retention (weekly) | retention | Возвращаемость недельных когорт (окно 7d) |
| Landing downloads (by platform) | breakdown | Клик-разбивка `download_*` по платформам (windows/appstore/android_rustore/android/linux/linux_rpm/linux_aur) |
| Traffic sources (UTM) | utm | Источники/кампании (utm_source/medium/campaign) |

Funnel-шаги несут per-step фильтр `hostname neq localhost` — dev-трафик
(чистый `localhost`, локальный `trunk serve`) в воронки не попадает;
прод-десктоп (`tauri.localhost`) и обе прод-поверхности (`*.origa.uwuwu.net`)
проходят. Обновление шагов: `POST /api/reports/{reportId}` с полным payload
(частичной правки нет). Retention-отчёт фильтров на уровне parameters не
поддерживает (схема — только `window`); его dev-чистота — только UI-фильтр
hostname при просмотре.

Гочи funnel-отчётов через API (обе ловили тестировщика на «Something went
wrong» в UI — карточка воронки без данных):

- `parameters.window` **обязателен** (число минут, в наших — 60): reports-POST
  его не валидирует (создастся с 200), но UI-stats-валидатор
  `GET /api/websites/{id}/funnels/{reportId}/stats` без него отдаёт 400 и
  карточка рендерит ошибку;
- тип шага — `path` или `event`, **не `url`** (reports-POST пропустит `url`,
  stats-валидатор отвергнет — снова карточка с ошибкой при валидных, казалось
  бы, данных);
- проверка до отдачи владельцу: `GET .../funnels/{reportId}/stats?startAt&endAt`
  → 200 + массив шагов.

### Где отчёты живут в UI (v3.4 — единого раздела «Reports» нет)

| Отчёт | Вкладка сайта | Как выглядит |
| --- | --- | --- |
| Onboarding funnel, Landing to Web app | **Funnels** | карточки с шагами и drop-off |
| User retention (weekly) | **Retention** | cohort-таблица (окно из parameters) |
| Traffic sources (UTM) | **UTM** | конструктор source/medium/campaign |
| Landing downloads (by platform) | **Breakdown** | Fields-конструктор: выбрать `Event` в Fields — разбивка всех событий; `download_*` фильтром по значению |

Breakdown-отчёт сохраняется валидными параметрами (`fields: ["event"]` —
строки-имена из enum, объекты и `filter` в parameters схемой не допускаются —
иначе вкладка отчёт не рендерит), но карточкой на вкладке не показывается —
вкладка работает как Fields-конструктор, сегмент `download_*` выбирается
фильтром при просмотре.

Типы отчётов v3.4.0: `attribution|breakdown|funnel|goal|heatmap|journey|performance|retention|revenue|utm`
(инсайтов-«insights» нет — их роль играют breakdown). Создание:
`POST /api/reports {"type", "websiteId", "name", "parameters"}` токеном admin.
Custom-события продукта уже вшиты в код (ADR-054 §5): onboarding step-пара,
`login_submit`/`login_{google,apple,yandex}`, `lesson_finish`, лендинг-CTA —
имена фиксированы, воронки выше завязаны на них.

### Разрезы: лендинг vs приложение vs десктоп

Сайт один на всё (domain `origa.uwuwu.net`); поверхности различаются по
`hostname` в UI-фильтре дашборда:

| hostname | Поверхность |
| --- | --- |
| `origa.uwuwu.net` | лендинг (SSR, `data-domains="origa.uwuwu.net"`) |
| `app.origa.uwuwu.net` | веб-приложение |
| `tauri.localhost` | десктоп Tauri (Windows/Linux/macOS) |
| `localhost[:1420]` | локальный dev — шум, фильтровать |

### Проверка потока данных (beacon-check)

Синтетический бикон с конкретного контура (проверено 09.11 для мира; для
РФ — тот же POST с `--resolve umami.origa.uwuwu.net:443:193.233.217.243`):

```bash
curl -sS -X POST https://umami.origa.uwuwu.net/api/send \
  -H 'Content-Type: application/json' \
  -H 'Origin: https://origa.uwuwu.net' \
  -H 'Referer: https://origa.uwuwu.net/' \
  -H 'User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36' \
  -d '{"payload":{"website":"57c7d57f-3bd9-48cf-b9ee-ea02c95d967a","hostname":"origa.uwuwu.net","language":"ru-RU","referrer":"","screen":"1920x1080","title":"Test","url":"/__runbook_beacon_check_YYYYMMDD"},"type":"event"}'
```

Успех = в ответе `{"cache":"<jwt>","sessionId":...,"visitId":...}` и URL
появляется в метрике `path` через ~1 мин (кэш метрик 60с). Гоча: куцый
payload (без `title`/`referrer`/браузерного UA) принимается со схемным
`{"beep":"boop"}`, но запись не создаётся — payload должен повторять набор
полей трекера. Синтетика попадает в данные — имя-маркер с датой делает её
заметной, не путать с реальным трафиком.

## 6. Доступы (состояние 2026-11-09)

- **admin** — единственная учётка. Креды: `marketing/.env`
  (`UMAMI_ADMIN_USER`/`UMAMI_ADMIN_PASSWORD`, ноут). Программный доступ:
  `POST /api/auth/login` → Bearer JWT.
- **view-only юзеры `umami-ru/en/ko/vi` удалены 2026-11-09.** Причины: (а) сайт
  живёт в личном пространстве admin (`teamId: null`), юзеры его не видели и их
  API-ключи отдавали 401 на любые данные (проверено 08.11: логин юзера работает,
  `GET /api/websites` у него пуст); (б) потребителей ключей в `marketing/` нет.
  Строки `UMAMI_USER_UMAMI-*`/`UMAMI_PASSWORD_*`/`UMAMI_API_KEY*` в
  `marketing/.env` — историческая запись, ключи невалидны.
- **Если view-only с API-ключом понадобится снова** (маркетолог по локалям и
  т.п.): создать юзера (`POST /api/users {"username","password","role":"view-only"}`),
  ключ (`POST /api/me/api-keys` под его токеном — значение показывается один раз)
  и **дать юзеру доступ к сайту — иначе повторится та же мёртвая схема**: сайт
  держится у admin, поэтому нужен либо transfer сайта в team с юзером, либо
  website под юзером. Ключ ходит заголовком `x-umami-api-key` (Bearer с ключом
  не работает — проверено 08.11).
- **Gotcha v3-роутов:** юзеры — `/api/users` (`POST`, `DELETE /api/users/{id}` —
  админ), список — `/api/admin/users`; `DELETE /api/admin/users/{id}` — 404.
  Шаринг — `POST /api/websites/{id} {"shareId"}`; GET на `/api/share/{slug}`
  отдаёт анон JWT токен дашборда.

### Известные ограничения

- **Dev-шум в прод-данных:** локальные dev-сборки шлют биконы (hostname
  `localhost*`) в тот же инстанс. Отличить их от прод-десктопа нетривиально:
  `TRAILBASE_URL` без env резолвится в прод (`build_defaults.rs`), поэтому
  build-переменные dev-сборку не выдают. CI/e2e и android-smoke компилируют
  трекер out (`UMAMI_DISABLED=1`); остальное — фильтр hostname в UI/отчётах.
- `bootstrap_test_event` (единично в событиях) — ручной тест API, не из кода.
- Лендинг и приложение в одном website: агрегаты «в целом по продукту»
  смешаны — разрезается hostname-фильтром (см. выше).

## Откат

- Скрипт Bunny: revert + редеплой (TTL 60) — трафик уходит с имени.
- Railway: MCP/CLI → удалить сервисы `umami`/`Postgres`/`Valkey`.
- Код: revert релизного коммита — трекер возвращается на Cloud (website id
  Cloud `0b8b69aa-9c94-41ef-b27a-f928011b797b` хранится в git-истории до
  деактивации аккаунта).
