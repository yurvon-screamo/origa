# Origa — Маркетинг и SEO (карта документов)

> **Входная точка для агентов и людей.** Обновлено: 2026-09-18. Устаревшее и исполненное в репу не хранится — git-история помнит.

## Быстрая навигация

| Задача | Документ |
|---|---|
| Спрос по 4 локалям (что живо/мертво, валидация Wordstat/Trends/Bing/GSC) | [`research/2026-09-16-keyword-research.md`](research/2026-09-16-keyword-research.md) — **§15** |
| Мастер-стратегия SEO | [`strategies/origa-seo.md`](strategies/origa-seo.md) |
| Наращивание авторитетности | [`strategies/seo-authority-playbook.md`](strategies/seo-authority-playbook.md) |
| Внешние площадки (куда постоваться) | [`strategies/origa-distribution-platforms.md`](strategies/origa-distribution-platforms.md) |
| Outreach-процессы и контакты | [`playbooks/outreach-contacts.md`](playbooks/outreach-contacts.md) |
| Ответы на возражения (запуск) | [`playbooks/objections-handling.md`](playbooks/objections-handling.md) |
| Reddit (карма 3 → launch-ready) | [`strategies/reddit-strategy.md`](strategies/reddit-strategy.md) |
| Конкуренты: KanaDojo / Naminori / SRS-механики | [`research/2026-09-18-*.md`](research/) |
| Позиционирование vs конкуренты | [`product-positioning.md`](product-positioning.md) |
| Правила текста лендинга (тон, запреты) | [`../landing-content-plan.md`](../landing-content-plan.md) §10 |

**Исполнение — issue [#576](https://github.com/yurvon-screamo/origa/issues/576)** (контент-конвейер, волны SEO, итоговый лист 4 локалей).

## Структура

```
docs/marketing/
├── README.md                 ← эта карта
├── product-positioning.md    ← позиционирование vs конкуренты (свежесть не проверялась с 2026-06)
├── strategies/               ← origa-seo · seo-authority-playbook ·
│                              origa-distribution-platforms · reddit-strategy
├── research/                 ← 2026-09-16-keyword-research (главный) ·
│                              2026-09-18-{kanadojo, naminori, grammar-relearning-ghosts}
└── playbooks/                ← objections-handling · outreach-contacts
```

## Конвенции

1. **Ресёрчи:** `research/YYYY-MM-DD-тема.md`.
2. **Устаревшее и исполненное — удаляется** (история в git). Уникальные уроки перед удалением сжимаются в живой док-преемник.
3. **Черновики запусков в репе не хранятся** — генерируются при подготовке события.
4. **Factcheck:** `*.factcheck.json` рядом со своим документом.
5. **Пути в текстах:** repo-root-relative — для grep-находимости агентами.
6. **Ключи/креды в репо не хранить** (ключ Bing Webmaster API — в uwuwu-вики).

## Известные дыры

- `playbooks/outreach-tracker.md` — упомянут как TODO, не создан (после HUMAN GATE)
- `koharu.md` — precedent-файл потерян ещё до чистки (упоминания помечены)
- Не измерены: Google Keyword Planner-абсолюты, Naver (нужны аккаунты) — см. keyword-research §15
