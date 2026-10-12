# Гайд: ASO-срез App Store — по чему и как нас находят в сторе

> **Автор:** @origa_marketing_en_bot · 2026-10-12 · По задаче босса («для аппстора такую же работу»).
> Аналог поискового гайда `2026-10-12-search-visibility-toolkit.md`, витрина App Store.

Формат: инструмент → как получить данные → что смотреть → куда сохранить.

---

## 0. Два уровня

| Вопрос | Инструмент | Ключ |
|---|---|---|
| По чему нас (и конкурентов) реально находят в выдаче стора | **iTunes Search API** (безключевой) | не нужен |
| Показы/просмотры/загрузки по источнику «App Store search», keyword popularity | **App Store Connect API + UI** | ключ ASC (см. §3) |

## 1. iTunes Search API — позиции в выдаче (без ключа)

Официальный endpoint `itunes.apple.com/search` отдаёт ту же выдачу поиска витрины App Store (не топ-чарты). Лимит 200 результатов на запрос — глубже третьего экрана пользователь не листает, этого достаточно.

### Снятие позиций (рабочий скрипт: `marketing/en/scripts/` или копия ниже)

```python
import json, time, urllib.parse, urllib.request

ORIGA_ID = 6795012532  # "Origa - JLPT Learn Japanese"

def search(term, country, limit=200):
    url = "https://itunes.apple.com/search?" + urllib.parse.urlencode(
        {"term": term, "country": country, "entity": "software", "limit": limit})
    with urllib.request.urlopen(url, timeout=30) as r:
        return json.load(r)

def pos(term, country):
    d = search(term, country)
    ids = [r["trackId"] for r in d["results"]]
    return (ids.index(ORIGA_ID) + 1 if ORIGA_ID in ids else "-"), d["resultCount"]
```

Правила гигиены: `sleep` 2–3 сек между запросами (429 при серийном долблении); кириллицу/кане передавать только через urlencode (голый curl с не-ASCII отдаёт пустой ответ); country=витрина, язык терминов — язык той витрины.

### Срез 2026-10-12 (позиция Origa / размер выдачи)

| Витрина | Запрос | Позиция |
|---|---|---|
| US | japanese, learn japanese, jlpt, japanese flashcards, kanji, japanese ocr, hiragana, learn japanese offline, anki alternative | все вне топ-200 |
| RU | изучение японского, японский язык, jlpt, хирагана, японский для начинающих | вне топ-200; **кандзи 131/142** |
| KR | 일본어 공부, 일본어 앱 | вне топ-200; **jlpt 94/188** |
| VN | học tiếng nhật, jlpt, tiếng nhật | вне топ-200 |
| JP | 日本語学習, jlpt | вне топ-200 |

**Чтение результата:** ASO-видимость ≈ нулевая на всех пяти витринах. Контекст: у приложения **0 ratings** — Apple сильно взвешивает velocity и рейтинг; без базы оценок даже точное попадание ключа в заголовок не вытягивает выдачу. Топ US-выдачи («MARU», «Japanese!!», «Teuida», «LingoDeer», «renshuu», «Bunpo», «Drops») имеет 1.4k–5.5M оценок.

**Вывод:** ключевой ASO-рычаг сейчас не тексты метаданных, а база оценок и velocity (программа просить отзывов внутри приложения), потом уже итерации подзаголовка/ключевых полей под кластеры jlpt/кандзи (единственные, где мы уже в выдаче).

### Что смотреть в конкурентном срезе

Для каждого seed-ключа сохранять: топ-10 (имя, id, разработчик, рейтинг × кол-во оценок). Дельта по месяцам покажет, кто вкладывается в ASO. Устойчивые соседи по «jlpt»/«кандзи» — кандидаты в compare-страницы.

### Куда сохранить

Срезы → `docs/marketing/research/YYYY-MM-DD-aso-slice.md` (конвенция ресёрчей); скрипт → репо маркетинга.

## 2. Suggestions (безключевой, быстрый доп.) — см. гайды локалей

Подсказки поиска App Store (what users type) API не имеет; обходной путь — ручной ввод в iOS Search по витринам раз в месяц, скриншотить. Низкочастотно, но бесплатно.

## 3. App Store Connect API — «кто нас ставит» по-настоящему

Показы/просмотры/загрузки по источнику (App Store search vs browse vs web) и keyword popularity — только в ASC.

### Доступ (делает владелец)

1. App Store Connect → Users and Access → **Integrations** → App Store Connect API → Generate API key (роль **Admin** или **App Manager**).
2. Три значения → `marketing/.env` (не в чат): `ASC_KEY_ID`, `ASC_ISSUER_ID`, путь к `.p8`-файлу `ASC_PRIVATE_KEY_PATH`.
3. JWT подписывается ES256 этим ключом на 20 минут; клиент — `pyjwt[crypto]` или `appstoreconnect-api` обёртки.

### Что смотреть

- **Analytics → Sources** (UI): Impressions/Downloads by source — доля «App Store search» против browse; keyword-строку по каждому показу даёт вкладка Impressions → Search keywords (UI-only).
- **API `/v1/analytics/reports`**: отчёты по предопределённым типам (downloads/impressions breakdown). Keyword-level через публичный API не отдаётся — популярность ключей снимаем из UI-вкладки вручную раз в месяц.
- Наша цель-метрика: impressions by «App Store Search» и их конверсия в downloads по витринам RU/KR (единственные витрины с ненулевой видимостью на старте).

### Куда сохранить

Ежемесячный срез → ресёрчи по конвенции; процедуру выдачи ключа → заявка в uwuwu (проект origa).

## 4. Ритм

1. **iTunes-срез позиций — раз в месяц** (безключевой, дешёвый) + после каждой смены подзаголовка/ключевых полей.
2. **ASC Sources-срез — раз в месяц**, с дельтой.
3. После появления первых 20–50 оценок — внеочередной срез: ранжирование сдвинется резко.
