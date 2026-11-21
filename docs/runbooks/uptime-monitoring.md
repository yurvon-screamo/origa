# Runbook: Геораспределённый мониторинг доступности Origa

План и вердикты ревью — задача uwuwu «Мониторинг доступности»
(20261104060619-zbrwyvq). Архитектура: **HetrixTools** (мир + серты + dead-man,
free: 15 мониторов) + **Sentry Uptime** (вторая линия на критичный URL, 1
free-монитор) + **RF-поллер на Aeza СПб ВМ** (DNS-ассерты, RF-ветка, TSPU,
серт umami). Ноль новых вендоров против Sentry/HetrixTools.

Топология, под которую собраны ассерты (live-пробы 2026-11-08): geo-split
Bunny SCR жив только на `s3` — RU-резолверы получают A `193.233.217.243`
(RF-ВМ), мир — CNAME CloudFront; `origa`/`app`/`umami` — plain CNAME Railway
для всех гео (DNS-фикс 06.11, ADR-072); `content` — Tigris CNAME без гео.

## 1. Секреты и переменные (плейсхолдеры — значения в env, не в репо)

| Переменная | Где живёт | Зачем |
| --- | --- | --- |
| `HETRIX_V3_API_KEY` | env машины, где гоняется setup-скрипт | Account API Key, `Authorization: Bearer` для v3-read |
| `HETRIX_V2_API_TOKEN` | там же | Uptime API Token для v2-write (создаётся в HetrixTools UI → Account → API). Токен едет в URL-path — **не логировать URL**, не вставлять в тикеты |
| `TG_BOT_TOKEN`, `TG_CHAT_ID` | cron-env RF-ВМ (`crontab`/env-файл) | доставка алертов поллера в Telegram |
| `HETRIX_HEARTBEAT_URL` | там же | ping-URL heartbeat-монитора «poller» из HetrixTools |
| `BUNNY_API_KEY` | там же | Bunny AccessKey для зонда метаданных edge-скрипта (`GET /compute/script`) |
| `SENTRY_*` | существующий Sentry-контур | алерты Sentry Uptime (email) |

Инструменты на ВМ: `curl`, `dig`, `openssl`, `jq` (зонд дрейфа парсит Bunny-JSON;
на чистой Ubuntu `apt install jq`).

## 2. Мониторы HetrixTools

Автоматическая настройка — `scripts/setup_uptime_monitors.py` (read v3 →
write v2, идемпотентно, `--apply` для записи). Механика API: write-CRUD
существует только в v2 (токен в URL-path, типы website/ping/smtp/agent);
v3 — read-only (Bearer). Heartbeat-мониторы API не создаёт — руками в UI.

| Монитор | Target | Keyword | Флаги |
| --- | --- | --- | --- |
| `origa-landing` | `https://origa.uwuwu.net/` | `Learn Japanese` | SSL + domain expiry |
| `app-healthcheck` | `https://app.origa.uwuwu.net/api/healthcheck` | `Ok` | SSL |
| `s3-manifest` | `https://s3.origa.uwuwu.net/manifest.json` | `generated_at` | SSL |
| `content-tigris` | `https://content.origa.uwuwu.net/` | — | SSL |

Keyword-контракт: тело никогда не содержит строк, которые отдавал бы Railway
fallback 404 (`Application not found`) — тихий слап custom-domain binding
переводит монитор в DOWN, а не врёт зелёным. Поля v2-пейлоада (по доке
вендора): `Target` — URL, `Frequency` — строка минут `"1"`, `Locations` —
объект `{"fra": true, ...}`, `FailsBeforeAlert` — 2, `SSLExpiryReminder`/
`DomainExpiryReminder` — строки-дни `"30"`, `MID` — адрес монитора при edit;
доставка алертов — через `ContactList` (полей SendEmail/SendSMS в v2 нет).
Локации мониторинга — Frankfurt, Warsaw, London, Amsterdam (кворум).
Граница идемпотентности setup-скрипта: сверка по `name`/`target`/`keyword` и
доступному в v3-листинге `ssl_expiration_warn` — ручное переключение SSL-флага
в UI вернёт edit при следующем `--apply`; остальные поля (частоту, напоминания
о днях, локации) v3-листинг не отдаёт и вне контракта — их правят вручную в UI
HetrixTools, правка `DESIRED_MONITORS` в скрипте их не вернёт.

Счёт против free-лимита 15: 4 website + 2 heartbeat = 6, запас 9. Если
включите App Sleeping на спящем сервисе — HTTP-монитор этого сервиса снимается
(иначе монитор будит сервис и ломает экономию zero-scale).

Heartbeat-мониторы (вручную в UI, HetrixTools → Uptime → Add monitor →
Heartbeat): «rf-poller» — период тишины **45 мин** (поллер ходит раз в 15 мин);
«evening-probe» — **90 мин** (часовой probe на RF-ВМ). Пинг-URL heartbeat'а
«rf-poller» = `HETRIX_HEARTBEAT_URL` в cron-env.

Алерты: Telegram-контакт (+ email fallback) → Contact List → назначить на все
мониторы. Тест-алерт: кнопка Test в настройках контакта.

## 3. RF-поллер

Скрипт: `scripts/uptime_poller.sh` (bash, проверки ниже), cron каждые 15 мин
на RF-ВМ (`193.233.217.243`):

```cron
*/15 * * * * . /root/.uptime_poller.env && /opt/uptime_poller/uptime_poller.sh >> /var/log/uptime_poller.log 2>&1
```

Установка:

```bash
scp scripts/uptime_poller.sh root@193.233.217.243:/opt/uptime_poller/
cat > /root/.uptime_poller.env <<'EOF'
export TG_BOT_TOKEN="..."
export TG_CHAT_ID="..."
export HETRIX_HEARTBEAT_URL="https://hetrixtools.com/heartbeat/..."
export BUNNY_API_KEY="..."
EOF
chmod 600 /root/.uptime_poller.env
```

Проверки и семантика (антифлап: инцидент = 2 подряд, RESOLVE при возврате,
state-файл счётчиков `/var/tmp/uptime_poller.state`):

| Проверка | Что ловит | Семантика |
| --- | --- | --- |
| `dns-ru` | Bunny SCR drift: s3@обоих RU-резолверов → строго A ВМ; origa/app/umami/content → CNAME | инцидент |
| `dns-world` | мир-ветка DNS: s3 → CloudFront CNAME, остальные CNAME (`@1.1.1.1`) | инцидент |
| `ru-resolvers` | смерть обоих RU-резолверов (Yandex anycast, один оператор) | **warn**, не инцидент |
| `rf-branch` | законтренный `--resolve` на ВМ: 200 + `server: nginx/` (версионная подпись ВМ) + отсутствие CloudFront-маркеров `via`/`x-cache`/`x-amz-cf-id` (kz1-гоча misroute); `x-tigris-*` не дискриминируется — Tigris-origin ставит их в обеих ветках | инцидент |
| `rf-railway-origa` / `rf-railway-app` | полный ТСПУ-блок Railway edge из РФ (ADR-049); residential-деградации — слепая зона | инцидент |
| `umami-cert` | openssl notAfter < 21 дня; ноль HTTP-пингов umami (zero-scale) | warn |
| `bunny-script-drift` | смена пары (ReleaseId, LastModified) edge-скрипта 91629 = деплой вне записанной процедуры (затирание параллельной сессией; код API не отдаёт — см. ранбук `infra/bunny-dns/README.md`) | немедленный **CHANGED**, не антифлап |
| heartbeat | смерть самого поллера — тишина на HetrixTools | dead-man |

Контракт `bunny-script-drift`: Bunny API не возвращает код задеплоенного
скрипта, единственный машинно-проверяемый инвариант — «последний деплой
выполнен записанной процедурой». Первый цикл после установки тихо сеет пару в
state (установка поллера — не дрейф); смена пары → ровно один CHANGED-алерт с
обеими парами, state пере-сеется; свой CHANGED на задокументированный деплой —
бесплатное пост-деплой подтверждение, необъяснимый — инцидент. Фейлы зонда
(сеть/API/ключ) идут через общий антифлап. До внедрения поллера на ВМ зонд
спит; окно до того прикрывает процедурный гард из ранбука bunny-dns.

ТСПУ-деградации CloudFront из РФ (вечерний класс ADR-059) отдельной алерт-логики
не имеют — решение владельца от 03.11 («забей»).

## 4. evening_probe.sh (RF-ВМ)

К существующему часовому probe (`/root/rf-branch-tests/evening_probe.sh`)
добавить heartbeat-строку в конец основного цикла:

```bash
curl -fsS --max-time 15 "https://hetrixtools.com/heartbeat/<evening-probe-id>" > /dev/null || true
```

Пинг-URL — из heartbeat-монитора «evening-probe» (90 мин).

## 5. Sentry Uptime (вторая линия)

Sentry UI → Uptime → Add monitor: URL
`https://app.origa.uwuwu.net/api/healthcheck`, интервал 1 мин. Алерты — email
(существующая маршрутизация Sentry). Это независимый от HetrixTools сенсор
критичного пути; free-план держит один монитор.

## 6. Учения и приёмка

1. Тест-алерт каждого канала доходит (Telegram + email).
2. Dead-man: `systemctl stop cron` (или снять крон поллера) → алерт HetrixTools
   ~45 мин; остановить evening_probe → ~90 мин.
3. Сломать keyword монитора `origa-landing` (любая абракадабра) → алерт;
   вернуть.
4. Учения warn: вырубить egress к обоим RU-резолверам (firewall) → WARN в
   Telegram за 2 цикла.
5. Антифлап: остановить nginx на ВМ → ровно один ALERT через 2 цикла, не
   шторм; поднять → один RESOLVE.
6. RF-проба: остановить nginx → `rf-branch` инцидент.
7. Дрифт-учения: повторный деплой неизменного master-скрипта
   (`bunny dns scripts deploy infra/bunny-dns/handleQuery.js 91629`) →
   ровно один CHANGED-алерт с обеими парами, следующий цикл молчит; pair в
   state обновился. Возврата не требует — CHANGED разовый по построению.
8. Повторный `setup_uptime_monitors.py --apply` не создаёт дублей (v3-read
   матчинг по имени).

## 7. Ревизия при изменении топологии

Таблица DNS-ассертов валидна для plain CNAME-топологии 06.11. Возврат
Script-записей geo-split (после платформенного фикса Railway, задача
20261104060617-ag7v6qr / Bunny-failover 20261108155113-zhiy2o5) меняет
ожидания: origa/app/umami на RU-резолверах снова станут A ВМ — обновить
константы `ORIGA_CNAME`/`APP_CNAME`/`UMAMI_CNAME`/`CONTENT_CNAME`/
`S3_WORLD_CNAME` в шапке `scripts/uptime_poller.sh` и эту страницу одним
коммитом. Domain-expiry `uwuwu.net` — флаг на `origa-landing`; продление
регистратора OnlineNIC проверять при каждом плановом релизе.
