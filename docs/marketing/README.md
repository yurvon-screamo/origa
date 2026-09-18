# Origa — Маркетинг и SEO (карта документов)

> **Входная точка для агентов и людей.** Всё маркетинговое собрано здесь. Обновлено: 2026-09-18.

## Быстрая навигация по задачам

| Задача | Документ |
|---|---|
| Понять спрос: кем ищут, что живо/мертво (4 локали) | [`research/2026-09-16-keyword-research.md`](research/2026-09-16-keyword-research.md) — **§15 с практической валидацией** (Wordstat/Trends/Bing API/GSC) |
| Мастер-стратегия SEO | [`strategies/origa-seo.md`](strategies/origa-seo.md) |
| План наращивания авторитетности | [`strategies/seo-authority-playbook.md`](strategies/seo-authority-playbook.md) |
| Технический SEO-бэклог | [`strategies/seo-phase2-tech-lead-brief.md`](strategies/seo-phase2-tech-lead-brief.md) (исполнен, остаток → issue #576) |
| Конкуренты: KanaDojo / Naminori / механики SRS | [`research/2026-09-18-*.md`](research/) |
| Куда постоваться (площадки) | [`strategies/origa-distribution-platforms.md`](strategies/origa-distribution-platforms.md) |
| Как постоваться (процессы, контакты) | [`playbooks/outreach-contacts.md`](playbooks/outreach-contacts.md) |
| Ответы на возражения аудитории | [`playbooks/objections-handling.md`](playbooks/objections-handling.md) |
| Reddit (карма 3 → launch-ready) | [`strategies/reddit-strategy.md`](strategies/reddit-strategy.md) |
| Позиционирование vs конкуренты (Anki/Bunpro/...) | [`product-positioning.md`](product-positioning.md) |
| Правила текста лендинга (тон, запреты) | [`../landing-content-plan.md`](../landing-content-plan.md) §10 |

**Исполнение трекается в issue [#576](https://github.com/yurvon-screamo/origa/issues/576)** (контент-конвейер + волны SEO) — там же итоговый контент-лист 4 локалей.

## Структура

```
docs/marketing/
├── README.md                 ← эта карта
├── product-positioning.md    ← позиционирование vs конкуренты (проверить актуальность)
├── strategies/               ← действующие стратегии и брифы
│   ├── origa-seo.md                      (мастер-SEO, 2026-06-26)
│   ├── seo-authority-playbook.md         (авторитетность, 2026-09-16)
│   ├── seo-phase2-tech-lead-brief.md     (тех. бриф — ИСПОЛНЕН, остаток в #576)
│   ├── origa-distribution-platforms.md   (внешние площадки, 2026-07-21)
│   ├── reddit-strategy.md                (2026-07-16, active)
│   └── product-hunt.md                   (draft, не готов)
├── research/                 ← ресёрчи, именование YYYY-MM-DD-тема.md
│   ├── 2026-09-16-keyword-research.md    (ГЛАВНЫЙ: спрос 4 локалей + §15 валидация)
│   ├── 2026-09-18-kanadojo.md            (+машина роста, утилиты, бэклинки)
│   ├── 2026-09-18-naminori.md            (+§6: что забрать/отклонено)
│   ├── 2026-09-18-grammar-relearning-ghosts.md
│   ├── 2026-07-18-*.md                   (4 sentiment-ресерча)
│   ├── 2026-06-26-keyword-research-v1.md (⚠️ superseded, pre-launch базлайн)
│   └── 2026-06-kana-dojo-promotion.md    (⚠️ superseded, история промо-каналов)
├── playbooks/                ← objections-handling, outreach-contacts
├── blog/                     ← черновики постов (draft)
└── assets/                   ← визитка, арт черновиков
```

## Конвенции

1. **Ресёрчи:** `research/YYYY-MM-DD-тема.md`. Устаревшее не удаляется — баннер `⚠️ Superseded` в шапке + ссылка на преемника.
2. **Factcheck:** JSON-файлы `*.factcheck.json` лежат рядом со своим документом.
3. **Пути в текстах:** repo-root-relative (`docs/marketing/...`) — для grep-находимости агентами; markdown-ссылки — относительные.
4. **Ключевые решения (что взять/отклонить у конкурентов)** фиксируются в шапках research-файлов (см. Naminori §6) и в issues.
5. **AGENTS.md** корня указывает сюда — при переименовании папки обновить его.

## Не заведено / известные дыры

- `playbooks/outreach-tracker.md` — упомянут в стратегиях как TODO, файл не создан (создать после HUMAN GATE)
- `koharu.md` — precedent-файл упоминается в distribution-platforms, но в репо не сохранялся
- Naver-объёмы (KO) и Google Keyword Planner-абсолюты — не измерены (нужны аккаунты), см. keyword-research §15
