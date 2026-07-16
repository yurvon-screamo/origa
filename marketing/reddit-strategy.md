# Origa — Reddit Organic Strategy (карма 3 → launch-ready)

> **Status:** Active plan
> **Date:** 2026-07-16
> **Стартовая позиция:** 1 аккаунт существует (не turbin_y/yurvon-screamo), карма **3**, Origa на Reddit не упоминается ни разу.
> **Главный урок:** devvelox_x (NihongoPath) стартовал с кармы 3, постил raw self-promo (0:8), получил 0 upvotes, удаления модераторами, негатив. Мы идём другим путём.

---

## Главное правило (несгибаемое)

**НИКАКИХ self-promo постов Origa, пока карма < ~50–100 и нет истории полезных ответов в целевых сабреддитах.** Это не перестраховка — это технический лимит: r/LearnJapanese, r/SideProject и др. имеют пороги кармы и правила модерации. Пост с кармы 3 будет удалён, а аккаунт получет flair/тень-бан.

Каждый self-promo = 1 шт. Каждый self-promo должен быть preceded 10 полезными non-promo contributions в ТОМ ЖЕ сабреддите. **10:1 rule.**

---

## Фаза 1 — Reputation building (карма 3 → ~100, 3–5 недель)

Цель: к моменту первого self-promo аккаунт выглядит как «человек, который реально помогает и реально знает японский + Rust», а не «бот, рекламирующий свой продукт».

### Твоя экспертиза (что продавать как helpful-member, а не Origa)

| Домен | Что ты можешь давать | Где это ценно |
| --- | --- | --- |
| Японский (N5–N3 уровень) | Разбор грамматики, объяснение кандзи, помощь новичкам | r/LearnJapanese, r/japanese, r/kanji |
| Rust + Leptos 0.8 + WASM | Реальный опыт CSR/WASM на Leptos (редкая экспертиза) | r/rust, r/rust_gamedev |
| Tauri v2 (desktop + mobile) | Опыт desktop/mobile на Tauri v2 | r/tauri |
| Japanese-learning tools | Знание конкурентов (Anki/WaniKani/Bunpro/Migii/Octopus) | r/LearnJapanese (recommendation threads) |

**Ключевое:** отвечаешь из своей экспертизы, БЕЗ упоминания Origa. Origa упоминается только когда кто-то *прямо спрашивает* «а что ты используешь» — и то мягко.

### Быстрые точки кармы (low-threshold, где можно отвечать сразу)

Это threads/сабреддиты с низким порогом модерации — туда можно заходить с кармы 3 и набирать первую карму качественными ответами:

**Японский:**

- **r/LearnJapanese — еженедельный «質問 Shitsumonday» thread** (каждый понедельник). Там новичкам можно задавать и отвечать на вопросы. Низкий порог, быстрая карма за хорошие объяснения.
- **r/LearnJapaneseNovice** — поменьше, мягче модерация, та же аудитория.
- **r/kanji** — нишевый, вопросы про конкретные кандзи/чтения.
- **r/japaneseresources** — обсуждение ресурсов.

**Rust/стек:**

- **r/rust — «Hey Rustaceans! Got a question?»** еженедельные threads. Вопросы про Leptos/WASM там редки → твой ответ выделяется.
- **r/tauri** — меньше модерации, реальные вопросы про Tauri v2 mobile/desktop.

**Избегать пока карма низкая:**

- r/SideProject, r/IMadeThis — туда НУЖНО для launch позже, но сначала не светиться там с пустой историей.
- r/languagelearning — большая, но общая, медленная карма.

### Конкретные типы полезных ответов (что писать, не копируя)

Это **типы**, а не шаблоны. Пиши своими словами из реального опыта — Reddit мгновенно чует копи-паст и AI-тон.

1. **Разбор конкретного грамматического паттерна.** Кто-то спрашивает «в чём разница между は и が» — даёшь развёрнутое объяснение с примерами из своего опыта. Не «Origa объясняет грамматику так».
2. **Помощь с конкретным кандзи/чтением.** Объясняешь on/kun чтения, мнемонику, как кандзи встречается в словах.
3. **Сравнение методов/инструментов по прямому вопросу.** «Anki vs WaniKani для кандзи?» — честный разбор, когда что лучше. **Origa не упоминаешь** (или только если спросят «что используешь ты»).
4. **Rust/WASM опыт по Leptos.** Вопросы про CSR, hydration, leptos_router — отвечаешь из практики Origa (без упоминания, что это Origa).
5. **Tauri v2 mobile/desktop нюансы.** Реальные грабли, которые ты собрал.

### Таймлайн (реалистичный)

| Неделя | Действие | Целевая карма |
| --- | --- | --- |
| 1 | 5–8 ответов в Shitsumonday + r/LearnJapaneseNovice + r/tauri. Никакого promo. | ~10–20 |
| 2 | 5–8 ответов, пробуешь r/rust Hey-Rustaceans thread, r/kanji | ~25–40 |
| 3 | Уже отвечаешь уверенно, пару ответов набирают upvotes. r/LearnJapanese main | ~50–70 |
| 4 | ~100 карма, история из 25–30 полезных ответов в целевых сабреддитах | **~100 → launch-ready** |

**Не гнать.** Лучше 30 качественных ответов за месяц, чем 100 спам-комментариев. Карма от 1 хорошего разбора грамматики (50+ upvotes) стоит больше, чем от 30 «nice app!».

---

## Фаза 2 — Launch (когда карма ~100, история готова)

Только теперь готовятся paste-ready launch-post'ы. Правила:

### Куда (порядок важен)

1. **r/SideProject** или **r/IMadeThis** — первый пост. Story-format, tolerant к self-promo.
2. **r/rust** или **r/tauri** — отдельный пост, технический акцент (стек), НЕ «учу японский». Hook = Leptos 0.8 WASM + Tauri v2 + local OCR/STT.
3. **r/LearnJapanese** — **только** в среду в закреплённом «Materials Recommendations and Self-Promo» thread. Никаких отдельных постов.

### Формат launch-post (скелет, активируется в Фазе 2)

```
Title: I built an all-in-one Japanese learning app in Rust (Tauri + Leptos)

Body (~400-600 слов, story-format):
- The problem: устал сшивать Anki + WaniKani + Bunpro + YouTube + словари. 5 apps.
- Why native-language: учу японский не через английский — для RU/VI/KO нет нормальных инструментов.
- Architecture (для r/rust): Rust core, Leptos 0.8 CSR/WASM, Tauri v2, local NDLOCR + Whisper, FSRS, embedded DB.
- Killer-feature: всё в одном + native RU/VI/KO интерфейсы.
- Known limitations (честно): iOS нет, моложе Anki, меньше готовых колод, паритет desktop/mobile не абсолютный.
- Link: GitHub repo (НЕ лендинг).
- Открытый вопрос в конце: "what's missing for your workflow?"
```

**Тон:** Builder-Architect, calm. Без «🇯🇵🎉🥳» (devvelox_x-ошибка). Без «excited to share». Без superlatives.

### Чего НЕ делать на launch

- ❌ Кросс-постить один пост в 5 сабреддитов в один день (выглядит как spam-кампания).
- ❌ Просить друзей/коллег upvote (vote manipulation = перма-бан Reddit).
- ❌ Постить raw лендинг-ссылку без story.
- ❌ Emoji-heavy заголовки.
- ❌ Отвечать на каждый комментарий бот-стилем. Реально, лично, на каждый первые 2 часа (как Show HN etiquette).

---

## Фаза 3 — Build-in-public (ongoing, после launch)

- 10:1: на 1 явный self-promo = 9 полезных/дев-постов в тех же сабреддитах.
- Дев-стори: «добавил OCR card creation — вот инженерный разбор», «furigana hiding — как работает smart hiding», технические инсайты Origa.
- Часть постов — чистая экспертиза без Origa (разбор грамматики, сравнение методов изучения).

---

## Инфраструктура (отдельная задача, не блокер Фазы 1)

Для будущего **read-only мониторинга** болей (концепция Codex First Customer Finder, но без авто-постинга) — нужен Reddit script-app:

1. Создать script-app на <https://www.reddit.com/prefs/apps> (type: `script`, redirect `http://localhost:8080`).
2. Сохранить `client_id` + `client_secret` + `user_agent` в `marketing/.env` (через @tool-accessor, значения не покидают .env).
3. Read-only мониторинг через PRAW: поиск «anki alternative», «japanese learning app», болей — для тем полезных ответов.

Это **не блокер Фазы 1**. Фазу 1 можно начинать руками прямо сейчас — просто открывать r/LearnJapanese и отвечать на вопросы.

---

## Чеклист «готовность к launch» (когда можно постить self-promo)

- [ ] Карма ≥ ~100 (combined)
- [ ] ≥ 25–30 полезных non-promo ответов в r/LearnJapanese / r/rust / r/tauri
- [ ] Хотя бы 2–3 ответа с 20+ upvotes (значит тон заходит)
- [ ] Аккаунту ≥ 3–4 недели (не «появился и сразу promo»)
- [ ] Launch-post draft прошел HUMAN GATE
- [ ] Factcheck на claims в посте (особенно competitor claims и FSRS framing — см. `marketing/blog/.factcheck.json`)
