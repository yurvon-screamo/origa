# Origa — Стратегия распределения через внешние площадки

> **Статус:** Черновик → HUMAN GATE
> **Дата:** 2026-07-21
> **Стадия:** Pre-launch, нулевой domain authority
> **Охват:** 4 рынка (EN / RU / VI / KO)
> **Лицензия:** BSL 1.1 — учитывается для OSS-leaning площадок
> **Ограничение scope:** только **бесплатные** каналы публикации (review / guest post / listing / community mention). Платные sponsor slots, podcast advertising, paid placements — out of scope.
> **Источники:** `marketing/research/2026-07-18-*.md` (4 research-файла), `marketing/playbooks/objections-handling.md`, `marketing/strategies/origa-seo.md`, `marketing/reddit-strategy.md`, `marketing/product-hunt.md`, `marketing/koharu.md`, web research 2026-07-21

---

## 1. Краткое резюме

Внешние authority-площадки образуют третий канал распределения рядом с app stores и прямым SEO лендинга. Принцип заимствованный из platform-first distribution: вместо 6+ месяцев строить domain authority с нуля (контент + бэклинки) — Origa публикуется, обозревается или упоминается там, где аудитория изучающих японский уже собирается.

К каждой площадке применяется фильтр из четырёх критериев:

1. **Аудиторный fit** — пересекается ли аудитория площадки с wedge'ом Origa (RU/VI/KO-носители, content-driven immersion, FSRS-aware power users)
2. **Authority-сигнал** — измеримая авторитетность (подписчики, трафик, поисковая видимость, репутация обозревателя)
3. **Format-fit** — вписывается ли Origa в формат, который площадка уже публикует (review, listing, guest post, тематический thread, community mention)
4. **Бесплатность** — публикация без оплаты. Платные sponsor slots, paid reviews, advertising — out of scope этой стратегии

Три операционных принципа:

- **Тирирование.** Tier 1 (обязательно, ручной персонализированный pitch), Tier 2 (пробовать), Tier 3 (только listing / skip).
- **Сначала формат, потом pitch.** Форма запроса соответствует типу площадки. Pitch review для Tofugu отличается от guest-post запроса в J-Compass и от community-mention в Naver Cafe.
- **Учёт BSL.** OSS-leaning площадки (HN-околоток, awesome-lists) требуют отдельного framing'а; продуктовые и community-площадки принимают BSL без вопросов.

Outreach-ready: playbook в §6 даёт paste-ready шаблоны и cadence-последовательность. Этот трек не блокируется Reddit karma-milestone'ом из `reddit-strategy.md` — две дорожки идут параллельно.

---

## 2. Связь с существующими стратегиями

| Существующий артефакт | Что покрывает | Что добавляет эта стратегия |
|---|---|---|
| `origa-seo.md` | SEO собственного домена (лендинг, repo, README, тех.инфра) | Внешние authority-площадки, где Origa обозревается/листится третьими сторонами |
| `reddit-strategy.md` | Reputation-building + launch на Reddit | Всё вне Reddit: блоги, community, aggregators, niche-сообщества |
| `product-hunt.md` | Запуск на Product Hunt | Всё вне PH и Reddit |
| `objections-handling.md` | Talking points для 15 launch-возражений | Язык outreach-запросов наследует этот brand voice |
| `koharu.md` | Precedent: traction Rust-проекта через r/rust + r/LocalLLaMA + Discord + независимые дайджесты | Подтверждает ценность независимых дайджестов как канала |

Стратегия не заменяет ни один из этих артефактов. Она работает параллельно и фидит в launch-последовательность из `reddit-strategy.md` §2.

---

## 3. Карта бесплатных площадок — по рынкам

> Все размеры аудиторий — приблизительные снапшоты из web research 2026-07-21. Перед outreach'ом перепроверить. Площадки внутри тира расположены по алфавиту.

### 3.1 EN (англоязычный)

| Площадка | Тип | Аудитория | BSL | Формат | Tier |
|---|---|---|---|---|---|
| **Tofugu** (`tofugu.com`) | JP-learning блог | DA ~70, top-3 JP-блог; публиковал review Bunpro (8/10) | ✅ | Review request (формат established) | **T1** |
| **skerritt.blog** | Независимый tools-обозреватель | Публикует "Best Japanese Learning Tools" annual roundups | ✅ | Annual roundup pitch | **T1** |
| **MariaTheMillennial** (`mariathemillennial.com`) | Независимый обозреватель | Подробно обозревал MaruMori (дважды) | ✅ | Review request | **T1** |
| **Hacker News (Show HN)** | Tech aggregator | Distribution spike возможен; BSL-risk в комментах (см. §7) | ⚠️ | Show HN launch post | **T1** |
| **JLPT Samurai** (`jlptsamurai.com`) | JP-learning блог | Обозревал Duolingo JP, Migaku alternatives | ✅ | Review request | **T2** |
| **Tokyolingo** (`tokyolingo.com`) | JP-learning блог + aggregator | Публикует "Leading Japanese Learning Blogs" списки | ✅ | Listing / guest post | **T2** |
| **J-Compass** (`j-compass.com`) | Sentence mining гайды | Long-form how-to, ссылается на jpdb.io, ImmersionKit | ✅ | Guest post про mining-workflow | **T2** |
| **Mikey Does** (`mikeydoes.com`) | JP-learning анализ + глоссарий | Глоссарий Migaku, sentence mining | ✅ | Guest analysis / mention | **T2** |
| **Wordy.info, immit.co** | Независимые обозреватели | Migaku reviews 2026 | ✅ | Review request | **T2** |
| **nippon.com** | Japan-портал | High-DA Japan-interest site | ✅ | Editorial pitch (медленный цикл) | **T2** |
| **Indie Hackers** | Indie founder community | Founder-story формат | ✅ | Founders' story post | **T2** |
| **iTalki blog** (`italki.com/blog`) | Tutoring platform blog | Статьи "is-Duolingo-good-for-JP" | ✅ | Guest post | **T3** |
| **Awesome-* lists на GitHub** | Curated lists | `awesome-rust`, `awesome-tauri`, `awesome-japanese` | ⚠️ | PR на листинг | **T3** |
| **ImmersionKit** (`immersionkit.com`) | Sentence corpus (комплемент) | 500K JP-предложений из аниме/VN/dorama | ✅ | Cross-promotion | **T3** |

### 3.2 RU (русскоязычный)

| Площадка | Тип | Аудитория | BSL | Формат | Tier |
|---|---|---|---|---|---|
| **Daigaku — Японский язык** (`t.me/daigaku`) | Telegram-канал | 21,006 подписчиков; есть тег `#приложения` | ✅ | App review pitch | **T1** |
| **Nihongo — Японский язык** (`t.me/yaponskoe`) | Telegram-канал | 22,400 подписчиков; второй по размеру RU JP-канал | ✅ | App review pitch | **T1** |
| **Хабр** (`habr.com`) | Tech-блог платформа | RU dev-аудитория; статьи про Tauri/Leptos/Rust собирают traction | ⚠️ BSL поднимут в комментах | Founders'-story про Rust+Tauri+WASM стек | **T1** (stack-angle) |
| **RuStore editorial** | App store editorial | Native RU Android store; Origa уже заливает AAB (#271) | ✅ | Editorial pitch — featuring RU dev app | **T1** |
| **JAPAN_info** (`t.me/japan_info_nihongo`) | Telegram-канал | Большой JP-interest канал | ✅ | App mention | **T2** |
| **Японский язык \| Япония** (`t.me/japan_teach`) | Telegram-канал | 9,930 подписчиков | ✅ | App review pitch | **T2** |
| **ВК: крупные JP-сообщества** | VK communities | Несколько 100K+ сообществ (требует идентификации конкретных) | ✅ | Community post | **T2** |
| **VC.ru** | Tech / business блог | Принимают EdTech founders' story | ✅ | Founders' story | **T2** |
| **Пикабу** | RU Reddit-эквивалент | Есть JP-interest комьюнити | ✅ | Community post | **T3** |
| **Дзен** | Long-form blogging | JP-каналы существуют, качество ниже | ✅ | Republish | **T3** |

### 3.3 VI (вьетнамский)

| Площадка | Тип | Аудитория | BSL | Формат | Tier |
|---|---|---|---|---|---|
| **vnjpclub.com** | JP-портал + diễn đàn | Долгоживущий VI JP-портал; фокус JLPT | ✅ | Forum mention + site review | **T1** |
| **Minato Dorimu** (FB group) | Facebook-сообщество | 65,000+ участников; JLPT-фокус | ✅ | Group mention | **T1** |
| **Tokutei ginou FB groups** (несколько) | FB groups для work-visa комьюнити | Прямое попадание в wedge Origa (виза + JLPT) | ✅ | Community mention | **T1** |
| **Daruma Nihongo** (FB group) | Facebook-сообщество | "Tự học giao tiếp tiếng Nhật" + livestreams | ✅ | Group mention | **T2** |
| **nguphaptiengnhat.net** | JP-портал грамматики N1–N5 | Бесплатный VI-targeted контент | ✅ | Guest post | **T2** |
| **hoctiengnhatonline.forumvi.com** | diễn đàn | Активный VI JP-форум | ✅ | Forum thread | **T3** |
| **Dayti.vn** | Tech-блог платформа | VI dev-аудитория | ✅ | Founders' story (stack) | **T2** |

### 3.4 KO (корейский)

| Площадка | Тип | Аудитория | BSL | Формат | Tier |
|---|---|---|---|---|---|
| **JLPT 마이너 갤러리** (DC Inside) | DC gallery | Активное JLPT-комьюнити; pinned Anki-тред (160K+ просмотров) | ✅ | Community mention + Anki-import angle | **T1** |
| **Naver Cafes** (несколько JP-learning) | Naver Cafe | KO community-формат; несколько 10K–100K cafes | ✅ | Cafe mention | **T1** |
| **히라가나 일본어** (`hiragana.co.kr`) | JP-learning community site | Долгоживущий KO beginner-комьюнити + study material | ✅ | Site review | **T2** |
| **Naver Blogs** (отдельные блогеры) | Naver Blog | Naver ~50% KO-поиска; блогеры ранжируются в Naver search | ✅ | Review requests | **T2** |
| **Clien** (`clien.net`) | Tech community | KO dev community | ⚠️ BSL может быть поднят | Tech-stack founder story | **T2** |

---

## 4. Определение тиров и приоритизация

### Tier 1 — обязательно, ручной персонализированный pitch

**Критерии отбора:**

- Высокое пересечение аудитории с wedge'ом Origa (native-language learners, content-driven immersion, FSRS-aware)
- Сложившийся формат review/listing, под который Origa ложится без усилий
- Единичная точка отказа если пропустить — это площадки, где живут обзоры конкурентов и отсутствие Origa заметно

**Tier 1 (14 площадок):**

- **EN:** Tofugu, skerritt.blog, MariaTheMillennial, Hacker News (Show HN)
- **RU:** Daigaku Telegram, Nihongo Telegram, Хабр (stack-angle), RuStore editorial
- **VI:** vnjpclub.com, Minato Dorumu FB, tokutei ginou FB groups
- **KO:** DC Inside JLPT gallery, Naver Cafes

**Трудозатраты:** 2–4 часа на площадку (research + персонализированный pitch). Всего: ~35–50 часов за 6–8 недель.

### Tier 2 — стоит пробовать, semi-personalized

**Критерии:**

- Твёрдая аудитория, но более слабый fit (смежная ниша, или moderate signal)
- Подключаются после Tier 1 если остаётся время

**Tier 2 (~22 площадок):**

- **EN:** JLPT Samurai, Tokyolingo, J-Compass, Mikey Does, Wordy.info, immit.co, nippon.com, Indie Hackers
- **RU:** JAPAN_info, japan_teach, VK communities, VC.ru
- **VI:** Daruma, nguphaptiengnhat.net, Dayti.vn
- **KO:** 히라가나 일본어, Naver Blogs, Clien

**Трудозатраты:** 1–2 часа на площадку. Всего: ~25–40 часов.

### Tier 3 — низкий приоритет, listing-only / skip-if-busy

**Критерии:**

- Маленькая аудитория, или общий паттерн «active dev time непропорционален return»

**Tier 3 (~7 площадок):** iTalki blog, awesome-lists PR, ImmersionKit cross-promo, Пикабу, Дзен, hoctiengnhatonline diễn đàn.

---

## 5. Sequencing — pre-launch, launch, post-launch

### Фаза 0 — Pre-launch (сейчас → 4 недели)

Цели: подготовить все pitch-материалы, верифицировать liveness и контакты площадок, начать long-lead Tier 1 pitches (Tofugu, skerritt.blog, MariaTheMillennial — у них цикл ответа 2–6 недель).

**Конкретные действия:**

1. Построить outreach-tracker (таблица в `marketing/playbooks/outreach-tracker.md`). Колонки: Площадка, Контакт, Формат, Дата отправки, Статус, Дата followup, Заметки.
2. Персонализировать Tier 1 EN-pitch'и, используя talking points из `objections-handling.md`. Отправить в Tofugu, skerritt.blog, MariaTheMillennial.
3. Идентифицировать владельцев Naver Cafes (KO) — Naver cafes требуют join + наработку репутации перед pitch. Начать цикл join + lurk.
4. Изучить культуру DC Inside JLPT gallery — прочитать pinned-треды, понять модерационные нормы. DC Inside славится прямотой; плохо поданный промо будет ratio'нут.
5. Связаться с RuStore editorial — Origa уже заливает AAB через #271, использовать отношения.

### Фаза 1 — Launch-неделя (синхронно с Reddit launch из `reddit-strategy.md` §2)

Цели: усилить Reddit-launch синхронизированными площадками.

**Конкретные действия:**

1. Show HN-пост — синхронно с r/SideProject. См. §7 для handling'а BSL.
2. Хабр founders'-story (RU stack-angle: Rust + Tauri + Leptos + local AI).
3. Telegram-каналы pitch'и (Daigaku, Nihongo) — pitch как «RU-native JP app launched».
4. Tokutei ginou FB groups — pitch как «VI-native JP app for JLPT».
5. Naver Cafe mention'ы — pitch как «KO-native JP app с offline + OCR».
6. DC Inside JLPT gallery — Anki-import angle: «Anki 통합 덱互換 Origa».

### Фаза 2 — Post-launch (неделя 2 → 8 недель)

Цели: Tier 2-покрытие, guest posts, buildout комьюнити.

**Конкретные действия:**

1. Guest post outreach в J-Compass (EN), nguphaptiengnhat.net (VI), Хабр follow-up'ы.
2. Indie Hackers founders'-story (EN).
3. Глубокий engagement в VN tokutei ginou-комьюнити (wedge «work-visa + JLPT» там сильный).

### Фаза 3 — Ongoing (неделя 8+)

- 10:1 rule из `reddit-strategy.md` распространяется на все community-площадки (1 промо на 9 полезных вкладов).
- Ежеквартальный re-pitch в annual roundups (skerritt.blog — pitch в октябре-ноябре для inclusion в годовой список).
- Мониторить новые площадки (новые блоги, новые YouTube-каналы) и добавлять в tracker.

---

## 6. Outreach Playbook

### 6.1 Принципы pitch'а

Применяются к каждому типу площадки. Шаблоны ниже — скелеты, каждый pitch персонализируется под получателя.

1. **Ведём получателем, не Origa.** Первое предложение показывает, что вы читали их работу. Шаблонные pitch'и игнорируют.
2. **Один ask за письмо.** Review ИЛИ guest post ИЛИ listing — никогда не комбинировать.
3. **Никаких superlatives.** Origa это «приложение для изучения японского с локальным OCR, STT и FSRS», не «лучшее» и не «революционное». См. brand voice в `objections-handling.md`.
4. **BSL раскрываем upfront.** Лицензия — «BSL 1.1, source-available, free for personal use». Не говорим «open source». Скрытие BSL наносит удар по доверию при обнаружении.
5. **Делаем «да» лёгким.** Прикладываем описание в один абзац, скриншот или 30-секундное демо, ссылку на загрузку, чёткий ask.
6. **Follow-up один раз.** Через 7 дней — короткий polite bump. После этого — drop.

### 6.2 Шаблоны pitch'ей

> Язык шаблона соответствует языку площадки. Для EN-площадок — английский, для RU — русский, для VI — вьетнамский, для KO — корейский. Это часть pitch'а: native-локализация Origa сама по себе аргумент.

#### Шаблон A — Запрос review (Tofugu, skerritt.blog, MariaTheMillennial, JLPT Samurai, Wordy.info, immit.co)

```
Subject: Review candidate — Origa, a JP-learning app with local OCR + STT + FSRS

Hi [first name],

I read your [specific recent article name + one-sentence takeaway that shows
you read it]. The angle on [specific point] matches what we have been building.

I am writing about Origa — a Japanese-learning app shipping now on desktop
(Tauri) and Android. Three things differentiate it from the apps you usually
review:

1. Local OCR (NDLOCR-Lite — the National Diet Library engine) for scanning
   manga panels and textbook pages into cards.
2. Local STT (Whisper) for pronunciation feedback.
3. Native interfaces in Russian, Vietnamese and Korean — most JP apps are
   English-only, which leaves large learner populations underserved.

It is BSL 1.1 (source-available, free for personal use). Built in Rust with
Leptos 0.8 + Tauri v2. Repo: https://github.com/yurvon-screamo/origa

If this fits a future tools roundup or standalone review on [site name], I can
send screenshots, a 30-second demo video, and a review license. No pressure
either way — happy to be a source for context even if a full review is not on
your roadmap.

Best,
[name]
```

**Персонализация:** `[specific recent article name]` должен ссылаться на реальную статью за последние 90 дней. Если не можете найти статью, чтобы сослаться — не pitch'ите, вы ещё недостаточно знакомы с публикацией.

#### Шаблон B — Listing / aggregator submission (Tokyolingo, awesome-lists, vnjpclub.com, nguphaptiengnhat.net)

```
Subject: Submission — Origa for [list name / category]

Hi [name or "team"],

Submitting Origa for inclusion in [specific list or section name on their site].

Origa — приложение для изучения японского (desktop Tauri + Android). Wedge:
native UI на RU/VI/KO (большинство JP-apps EN-only), локальный OCR (NDLOCR)
+ STT (Whisper), FSRS-6, 200K+ нативных фраз с аудио.

- Landing: https://origa.uwuwu.net
- Repo: https://github.com/yurvon-screamo/origa (BSL 1.1)
- Free for personal use

[предложение о том, почему оно подходит под конкретный list — например,
"подходит под секцию offline-first tools, потому что весь OCR/STT работает
on-device"]

Готов предоставить доп. информацию.

Best,
[name]
```

#### Шаблон C — Guest post pitch (J-Compass, nguphaptiengnhat.net, iTalki blog, VC.ru, Хабр)

```
Subject: Guest post pitch — [конкретная тема, напр. "Adding OCR-mined cards
to your FSRS workflow"]

Hi [first name],

Я слежу за [site name] уже давно — особенно за [конкретная статья, к которой
отсылает pitch]. [Угол / методология / глубина] редко встречается в
JP-learning письме.

Хочу предложить guest post:

**Рабочий заголовок:** [конкретный, напр. "From manga panel to FSRS card: a
local-first OCR workflow"]

**Угол:** [один абзац — что покрывает пост, чего ещё нет на их сайте]

**Почему я:** Я разработчик Origa, JP-app с локальным OCR + STT + FSRS. Пост
будет включать скриншоты, детали конфигурации, честную секцию про лимиты
(локальный OCR CER vs. облачные альтернативы). Без хард-пиара Origa — пост
стоит на workflow, Origa как один из примеров.

Объём: [1200–1800 слов]. Бесплатно для публикации, код под BSL, оригинальные
изображения.

Если подходит — пришлю полный outline за 48 часов.

Best,
[name]
```

#### Шаблон D — Community mention (Telegram-каналы, DC Inside, Naver Cafes, FB groups, VK)

```
[Сообщение на языке комьюнити — RU/VI/KO/JA]

Здравствуйте, [admin / moderator name],

Меня зовут [first name], я разработчик Origa — приложения для изучения
японского с native [RU/VI/KO]-интерфейсом. Видел(а), что в вашем комьюнити
есть [app recommendation / resource sharing] thread, и хотел(а)
представиться перед публичным постом — вдруг есть гайдлайны, которых стоит
придерживаться.

Чем Origa отличается от [Anki / Bunpro / WaniKani — назвать инструменты,
которые ваше комьюнити уже использует]:

- Native [RU/VI/KO] UI (большинство JP-apps англоязычные)
- Локальный OCR + STT (работает полностью офлайн)
- FSRS-6 spaced repetition
- 200K+ нативных фраз с аудио

Free for personal use. BSL 1.1 (source-available). Repo: [link]

Как правильно поделиться этим с вашим комьюнити? Готов(а) сделать Q&A,
предоставить review-лицензии модераторам или ответить на вопросы в thread.

Best,
[name]
[подпись на языке комьюнити]
```

**Критично:** для KO (DC Inside, Naver Cafes) сообщение должно быть на корейском, не на английском. Для VI — на вьетнамском. Для RU Telegram-каналов — на русском. Используем locale, который Origa уже шипит — локализация сама по себе pitch.

### 6.3 Sequence и cadence

| День | Действие |
|---|---|
| 0 | Отправить initial pitch (персонализированный) |
| 7 | Follow-up #1: короткий polite bump. Новая информация если есть (новый релиз, новая фича) |
| 14 | Финальный follow-up: короткий, без давления. «Закрываю loop — рад вернуться, когда тайминг будет лучше» |
| 21+ | Drop. Двигаемся дальше. |

### 6.4 Язык pitch'а по рынкам

| Рынок | Язык pitch'а | Почему |
|---|---|---|
| EN | English | Дефолт |
| RU | Русский | Локализация — сам wedge. Pitch на RU сигнализирует, что продукт RU-native |
| VI | Вьетнамский | То же — VI-native wedge |
| KO | Корейский | То же — KO-native wedge |

**Исключение:** если владелец площадки билингвал или язык площадки английский (например Tofugu для KO audience) — pitch'им на EN.

---

## 7. Handling BSL — отдельная секция

BSL 1.1 — самый предсказуемый friction-point этой стратегии (см. `objections-handling.md` #2). Разные площадки требуют разного подхода.

### 7.1 Площадки, которые принимают BSL без вопросов

Большинство product/app/community-площадок (Tofugu, skerritt.blog, MariaTheMillennial, Telegram-каналы, Naver Cafes, FB groups, diễn đàn) обозревают приложения как продукты, а не как OSS-проекты. BSL для них нерелевантен — не упоминайте, если не спросят.

### 7.2 Площадки, где BSL поднимут в комментах

**Hacker News, awesome-lists, Хабр, Clien.**

У этих площадок техническая аудитория, проверяющая лицензии. Стратегия:

- **Раскрываем проактивно в launch-посте**, не в комменте. Скрытие триггерит HashiCorp-рефлекс.
- **Точная формулировка:** «source-available под BSL 1.1, free for personal use, конвертируется в пермиссивную лицензию после change-date, указанной в LICENSE».
- **Не называем open source.** Это самый быстрый способ потерять HN-кредититет.
- **Talking point наготове:** «BSL выбран, чтобы форк не мог добавить телеметрию и перепродать результат. Код на GitHub, каждая строка читается, комьюнити может собирать локально».
- **Ссылаемся на precedent:** Sentry, CockroachDB, сам HashiCorp (post-fork) шипят под BSL. Это уже не экзотика.

### 7.3 Площадки, фильтрующие по OSI-лицензии

**Awesome-lists со строгой OSI-политикой** (некоторые `awesome-*` на GitHub имеют maintainer-политики против non-OSI). Не воевать — submit, принять отказ, двигаться дальше. Время лучше потратить в другом месте.

### 7.4 Внутренняя consistency-проверка

Перед любым pitch'ем прогрепать `marketing/` и `origa_landing/` на: `open source`, `OSS`, `free software`, `libre`. Каждое вхождение должно либо (a) относиться к стороннему продукту, который реально OSI, либо (b) быть переформулировано в «source-available» / «free for personal use» / «BSL 1.1». Это action items B1 / I5 из `objections-handling.md`.

---

## 8. Бренд-contraints и red lines

Унаследовано из `objections-handling.md` §3 и `origa-seo.md` §10. Специфично для outreach:

1. **Никаких superlatives** в pitch'ах. «Origa делает X» — не «Origa лучший в X».
2. **Никаких «революционный» / «game-changing» / «next-generation»** или другой AI-cliché лексики (см. brand voice @marketer).
3. **Никаких эмодзи** в pitch-имейлах. Slack/Telegram DM — минимально, если того требует норма комьюнити.
4. **Честные лимиты.** Каждый pitch явно или неявно признаёт пробелы Origa (моложе Anki, нет iOS, меньше контент-библиотека чем у Migaku). Скрытие лимитов триггерит Migaku-pattern потери доверия.
5. **Никакой дискредитации конкурентов.** Anki / Bunpro / WaniKani / Migaku / MaruMori упоминаются уважительно, когда упоминаются.
6. **Framing российского происхождения.** См. `objections-handling.md` #13. Вести с миссии («изучение японского на родном языке»), не с происхождения.

---

## 9. Out of scope

Эта стратегия явно **не** покрывает:

- **Reddit** — см. `reddit-strategy.md` (отдельная дорожка, идёт параллельно)
- **Product Hunt** — см. `product-hunt.md`
- **App Store / Google Play / RuStore ASO** — отдельная дисциплина (покрыта tasks `android-release-cicd` и `ios-appstore-deployment`)
- **Платная реклама** (Google Ads, Meta Ads, и т.д.) — out of scope
- **Paid sponsor slots** (YouTube, podcasts, influencer networks) — out of scope. Добавить отдельный документ если появится бюджет.
- **Конференции / meetup talks** — возможный будущий трек, не сейчас
- **Influencer-агентства** — прямой контакт предпочтительнее; агентства добавляют markup без пропорциональной ценности на этой стадии
- **Discord community building** — отдельная задача; см. `koharu.md` для precedent'а
- **Блоги конкурентов** (Migaku blog, Kanjijo, Shinobi Japanese) — бесплатно не опубликуют, исключены
- **YouTube sponsor slots** — все платные, исключены
- **Podcast advertising** — платное, исключено

---

## 10. Roadmap и milestones

### Milestone 1 — Outreach-tracker построен (1 день)

Создать `marketing/playbooks/outreach-tracker.md`. Предзаполнить Tier 1 + Tier 2 из §3.

### Milestone 2 — Tier 1 long-lead pitch'и отправлены (1 неделя)

Tofugu, skerritt.blog, MariaTheMillennial, RuStore editorial, Хабр (draft founders'-story). У этих площадок цикл ответа 2–6 недель.

### Milestone 3 — Onboarding community-площадок (2 недели)

Join в Naver Cafes, DC Inside JLPT gallery, tokutei ginou FB groups, VK communities. Lurk, изучить нормы, помогать (10:1 rule из `reddit-strategy.md`).

### Milestone 4 — Synchronization с launch'ем (1 неделя, синхронно с `reddit-strategy.md` §2)

Все Tier 1 площадки активированы в течение 48 часов от Reddit-launch. Show HN, Хабр, Telegram pitch'и, DC Inside thread, Naver Cafe mention'ы, tokutei ginou FB posts.

### Milestone 5 — Tier 2 follow-through (3 недели)

Guest posts, listing submissions, Indie Hackers founders'-story.

### Milestone 6 — Ежеквартальный review

Re-pitch в annual roundups. Добавить новые обнаруженные площадки. Дропнуть площадки, не ответившие после 3 попыток.

---

## 11. Статус factcheck

Claims в этом документе берутся из:

- `marketing/research/2026-07-18-*.md` (4 research-файла — упоминания конкурентов, blog references, паттерны комьюнити)
- Web research 2026-07-21 (liveness площадок, размеры аудиторий)
- `objections-handling.md` (talking points, brand voice)

**Что требует верификации перед outreach'ом (по площадкам):**

- Размеры аудиторий (подписчики, участники, трафик) — перепроверить в течение 7 дней от pitch'а
- Контактные данные (email, DM-handle) — актуальность на дату outreach'а
- Площадка ещё активна (некоторые мелкие Telegram-каналы / блоги засыпают)
- Формат review всё ещё применим (частота review'ов у Tofugu; тайминг annual roundup у skerritt.blog)

**Внешние/непроверенные сигналы (помечены, не утверждаются как факт):**

- Tofugu DA estimate (~70) — приблизительно, не из прямого измерения
- Naver search-share estimate (~50%) — из keyword-research в `origa-seo.md`, не верифицировано независимо
- Minato Dorumu FB-группа размер (65,000+) — с их собственной marketing-страницы
- Telegram-подписки — с публичных страниц каналов, могут не отражать active readership

**Детальный per-claim confidence:** `.factcheck.json` (генерируется после HUMAN GATE, до отправки любого pitch'а).
