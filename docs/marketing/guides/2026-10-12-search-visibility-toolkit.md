# Гайд: снять поисковую картинку рынка за 10 минут

> **Автор:** @origa_marketing_en_bot · 2026-10-12 · По задаче lead'а («кто и как нас ставит, как трафик попадает из поисковиков»).
> База: `docs/marketing/research/2026-09-16-keyword-research.md` §15. Относится к рынку EN (Google); RU/KO/VI — см. гайды коллег.

Формат: инструмент → как получить данные → что смотреть → куда сохранить.

---

## 0. Карта инструментов (что чем отвечает)

| Вопрос | Инструмент | Доступ |
|---|---|---|
| Кто и по чему нас находит, позиции, CTR | **GSC Performance API** | service-account от владельца (сейчас НЕТ у агентов — рапорт отправлен) |
| Спрос на тему в динамике, related-запросы | **Google Trends (pytrends)** | открыт, без ключа |
| Абсолютные объёмы (fallback за отсутствием Keyword Planner) | **Bing Webmaster API** | API-ключ владельца (сейчас НЕТ у агентов) |
| Позиции по явному списку ключей | **ручная проверка SERP** (Tavily `site:` или инкогнито) | открыт |

Правило из §15: абсолюты — ДО построения любых SEO-активов; Suggest/related — только отбор кандидатов.

## 1. GSC — «кто и как нас ставит» (главный источник)

### Доступ (один раз, делает владелец)

1. Search Console → property `origa.uwuwu.net` (домен-проперти) → Настройки → Ассоциации → Service account: добавить email сервис-аккаунта с ролью **Full**.
2. JSON-ключ сервис-аккаунта — в `marketing/.env` (gitignored), имя переменной `GSC_SERVICE_ACCOUNT_JSON=<путь к файлу>`. Значения в чат не постить.
3. Проверка: `GET /webmasters/v3/sites` — свойство должно вернуться с `permissionLevel: siteFullUser` и выше.

### Снятие данных (python, `google-api-python-client`)

```python
from google.oauth2 import service_account
from googleapiclient.discovery import build

creds = service_account.Credentials.from_service_account_file(
    PATH_TO_JSON, scopes=["https://www.googleapis.com/auth/webmasters.readonly"])
svc = build("searchconsole", "v1", credentials=creds)

# ВАЖНО: siteUrl должен совпадать с типом проперти в GSC.
# У нас проперти — URL-prefix `https://origa.uwuwu.net/`; запросы к
# `sc-domain:origa.uwuwu.net` вернут 403 forbidden даже при полном доступе.
# Список доступных проперти: svc.sites().list().execute()
SITE = "https://origa.uwuwu.net/"

row = svc.searchanalytics().query(
    siteUrl=SITE,
    body={
        "startDate": "2026-09-12", "endDate": "2026-10-12",
        "dimensions": ["query"],            # + "page", "country" вторым срезом
        "rowLimit": 250,
        "dataState": "all",                  # включает fresh (недоденные) данные
    }).execute()
for r in row.get("rows", []):
    print(f"{r['clicks']:>3} clicks {r['impressions']:>5} imp  pos {r['position']:5.1f}  {r['keys'][0]}")
```

### Что смотреть (в этом порядке)

1. **Кластеры запросов**, не отдельные ключи: группируем по смыслу (OCR, offline, бренд, конкуренты). Жила = кластер с pos 5–20 и ростом imp — докрутка контентом даёт клики за недели.
2. **CTR против позиции**: pos < 10 и CTR < 2% = слабые title/meta, чинить дешевле, чем писать новое.
3. **Страницы-лидеры**: какая страница собирает imp (у нас 30% imp дал один OCR-пост).
4. **Гео**: наша база — Japan CTR 12.5% при US pos 24 без кликов (§15.1). Гео-сюрпризы меняют приоритеты локалей.
5. **Дельта к прошлому срезу** — сохранять каждый срез, чтобы видеть тренд, а не точку.

### Куда сохранить

Срез → `docs/marketing/research/YYYY-MM-DD-gsc-slice.md` (конвенция ресёрчей). Факт «доступ получен/процедура» → заявка в uwuwu (проект origa).

---

## 2. Google Trends (pytrends) — спрос и related-запросы

### Окружение

Trends работает без ключа, но ломается на urllib3 2.x (§15): пин `urllib3<2`.

```bash
uv venv .venv && uv pip install --python .venv/bin/python pytrends "urllib3<2"
```

### Снятие (рабочий скрипт: `/tmp/opencode/seo-slice/en_slice.py`, копия — ниже)

```python
import time
from pytrends.request import TrendReq

pytrends = TrendReq(hl="en-US", tz=0)
kw = ["learn japanese", "japanese ocr", "learn japanese offline"]  # ≤5 за батч
pytrends.build_payload(kw, timeframe="today 12-m", geo="")          # geo="" = worldwide
df = pytrends.interest_over_time()
print(df.mean().drop(labels="isPartial", errors="ignore").sort_values(ascending=False))

time.sleep(10)  # пауза между батчами, иначе 429
related = pytrends.related_queries()  # top/rising по первому ключу пейлоада
```

### Что смотреть

1. **Относительный интерес 0–100 внутри одного пейлоада** — сравнимы только ключи одного батча; между батчами сопоставление через общий якорь (у нас якорь «learn japanese»).
2. **related rising** — сильно шумит пересечениями («best way to learn javascript» у нас 27200% в EN-срезе). Фильтровать по смыслу, не брать топ без чтения.
3. **0.0 ≠ нулевой спрос** — это «ниже порога батча»: проверять кандидата вторым батчем с сильным якорем (learn japanese offline 0.1 при живой GSC-странице pos 10.5).

### Срез 2026-10-12 (WW, 12 мес; якорь-сет идентичен §15 для сопоставимости)

| Ключ | §15 (09-18) | Сейчас (10-12) |
|---|---|---|
| learn japanese | 72.7 | **74.7** |
| hiragana chart | 16.0 | 16.0 |
| learn japanese app | 5.9 | 5.9 |
| japanese counters | 2.6 | 2.6 |
| japanese verb conjugation | 1.8 | 1.9 |

OCR-кластер (отдельный батч с якорем): `ocr japanese` **50.4**, `japanese ocr` **50.2**, `japanese text scanner` 0.1, `learn japanese offline` 0.1, `yomiwa` 0.0.

**Вывод среза:** тренд EN-ядра стабилен за месяц; OCR-кластер в Trends держит ~2/3 масштаба хедтерма — единственная тематическая жила с одновременным подтверждением спроса (Trends) и наличия наших позиций (GSC pos 7–15). Усилия EN-контента — сюда.

### Куда сохранить

Скрипты → `docs/marketing/guides/assets/` или `/tmp` с копией в гайд; срезы → ресёрчи по конвенции дат.

---

## 3. Bing Webmaster API — абсолютные объёмы

### Доступ

API-ключ выдаётся владельцу в Bing Webmaster → Settings → API Access. По правилам репо ключ живёт в uwuwu-вики, не в чате. **Статус: агенты доступа не имеют, ключ в вики не найден (grep 2026-10-12) — рапорт отправлен lead'у.**

### Снятие (когда ключ будет)

```bash
curl -H "Authorization: Basic <apikey>" \
  "https://ssl.bing.com/webmaster/api.svc/json/GetQueryStats?siteUrl=https%3A%2F%2Foriga.uwuwu.net"
# ключевые слова: GetKeywordStats?country=vn&language=vi-VN — язык строго с дефисом (§15.2)
```

### Что смотреть

`GetKeywordStats` — единственный бесплатный источник абсолютов без Keyword Planner. Bing ≈ 3–5% рынка EN — абсолюты ориентировочные, порядок величин.

**Ретест 2026-10-12 (после RU-рапорта о деградации):** API жив — us/en-US, fr-FR, de-DE, ru-RU отдают данные (hiragana chart 436 broad imp, learn japanese 291, jlpt 176 us; fr «apprendre le japonais» 31, de «japanisch lernen» 38). Пустой `{"d":[]}` коррелирует с низким объёмом ключа, а не с рынком: на ru-RU «jlpt» отдаётся, «изучение японского»/«japanese ocr» — пусто; на us «learn japanese offline» пусто при живом «learn japanese».

**Протокол:** (1) пустой ответ = «ниже порога Bing», не «спроса нет» — важные ключи перекрёстно проверять Trends; (2) при живом ответе смотреть `BroadImpressions` за всё ретро-окно, сегменты (`segments`) приходят не всегда — конверсию в /мес считать осторожно (окно ~175 дней, недельные бакеты подтверждены на vi-VN, см. uwuwu `bing-getkeywordstats-degraded`); (3) перед абсолютами для нового рынка — контрольный живой ключ («jlpt» как якорь).

### Куда сохранить

Туда же: срезы в ресёрчи, процедуру выдачи ключа — заявка в uwuwu.

---

## 4. Ритм (предложение)

1. **GSC-срез — раз в 2 недели** (главный индикатор «кто нас ставит»), дельта к предыдущему обязательна.
2. **Trends — раз в месяц** или перед запуском новой контент-волны (§15: абсолюты до активов).
3. **Bing — точечно**, под новые рынки/кластеры, где нужен порядок объёма.
4. Каждый срез заканчивается строкой «усилия → кластер X» — иначе это коллекционирование цифр.
