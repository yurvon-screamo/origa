# Origa — Blog Content Hub (план)

> **Status:** Draft → HUMAN GATE
> **Date:** 2026-07-16
> **Scope:** EN + RU (первая волна). VI / KO — future phases.
> **Goal:** Захват long-tail competitor-intent и educational ключевых запросов, которых не покрывает single-page `/compare`. Паттерн заимствован у OctopusLang (7 статей-компараторов за 2 месяца), но с honest-comparison тоном Origa, не sales-pitch.

---

## Зачем блог (проблема)

`/compare` — одна страница со сводной таблицей. Этого мало для SEO: Google ранжирует отдельные страницы под отдельные запросы. Конкурент OctopusLang уже публикует по 1–2 статьи в неделю ("Migaku vs Octopus", "Best LingQ alternatives", "How to learn Japanese with anime") и захватывает organic-трафик по competitor-intent запросам. Origa пока не имеет контентного хаба — это **главный пробел** vs OctopusLang.

Keyword-обоснование: см. `docs/keyword-research-report.md` (запросы с "reddit"-subtype = evaluation intent; competitor-name keywords = high commercial intent).

---

## Структура URL

| Локаль | Pattern | Пример |
| --- | --- | --- |
| EN | `/blog/<slug>` | `/blog/anki-alternative-japanese` |
| RU | `/ru/blog/<slug>` | `/ru/blog/anki-alternativa-yaponskiy` |
| VI | `/vi/blog/<slug>` | (future) |
| KO | `/ko/blog/<slug>` | (future) |

Slug — kebab-case, keyword-rich, без транслитерации в EN (в RU slug допускается латиницей для URL-стабильности).

Каждая статья должна попасть в `sitemap.xml` с hreflang-связкой EN↔RU (когда есть перевод).

---

## Список статей (приоритизированный)

Топ — competitor-intent и zero-competition wedge. Объём на статью: 1500–2500 слов.

### EN

| # | Slug | Target keyword (из keyword-research) | Intent | Статус |
| --- | --- | --- | --- | --- |
| 1 | `anki-alternative-japanese` | `anki alternative japanese` | commercial | ✅ draft готов |
| 2 | `best-japanese-learning-app-offline` | `learn japanese offline`, `best app to learn japanese offline` | transactional | planned |
| 3 | `learn-japanese-from-manga` | `learn japanese from manga` | info | planned |
| 4 | `japanese-ocr-app` | `japanese ocr app` (near-zero competition) | commercial | planned |
| 5 | `japanese-ai-tutor` | `japanese ai tutor` (emerging) | commercial | planned |

### RU

| # | Slug | Target keyword | Intent | Статус |
| --- | --- | --- | --- | --- |
| 1 | `luchshee-prilozhenie-izucheniya-yaponskogo` | `лучшее приложение для изучения японского` (zero competition) | commercial | ✅ draft готов |
| 2 | `anki-alternativa-yaponskiy` | `anki японский`, `колода анки японский` | commercial | planned |
| 3 | `yaponskiy-s-nulya` | `изучение японского языка с нуля` (доминантный модификатор) | info | planned |

---

## Brand voice для блога (обязательно к соблюдению)

Архетип: Builder-Architect (60%) + Pragmatic Operator (30%) + Precision Educator (10%). Calm conviction, не enthusiasm.

### DO

1. **Lead with the problem** — статья начинается с боли/контекста, не с названия продукта.
2. **Concrete numbers** — размеры словарей, лимиты, версии, даты. "fast" ≠ метрика.
3. **Honest comparison** — для каждого конкурента указать, КОГДА он лучший выбор. Origa — один из вариантов, не единственный правильный.
4. **State limitations** — раздел "Known limitations" обязателен для статей о собственном продукте.
5. **First person / нейтральный эксперт** — для статей-обзоров нейтральный тон; для статей о процессе разработки — первое лицо.
6. **Vary sentence length.**
7. **Link to source** — для competitor claims ставить ссылку на официальный сайт/документацию конкурента.

### DON'T (баны HN/Reddit, AI-slop маркеры)

1. Никаких AI-маркеров: delve, harness, unlock, revolutionize, robust, seamless, cutting-edge, game-changer, empower, elevate и пр. (полный список — `rules-text-writing` + `@marketer` forbidden words).
2. Никаких superlatives без данных: "revolutionary", "best-in-class", "next-gen".
3. Никаких "I'm excited to announce" / "thrilled to share".
4. Никакого обесценивания читателя: "just", "simply", "obviously".
5. Никакого wall of text — таблицы, списки, подзаголовки.

### Origa-specific red lines (из `docs/landing-content-plan.md` §10)

- **НЕ упоминать:** pricing, точные цены конкурентов (если не верифицировано), open source, license, BSL.
- `free` / `бесплатно` — **разрешено** (приложение сейчас бесплатно).
- Конкурентов **не очернять** — только "X is best for Y, Origa for Z".

---

## Критичный фактчек-нюанс: FSRS

Origa использует FSRS. **Anki тоже использует FSRS** (добавлено в 2023, стало опцией, позже default). Поэтому статья "Anki alternative" НЕ должна утверждать "Origa uses FSRS, Anki doesn't" — это ложь.

Корректная формулировка: оба используют FSRS; разница в интеграции (Origa — FSRS с первого дня в единой экосистеме vocab+kanji+grammar; Anki — FSRS включается отдельно, экосистему собираешь сам).

Перед публикацией каждой статьи — прогнать через `.factcheck.json` (см. `@marketer` factcheck pipeline). Competitor claims требуют **минимум 1 внешнего источника** (официальный сайт/документация конкурента), не только README Origa.

---

## Техническая интеграция (отдельная задача для Tech Lead)

Блог-инфраструктура отсутствует в `origa_landing/`. После одобрения контента — отдельный tech-brief (по аналогии с `docs/seo-phase2-tech-lead-brief.md`):

- Маршрут `/blog` и `/ru/blog` в Leptos router.
- Markdown → HTML рендер (или контент как Rust-строки в `content/`, как сейчас).
- JSON-LD `Article` / `BlogPosting` schema (для rich results).
- BreadcrumbList.
- Добавление статей в `sitemap.xml` с hreflang EN↔RU.
- OG-изображения под статью.
- IndexNow-сабмит новых статей (когда IndexNow заведён — P0 из seo-phase2 brief).

Пока статьи хранятся здесь как paste-ready draft'ы для ревью.
