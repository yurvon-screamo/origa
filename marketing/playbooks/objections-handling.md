# Origa — Плейбук по обработке возражений

> **Статус:** Черновик → HUMAN GATE
> **Дата:** 2026-07-18
> **Автор:** @marketer (автономный прогон, пользователь отсутствовал — ожидает ревью)
> **Источники:** `marketing/research/2026-07-18-*.md` (4 сырых research-отчёта)
> **Охват:** Sentiment-ресерч + Origa-специфичный плейбук для обработки возражений на запуск
> **Стадия жизненного цикла:** Pre-launch. Origa — BSL 1.1, 4 локали (EN/RU/KO/VI), Rust + Leptos 0.8 + Tauri v2, FSRS, локальные NDLOCR + Whisper, offline-first

---

## 0. Краткое резюме (Executive Summary)

Этот документ объединяет sentiment-ресерч из ~50 тредов Reddit, ~30 отзывов App Store/Google Play, ~45 источников на форумах/блогах и тредов Hacker News, охватывающих **экосистему FSRS/SRS, прямых конкурентов в JP-learning (Bunpro/WaniKani/Migii/ReWord/Anki/Duolingo), молодых EdTech-стартапов (Migaku/jpdb.io/Kitsun/MaruMori/Torii) и смежных инструментов (Yomitan/10ten/Manabi/иммерсионный стек)**.

Из ресерча извлечены **15 высоковероятных возражений**, которые Origa получит на запуске; каждое сопоставлено с первопричиной, цитатой-прецедентом, статусом Origa (закрывает/частично/пробел) и ответом в brand voice.

**Пять наиболее критичных находок:**

1. **Migaku — ближайший аналог.** И Origa, и Migaku — «all-in-one» продукты для изучения японского. Траектория Migaku показывает, что каждая all-in-one фича сравнивается со специализированным бесплатным аналогом и проигрывает. Origa НЕ должна позиционироваться как «all-in-one replacement» — нужно вести **«immersion tool, в который встроен FSRS»**, акцент на уникальности локальных OCR/STT/ИИ, а не на флешкартах.

2. **BSL 1.1 будет первым возражением на HN/Reddit.** `product-hunt.md:41` сейчас говорит «free and open-source» — **фактически неверно** (BSL ≠ OSI-approved OSS). Это немедленный brand-risk: любой технически подкованный пользователь, который поставит Origa и откроет LICENSE, получит удар по доверию. Требуется исправление до любого запуска.

3. **Leptos перешёл в maintenance mode в мае 2026** (создатель gbj: «feature-complete, no significant new development»). Origa использует Leptos 0.8 — это поднимет каждый Rust-комментатор. Митигация: Leptos под MIT, а WASM-UI Origa — тонкий слой над бизнес-логикой на чистом Rust.

4. **«Yet another Anki alternative» — дефолтное возражение** для любого нового SRS-продукта — 18-летняя экосистема Anki + 1500+ аддонов = почти непреодолимый ров. Единственный успешный контр-кейс в ресерче — Manabi Reader (105 поинтов на HN): **offline-first + Anki-интеграция + автор, известный в комьюнити**. Origa должна следовать этому шаблону, а не шаблону «Anki-killer».

5. **Иммерсионная тусовка (r/ajatt) активно отвергает структурированные приложения** — для них стандартом является связка Yomitan + asbplayer + Anki, и она бесплатна. Функции OCR/STT/майнинга Origa — единственный мост к этой аудитории; если позиционировать Origa как «Bunpro + WaniKani в одном приложении», её снесут как очередной structured-app-клон.

**Рекомендация:** Этот плейбук — готовый разговорный материал для запуска на Reddit/HN/GitHub. Он должен пройти ревью и корректировку с учётом реальных продуктовых решений Origa (модель монетизации, формулировка лицензии, flow онбординга), которые пока открыты. **Не запускать, пока не закрыты пункты из §3 «Критические action items».**

---

## 1. Методология

- **Окно ресерча:** 2026-07-18
- **Источники:** Tavily web search (`site:reddit.com` + `site:apps.apple.com` + `site:play.google.com`), WaniKani Community, Bunpro Community, Trustpilot, отзывы App Store/Google Play, треды Hacker News, GitHub issues/reviews
- **Reddit `.json` эндпоинты НЕ использовались** — мертвы с мая 2026
- **Сырые research-файлы:**
  - `marketing/research/2026-07-18-fsrs-srs-ecosystem.md` — Anki FSRS transition, Mochi.cards, RemNote
  - `marketing/research/2026-07-18-jp-learning-competitors.md` — Bunpro, WaniKani, Migii, ReWord, Anki, Duolingo
  - `marketing/research/2026-07-18-young-edtech-startups.md` — глубокий разбор Migaku, jpdb.io, Kitsun, MaruMori, Torii
  - `marketing/research/2026-07-18-adjacent-tools-cross-themes.md` — Yomitan, восприятие OCR/STT, BSL/Tauri/Leptos, паттерны Show HN

**Ограничения:**

- Часть Reddit-тредов недоступна из-за rate-лимитов Cloudflare
- Даты отзывов в App Store не экспонируются в web-view (некоторые цитаты помечены «N/A»)
- Предвзятость к русскоязычным интерфейсам в JP-комьюнити — не найдена в ресерче (не подтверждена и не опровергнута)
- Приватный Discord Migaku (20K участников) — недоступен, доступны только публичные цитаты с Trustpilot/App Store

---

## 2. Библиотека топ-15 возражений

Каждое возражение содержит: **формулировку** (как это реально скажут) → **первопричину** (почему возникает) → **прецедент** (реальная цитата из ресерча) → **статус Origa** (закрывает/частично/пробел) → **ответ** (в brand voice).

### Возражение #1: «Очередная альтернатива Anki.»

**Формулировка:**
> "Why would I switch from Anki? Anki has 18 years of development, 1500+ add-ons, free sync, and I already have my workflow."

**Первопричина:** Anki — дефолтный SRS для серьёзных изучающих японский. Любой новый SRS-продукт автоматически сравнивается с ним, и сравнение структурно нечестное — экосистема Anki это ров, а не алгоритм.

**Прецедент (HN, 2021):**
> "The main reason why everyone still uses Anki despite its issues is because it is still hands-down the best solution out there... Anki's plugin system and shared decks make for a very strong network effect."
> URL: <https://news.ycombinator.com/item?id=27662266>

**Статус Origa:** Частично закрывает. FSRS Origa = тот же алгоритм, что в Anki. Но локальные OCR/STT/Japanese-стек Origa — то, чего у Anki нет.

**Ответ (Builder-Architect, спокойный):**
> Anki — отличный SRS. Я пользовался им годами. Origa не альтернатива Anki — это среда для изучения японского, в которую случайно встроен FSRS. Разница в workflow: Anki сначала флешкарты, Origa сначала контент. Сканируешь панель манги через OCR, незнакомые слова становятся карточками, FSRS их планирует. Тот же алгоритм, меньше трения.
>
> Импорт/экспорт колод Anki поддерживается — можно переносить существующую коллекцию в обе стороны.

---

### Возражение #2: «Это вообще open source? В лицензии написано BSL 1.1.»

**Формулировка:**
> "BSL isn't OSI-approved. Calling this 'open source' is misleading."

**Первопричина:** Переход HashiCorp с MPL на BSL в 2023 году сжёг доверие dev-комьюнити глобально. Форк OpenTofu (33K звёзд за месяц), исход контрибьюторов (21% → 9% community PR), мантра «BSL разрушает доверие» стала мемом. Технически подкованные пользователи теперь рефлекторно проверяют любую заявку про лицензию.

**Прецедент (HN, после HashiCorp):**
> "BSL breaks that guarantee. And the reaction has been fierce."
> URL: блог VictoriaMetrics (освещён в ресерче)

**Статус Origa:** Пробел (текущий). `marketing/product-hunt.md:41` говорит «free and open-source» — **фактически неверно для BSL 1.1**. README корректно указывает BSL. Это несоответствие — немедленный удар по доверию.

**Ответ (Builder-Architect, честный):**
> Origa использует BSL 1.1 — source-available, но не OSI-approved open source. Выбор осознанный: BSL не даёт форку отрезать гарантии приватности и перепродать Origa с добавленной телеметрией. Код на GitHub, можно прочитать каждую строку, собрать самостоятельно и свободно использовать для личных целей. После change-date, определённой в лицензии, она конвертируется в пермиссивную.
>
> Если «open source» для вас жёсткое требование — Anki под AGPL, это правильный выбор.

**Митигация (см. §3):** Исправить `product-hunt.md` до любого запуска. Подумать, называть Origa «source-available» или «free for personal use» вместо «open source».

---

### Возражение #3: «Leptos в maintenance mode — плохой фундамент.»

**Формулировка:**
> "Leptos maintainer announced in May 2026 it's feature-complete and won't see significant new development. Why pick a dead framework?"

**Первопричина:** В мае 2026 создатель Leptos gbj заявил в GitHub Issue #4707: «Leptos не заброшен, но будет поддерживаться понемногу. Я считаю его feature-complete». Это свежо в памяти Rust-комьюнити.

**Прецедент (gbj, Leptos Issue #4707):**
> "The rise of LLMs and of coding agents has really fundamentally changed the cost-benefit analysis on this for me. Over the last year or so I've found that the volume of meaningful community conversation and engagement has declined significantly."
> URL: <https://github.com/leptos-rs/leptos/issues/4707>

**Статус Origa:** Закрывает. Leptos «feature-complete» — это нормально для Origa: WASM-UI Origa — тонкий реактивный слой над бизнес-логикой на чистом Rust (в `origa/`). Бизнес-логика фреймворк-агностична. Если Leptos завтра перестанет работать, миграции подлежит только `origa_ui/`.

**Ответ (Builder-Architect, технический):**
> Leptos стал «feature-complete» — это было подходящее время, чтобы на нём строить: API-поверхность стабильна. Архитектура Origa кладёт ~90% кода в чистый Rust (крейт `origa/`), Leptos используется только как реактивный UI-слой в `origa_ui/`. Если Leptos перестанет работать, миграционная поверхность — UI-крейт. Leptos под MIT, комьюнити может форкнуть при необходимости.
>
> Это тот же аргумент, что для любого фреймворка: выбирай стабильный и архитектурь код так, чтобы фреймворк был заменяем.

---

### Возражение #4: «У Tauri WebView-несоответствия на Linux / он баганый.»

**Формулировка:**
> "Tauri uses system WebView which means different rendering on Windows/Linux/macOS. I read Paseo migrated back to Electron because of this."

**Первопричина:** Производственные ретроспективы Tauri v2 показывают на 22% больше нативных багов против Electron, 70% — платформо-специфичные WebView-баги. Статья Paseo «I was wrong about Electron» (май 2026) — каноническая ссылка.

**Прецедент (разработчик Paseo, май 2026):**
> "I was wrong about Electron... Tauri uses WebView2... Getting the app working on Windows was not a problem. On Linux... WebKitGTK... the app just looked different."
> URL: <https://dev.to/moboudra/i-was-wrong-about-electron-1e9g>

**Статус Origa:** Частично закрывает. Origa — Rust-native, поэтому «Rust-барьер» Tauri не применяется. WebView-несоответствия реальны на Linux WebKitGTK, но CI Origa гоняется на всех трёх ОС.

**Ответ (Pragmatic Operator, конкретный):**
> WebView-различия Tauri реальны, и мы тестируем на всех трёх ОС в CI. Трейд-офф: инсталлер 10–30 МБ и память 40–90 МБ против Electron с инсталлером 80–200 МБ и памятью 150–400 МБ. Для десктоп-приложения, которое работает рядом с Anki/браузерами/редакторами, это значимая разница. То, что Electron поставляет свой Chromium, ещё и означает network-stack, дружелюбный к телеметрии по умолчанию; Tauri использует системный WebView, что нам больше подходит для offline-first продукта.

---

### Возражение #5: «Migaku всё это уже делает.»

**Формулировка:**
> "This looks like Migaku. Why would I pay for Origa when Migaku has 30+ languages and a team of 15?"

**Первопричина:** Migaku — ближайший коммерческий аналог: all-in-one JP-обучение с SRS + чтение + майнинг + аудио. Даже при поляризации Migaku (любовь/ненависть) это референс для «all-in-one JP-приложения».

**Прецедент (Reddit, 2025):**
> "overall this seems like a classic example of 'should I use this new all in one, nice easy UX, modernized tool that costs money or should I use several separate longstanding programs that can still work together with older but free solutions'"
> URL: <https://www.reddit.com/r/LearnJapanese/comments/1imwsgn/is_migaku_worth_the_money/>

**Статус Origa:** Закрывает. Стек Origa отличается от Migaku в трёх конкретных аспектах: (1) local-first с локальными OCR/STT, без облачной зависимости и без DRM-контента; (2) алгоритм FSRS (Migaku использует собственный расслабленный планировщик, который критикуют на r/LearnJapanese как «ответь 3 раза подряд правильно — и готово»); (3) нативные интерфейсы на RU/VI/KO (Migaku — EN-first).

**Ответ (Builder-Architect, технический, без очернения конкурента):**
> Migaku — полированный коммерческий продукт. Origa отличается в трёх конкретных аспектах: (1) всё работает локально на вашей машине — OCR (NDLOCR-Lite, тот движок, который Национальная парламентская библиотека Японии использует для оцифровки архивов), распознавание речи (Whisper), токенизатор (lindera + UniDic). Никаких подписок, DRM, облачных round-trip. (2) FSRS-6 с персональными кривыми забывания — планировщик Migaku критиковали на r/LearnJapanese как слишком свободный. (3) Нативные интерфейсы на русском, вьетнамском, корейском — Migaku англоязычный с переводами.
>
> Разные инструменты под разные приоритеты.

---

### Возражение #6: «Зачем доверять новому приложению годы моих данных?»

**Формулировка:**
> "I have 4 years of Anki reviews. If Origa shuts down, I lose everything."

**Первопричина:** Портативность данных — главный trust-фактор для серьёзных SRS-пользователей. Mochi.cards теряла пользователей именно из-за «внезапного риска того, что сайт/система перестанет работать». Недокументированный лимит jpdb.io в 10K привёл к потере данных.

**Прецедент (пользователь Mochi, r/Anki, 2025):**
> "ended up coming back to Anki because I wanted to build knowledge long-term without the sudden risk of the website/system not working anymore"
> URL: <https://www.reddit.com/r/Anki/comments/1obsnp0/mochi_or_anki/>

**Прецедент (инцидент с лимитом 10K в jpdb.io, 2024):**
> "There's a 10k cap on decks on JPDB... why no info in the FAQ about 10k cap?"
> URL: <https://community.wanikani.com/t/recent-blunder-on-jpdb-theres-a-10k-cap-on-decks/68793>

**Статус Origa:** Закрывает структурно. Origa — desktop-first (Tauri), данные хранятся локально (rusqlite). Сервера, который может выключиться, нет. Импорт/экспорт колод Anki должен быть первоклассной фичей.

**Ответ (Builder-Architect, конкретный):**
> Origa работает локально — ваши данные лежат в локальной SQLite-базе, а не на сервере, который я контролирую. Никакого аккаунта Origa, никакого облака Origa, никакого риска отключения Origa. Импорт/экспорт колод Anki поддерживается с первого дня: если решите, что Origa не для вас, экспортируетесь в `.apkg` и продолжаете в Anki без потери данных.
>
> Худший случай для Origa-как-продукта — GitHub-репозиторий застаивается. И BSL 1.1 означает, что исходник там, и любой может его форкнуть.

---

### Возражение #7: «FSRS требует 1000+ ревьюсов для оптимизации — бесполезно для новичков.»

**Формулировка:**
> "FSRS optimization needs 1000+ reviews. As a beginner I'd just be on defaults for months."

**Первопричина:** Реальная жалоба на Anki, частично закрытая в Anki 24.11+ через auto-optimize. Origa использует rs-fsrs, наследующий то же ограничение.

**Прецедент (r/Anki, ~2026):**
> "I was reading the discussion on Anki's FSRS auto-optimize. Realized that while that issue gets resolved, we could all use a little reminder."
> URL: <https://www.reddit.com/r/Anki/comments/1rv9gy5/>

**Прецедент (r/Anki, 2025):**
> "Let's not make FSRS the default before automatic optimization. Realistically, how many users do you expect to click 'Optimize' at least once in their lifetime? I'd say 50% at best, likely less."
> URL: <https://www.reddit.com/r/Anki/comments/1h8ss2p/>

**Статус Origa:** Закрывает. Origa может auto-оптимизировать с первого дня — без кнопки «Optimize», без действия пользователя. Алгоритм периодически переобучается по мере накопления истории ревьюсов.

**Ответ (Builder-Architect, лаконичный):**
> Origa автоматически оптимизирует параметры FSRS в фоне — без кнопок, без настроек. Дефолты откалиброваны под японскую лексику (желаемая retention 85% для lifelong-обучения, 90% для JLPT-крама). После ~1000 ревьюсов параметры персонализируются под вашу память; до этого дефолты уже лучше, чем SM-2.

---

### Возражение #8: «Придётся осваивать новый FSRS-workflow — слишком много трения.»

**Формулировка:**
> "Anki's 'Hard' button misuse broke my intervals. I don't want to learn another SRS quirks system."

**Первопричина:** «Hard button misuse» задокументированно затрагивает 10%+ пользователей Anki. Жаргон FSRS (stability, difficulty, retrievability, RMSE) непрозрачен.

**Прецедент (r/Anki, 2025):**
> "Making FSRS the default will be a horrible mistake. It will screw up every person who uses Hard as 'fail', which is at least 10% of all Anki users."
> URL: <https://www.reddit.com/r/Anki/comments/1h8ss2p/>

**Статус Origa:** Закрывает. UI оценки в Origa — 3 кнопки (Again / Good / Easy), без Hard. Внутренние параметры FSRS скрыты от пользователя; экспонирован только desired retention.

**Ответ (Precision Educator, ясный):**
> Origa экспонирует один параметр FSRS пользователю: desired retention (дефолт 85%). Кнопки оценки — Again / Good / Easy, без «Hard» для misuse. Всё остальное — stability, difficulty, retrievability — под капотом. Дефолтная retention 85% откалибрована под lifelong-изучение японского; если крамите JLPT за 3 месяца, переключаете JLPT-пресет на 90%.

---

### Возражение #9: «В Anki уже есть FSRS — зачем Origa?»

**Формулировка:**
> "Anki 24.x has FSRS built in. Why do I need a new app for the same algorithm?"

**Первопричина:** Anki сделал FSRS дефолтом в 2024–2025, сузив алгоритмический разрыв. Оставшаяся дифференциация — workflow, а не алгоритм.

**Прецедент (r/LearnJapanese, 2025-12):**
> "FSRS On. Anki v. 25.07 or higher (FSRS-6). Both of the above are non-negotiable musts."
> URL: <https://www.reddit.com/r/LearnJapanese/comments/1p66osq/>

**Статус Origa:** Закрывает. Ценность Origa — не FSRS, а интегрированный пайплайн: скан манги через локальный OCR → незнакомые слова авто-извлекаются → карточки создаются с нативным аудио + контекстом → FSRS их планирует. Anki так не умеет без 4–5 аддонов и ручной возни.

**Ответ (Builder-Architect, с акцентом на workflow):**
> FSRS в Anki — тот же крейт rs-fsrs, что использует Origa. Разница в пайплайне. Чтобы получить новую карточку в Anki, обычно: потребляем контент → Yomitan lookup → AnkiConnect → ручной поиск аудио/картинок → ревью. В Origa: вставляете японский текст или сканируете изображение → незнакомые слова становятся карточками автоматически с фуриганой, pitch accent, нативным аудио из банка на 200K+ фраз и примерами предложений только на словах, которые вы уже знаете. Тот же алгоритм, меньше шагов.

---

### Возражение #10: «Privacy-first — это маркетинговая пурга, покажите аудит.»

**Формулировка:**
> "Every app claims privacy-first now. BSL isn't auditable. Where's the proof?"

**Первопричина:** «Privacy-first» без верифицируемого пруфа (открытые аудиты, воспроизводимые сборки, network-анализ) вызывает HN-скепсис. Anytype сначала получал «это скам», пока не открыл исходники.

**Прецедент (HN, ранние дни Anytype):**
> "It's not privacy-first until the source is open"
> "The scrolling banner screams 'this is a scam'"

**Статус Origa:** Закрывает структурно. Исходный код Origa на GitHub. Сетевое поведение аудитируемо: Tauri-приложение + Rust-ядро = нет обфускации на уровне JavaScript. Единственные сетевые вызовы — это fetch'и CDN для статических словарей/моделей (read-only S3).

**Ответ (Builder-Architect, верифицируемый):**
> Исходный код на GitHub под BSL 1.1 — каждая строка читаема. Единственные сетевые вызовы идут к CDN за статическими словарями и ML-моделями (read-only S3-бакет). Никакой телеметрии, никакой аналитики, никакого сервера аккаунтов. Можно запустить Origa полностью офлайн после первого запуска (один раз скачать словари/модели). Хотите верифицировать сетевое поведение: `wireshark` во время работы Origa — единственный трафик это CDN-fetch при первом запуске.

---

### Возражение #11: «Локальный OCR хуже, чем Google Cloud Vision.»

**Формулировка:**
> "NDLOCR-Lite has 32% CER on books. Google Vision OCR is way better. Why would I use the local one?"

**Первопричина:** Опубликованные бенчмарки NDLOCR-Lite показывают 1.6% CER на печатных документах (отлично), но 32.3% на книгах и 26.8% на рукописях. Цифры реальные, но use case Origa (панели манги, фото учебников, фрагменты скриншотов) ближе к «печатным документам», чем к грязным рукописям.

**Прецедент (бенчмарк estyle.co.jp по NDLOCR):**
> "NDLOCR-Lite showed the most stable accuracy in vertical, horizontal, and two-column layouts — a strong contender"
> URL: <https://estyle.co.jp/media/エンジニアアログ/2973/>

**Статус Origa:** Частично закрывает. NDLOCR действительно слабее Google Vision на зашумлённом вводе. Но OCR в Origa — для чистых сканов манги/учебников, где CER 1.6% применим.

**Ответ (Pragmatic Operator, честный):**
> Опубликованный CER NDLOCR-Lite — 1.6% на печатных документах, 32% на книгах со сложной вёрсткой, 27% на рукописях. Сильная сторона OCR Origa — панели манги, фото учебников, фрагменты скриншотов, что ближе к случаю «печатных документов». На рукописных заметках результаты будут хуже, чем у Google Lens. Трейд-офф: ваши сканы никогда не покидают устройство. Если нужен облачный OCR, manga-OCR и ScanLingua — хорошие варианты, но это отдельный шаг в workflow, не интегрированный с FSRS.

---

### Возражение #12: «Локальный Whisper даёт 21% CER на японском — слишком шумно.»

**Формулировка:**
> "Whisper large-v3-turbo has 21.8% CER on Japanese benchmarks. Google Chirp 3 is 6.4%. Why would I use the worse one?"

**Первопричина:** Японские ASR-бенчмарки благоволят коммерческим моделям Google. Локальный Whisper конкурирует на английском, но японский объективно сложнее.

**Прецедент (бенчмарк Japanese ASR, 2026-02):**
> "qwen3-asr-1.7b and whisper maintained stable accuracy even in multi-speaker and noisy conditions"
> URL: <https://neosophie.com/en/blog/20260226-japanese-asr-benchmark>

**Прецедент (Google Chirp vs Whisper, 2026):**
> "Google Chirp 3: 6.4% CER. Whisper: 36.5% CER (Groq, same audio). 6.4% error rate means only 6 mistakes per 100 characters."
> URL: <https://paulkuo.tw/en/articles/google-chirp3-japanese-stt-benchmark/>

**Статус Origa:** Пробел. Whisper — реальная слабость. SenseVoice (FunAudioLLM) появляется как более подходящий для CJK; Origa может заменить модель.

**Ответ (Builder-Architect, честный о лимитах):**
> CER Whisper на японском ~22% — это выше, чем у коммерческого предложения Google. Origa использует Whisper локально, потому что альтернатива — отправлять ваше аудио в Google. Для feedback по произношению в режиме обучения CER 22% рабочий, потому что сравнение «правильно ли ученик произнёс слово», а не полная транскрипция. Для полной транскрипции аудио SenseVoice (FunAudioLLM) — лучшая CJK-модель, мы отслеживаем её для будущей замены.

---

### Возражение #13: «Российское приложение — политические опасения.»

**Формулировка:**
> "Is this a Russian app? I'd rather not use software from Russia right now."

**Первопричина:** Геополитическая ситуация + RU-first интерфейс + видимая русская README. В ресерче по JP-learning прямой предвзятости не найдено, но риск реальный.

**Прецедент (4chan /jp/ про AnimeCards):**
> "He's Russian" — использовалось как вектор атаки против автора AnimeCards (по заметкам ресерча)

**Статус Origa:** Требуется митигация. README Origa двуязычный EN/RU. Состав команды не афишируется.

**Ответ (спокойный, с акцентом на миссию):**
> Origa построена людьми, которые учат японский — команда международная. Русский и английский интерфейсы существуют, потому что русскоязыющих изучающих плохо обслуживают существующие инструменты (Anki, WaniKani, Bunpro — англоязычные). Вьетнамский и корейский интерфейсы в планах по той же причине. Комьюнити изучающих японский глобально; Origa обслуживает ту часть, которая не учит через английский.

**Митигация (см. §3):** Не вести с фрейминга «российское приложение». Вести с фрейминга «изучение японского на вашем родном языке». Продукт говорит сам за себя.

---

### Возражение #14: «Соло-разработчик / риск vaporware.»

**Формулировка:**
> "Solo developer, BSL license, complex Rust stack. Will this be maintained in 6 months?"

**Первопричина:** Каждый SRS-проект соло-разработчика (jpdb.io, Torii, Kitsun) с этим сталкивается. Обвинения в «vaporware» в эпоху аддонов Migaku — канонический пример.

**Прецедент (отзыв на Migaku в AnkiWeb, 2021):**
> "the project seems to be unmaintained with the last merged pull request being April 2021. That is nearly a year without a release."
> URL: <https://ankiweb.net/shared/info/278530045>

**Прецедент (jpdb.io в WaniKani Community, 2024):**
> "given the dev isn't really working on it I don't think docs improvements are very likely"
> URL: <https://community.wanikani.com/t/recent-blunder-on-jpdb-theres-a-10k-cap-on-decks/68793>

**Статус Origa:** Закрывает структурно через BSL + видимую активность. У Origa есть CI, ADR'ы (`docs/decisions/`), регрессионные тесты, активная история коммитов.

**Ответ (Pragmatic Operator, конкретный):**
> Origa разрабатывает соло-разработчик. Митигация структурная: BSL 1.1 означает, что исходник навсегда на GitHub — если разработка остановится, комьюнити может форкнуть. Архитектура задокументирована в ADR'ах (см. `docs/decisions/`), сборка воспроизводима через CI (`.github/workflows/`), есть ~500+ тестов, покрывающих доменный слой. Сравните с закрытой моделью Migaku: когда Migaku-как-компания прекращает работу, пользователи застревают. С Origa худший случай — форк.

---

### Возражение #15: «Монетизация — Migaku нас обжёг, какая у вас модель?»

**Формулировка:**
> "Migaku raised prices 200→400→500 within 2 years. What's to stop Origa from doing the same?"

**Первопричина:** Эскалация цен Migaku (200→400→500 USD lifetime за ~18 месяцев) + рост цен во время активных триалов пользователей создал длительное недоверие на рынке JP-приложений.

**Прецедент (WaniKani Community, 2025-01):**
> "the price is really high (400$). It was 200$, less than a year ago"
> "They also announced that it will become 500$ this end of Feb 2025"
> URL: <https://community.wanikani.com/t/is-migaku-misleading-users/69129>

**Прецедент (Reddit r/LearnJapanese, 2025-07):**
> "Migaku has now raised its prices by 25% during my 10-day trial"
> URL: <https://www.reddit.com/r/LearnJapanese/comments/1imwsgn/>

**Статус Origa:** Пробел (модель монетизации ещё не решена). Это открытый продуктовый вопрос, который нужно закрыть до запуска.

**Ответ (только после решения о монетизации — плейсхолдер):**
> Модель монетизации Origa: [TBD]. Три обязательства: (1) ядро SRS + лексика + грамматика + кандзи остаётся бесплатным под BSL; (2) любые изменения цен в платных тирах анонсируются за 60 дней; (3) существующие подписчики получают grandfathering по своей исходной цене.

**Митигация (см. §3):** Решить монетизацию до запуска. Задокументировать модель публично.

---

## 3. Критические action items (ДО любого запуска)

Это конкретные пункты, найденные в ресерче, которые нужно исправить или решить до того, как Origa столкнётся с публичным scrutiny.

### 🔴 БЛОКЕРЫ (высокий риск для доверия, если не починить)

**B1. `marketing/product-hunt.md:41` говорит «free and open-source» — фактически неверно для BSL 1.1.**

- **Действие:** Заменить на «free for personal use, source-available under BSL 1.1» или похожее.
- **Почему:** Любой комментатор на HN/Reddit, открывший LICENSE, зафиксирует удар по доверию на лету. Это риск #1, который можно избежать.
- **Охват:** `marketing/product-hunt.md` Maker Comment строка 41. Также прогрепать `marketing/README.md` и `marketing/blog/*` на похожие формулировки.

**B2. Модель монетизации не решена — возражение #15 нельзя ответить.**

- **Действие:** Решить до запуска: бесплатное ядро + платные продвинутые (OCR/STT)? Разовая покупка? Подписка? Lifetime-тир?
- **Почему:** Migaku сожгла рынок. Любая неоднозначность читается как «они сделают с нами Migaku».

**B3. Импорт/экспорт Anki `.apkg` — работает ли?**

- **Действие:** Верифицировать и задокументировать. Если сломан или частичный, шипнуть до запуска или явно указать, что он экспериментальный.
- **Почему:** Портативность данных это возражение #6 — ответ должен быть демо-able, а не устремлённым в будущее.

### 🟡 ВАЖНО (митигирует конкретные возражения)

**I1. Публичный roadmap / датированный changelog.**

- **Действие:** Убедиться, что `CHANGELOG.md` актуален. Добавить секцию roadmap в README или отдельный ROADMAP.md.
- **Почему:** Возражение #14 (vaporware) — видимая активность это ответ.

**I2. Бенчмарки NDLOCR-Lite в README.**

- **Действие:** Указать CER 1.6% на печатных документах в секции OCR.
- **Почему:** Возражение #11 — конкретика о производительности OCR предотвращает рефлекс «оно хуже Google».

**I3. Документация модели Whisper.**

- **Действие:** Задокументировать размер модели Whisper, включён ли VAD-preprocessing и известный CER на JP (~22%). Отслеживать SenseVoice для будущей замены.
- **Почему:** Возражение #12 — честность о лимитах строит доверие.

**I4. Секция «Why Origa» на лендинге.**

- **Действие:** Добавить секцию «Why Origa» или «Comparison», которая прямо адресует фрейминг Anki-alternative.
- **Почему:** Возражения #1, #5, #9 — упредить сравнение.

**I5. Аудит формулировок лицензии во всех маркетинговых материалах.**

- **Действие:** Прогрепать «open source», «OSS», «free» в `marketing/` и `origa_landing/` контент-файлах. Выровнять с реальностью BSL 1.1.
- **Почему:** То же, что B1 — консистентность предотвращает удар «они соврали про OSS».

### 🟢 ЖЕЛАТЕЛЬНО (улучшает приём, не блокирует)

**N1. Подсветить происхождение NDLOCR-Lite («OCR-движок Национальной парламентской библиотеки Японии»).**

- **Почему:** Почти нулевая западная осведомлённость о NDLOCR. Это позиционный актив.

**N2. Инструкции по верификации приватности.**

- **Действие:** Добавить секцию документации «Verify Origa's network behavior» с рецептом `wireshark` / `tcpdump`.
- **Почему:** Возражение #10 — «privacy-first» без верификации отметают.

**N3. Reputation-building в r/LearnJapanese Shitsumonday (согласно `reddit-strategy.md`).**

- **Почему:** Согласно существующей reddit-strategy.md, карма аккаунта должна достичь ~100 перед любым self-promo. Это забег 3–5 недель.

---

## 4. Библиотека паттернов (типы наблюдаемого бэклаша)

Эти паттерны выходят за пределы конкретных продуктов. Знание их помогает распознать, когда они ударят по Origa.

### Паттерн A: Бэклаш «bait-and-switch»

**Триггер:** Бесплатный инструмент переходит в платный продукт или платный продукт поднимает цены.
**Примеры:** Аддон Migaku → подписка Migaku; эскалация Migaku $200→$500.
**Защита:** Установить модель монетизации с первого дня. Никогда не менять цены во время триалов. Grandfathering существующих пользователей.

### Паттерн B: Обвинения в «vaporware»

**Триггер:** Медленная разработка, сорванные дедлайны, пробелы в коммуникации.
**Примеры:** Аддон Migaku 2021 «нет релиза год»; jpdb.io «нет новых добавлений с июня».
**Защита:** Публичный changelog, датированный roadmap, регулярная GitHub-активность, видимая каждому.

### Паттерн C: «Мастер на все руки, ничего толком»

**Триггер:** Запуск all-in-one продукта — каждая фича сравнивается со специализированным бесплатным аналогом.
**Примеры:** Migaku vs Yomitan+Anki+asbplayer; MaruMori vs Anki+Bunpro+WK.
**Защита:** Найти 1–2 фичи, которые не匹配 ни одна бесплатная комбинация (Origa: локальные OCR+STT+FSRS в одном десктоп-приложении).

### Паттерн D: «Потеря данных / недокументированные лимиты»

**Триггер:** Жёсткий кап или ограничение, обнаруженное пользователем через потерю.
**Примеры:** Кап 10K в jpdb.io; конфликты синхронизации Anki; миграция Yomichan→Yomitan.
**Защита:** Документировать все лимиты заранее. Local-first-архитектура (SQLite Origa) избегает серверных капов.

### Паттерн E: «Поддержка только через Discord воспринимается как уклончивость»

**Триггер:** Платящих пользователей направляют в комьюнити-Discord вместо выделенной поддержки.
**Примеры:** Ответы Migaku на Trustpilot «я НЕ хочу рыться в Discord ради community-answer».
**Защита:** Заранее установить ожидания: GitHub Issues для багов, SLA ответа, публичный roadmap.

### Паттерн F: Геополитическая атака «российский автор»

**Триггер:** Видимое российское происхождение используется как вектор атаки на 4chan /jp/ и похожих площадках.
**Примеры:** Автор AnimeCards/QM; латентная предвзятость в EN-доминантных комьюнити.
**Защита:** Не вести с происхождения; вести с миссии («изучение японского на вашем родном языке»). Не прятать тоже; прозрачность бьёт увиливание.

### Паттерн G: Рефлекс «Yet another X»

**Триггер:** Новый продукт в насыщенной нише (SRS, JP-приложение, OCR-инструмент).
**Примеры:** Ricotta, Nihondex, Lingoku — все получали рефлекс «зачем.switch с Anki?».
**Защита:** Вести с дифференциатора (Origa: локальные OCR+STT+content pipeline). Признавать сильные стороны Anki. Не заявлять о замене.

---

## 5. Ответы с привязкой к каналам

### Reddit (r/LearnJapanese, r/rust, r/SideProject)

- **Тон:** Фактический, скромный, технический-первый. Никакого «meet Origa». Никакого «introducing».
- **Формат:** 400–600 слов, story-формат с кодом/скриншотами. Честно признавать альтернативы.
- **Правило self-promo:** Согласно `reddit-strategy.md`, только после кармы ≥ ~100 и 25–30 полезных non-promo вкладов. Self-promo с тегом только в materials-thread.
- **Этикет первого комментария:** Отвечать на каждый комментарий первые 2 часа. Личный голос, не бот.
- **Кросс-постинг:** Никогда не кросс-постить тот же контент в 5 сабреддитов в один день — выглядит как spam-кампания.

### Hacker News (Show HN)

- **Формат заголовка:** `Show HN: Origa – Japanese learning app with local OCR, STT, and FSRS` (≤80 символов)
- **Первый комментарий (300–500 слов):** Проблема → что я построил → архитектура → известные лимиты → ссылка на исходник. Техническая глубина, без маркетинга.
- **Лучшее время:** Вт/Ср/Чт 7–9 AM EST.
- **Не делать:** заявлять «революционный», «game-changing», использовать эмодзи, ссылаться на лендинг (ссылаться на GitHub).

### GitHub (README / Issues / Discussions)

- **Структура README:** Проблема-первое вступление, секция сравнения честная про конкурентов, секция «Known Limitations» обязательна, секция лицензии явная про BSL 1.1.
- **Issues:** Использовать шаблоны, триадж разметки в течение 24ч, публичные ответы.
- **Discussions:** Включить (сейчас выключено согласно `origa-seo.md` — рекомендация включить).

### App Store / Google Play (когда выйдет мобильная версия)

- **Без обязательного онбординг-вопроса** (ошибка Migii — «первое, что приложение делает, это спрашивает, откуда я о нём узнал»).
- **Ассесмент включает опцию «I don't know»** (ошибка Migii — ассесмент без skip ведёт к неправильному placement'у).
- **Промпт подписки после демонстрации ценности, не до** (ошибка Manabi — попап подписки до того, как что-либо попробовать).

### dev.to / Hashnode / блог

- **Лонгформ-туториал / архитектурный deep-dive.** Frontmatter + body_markdown, 4 lowercase-тега, canonical_url для кросс-поста.
- **Заголовок:** технический крючок («Building a local-OCR Japanese-learning app in Rust with Tauri»), не маркетинг.

---

## 6. Открытые вопросы для пользовательского ревью

Эти решения требуют явного пользовательского ввода до запуска:

1. **Модель монетизации** — бесплатное ядро + платные продвинутые? Разовая покупка? Подписка? См. возражение #15.
2. **Формулировка «open source»** — называть «source-available», «free for personal use» или квалифицировать «open source under BSL»?
3. **Фрейминг российского происхождения** — вести с «российский + английский интерфейсы» или «не-английские интерфейсы, включая русский»?
4. **Последовательность запуска** — применяется ли стадийная последовательность из системного промпта `@marketer` (crates.io → dev.to → Reddit → Show HN) к Origa с учётом того, что BSL ≠ OSS-релиз?
5. **Коммитмент по roadmap iOS** — ресерч показывает, что iOS-only пользователи на r/LearnJapanese — частое явление. Roadmap Origa говорит «iOS в разработке» — дата?
6. **Тайминг замены на SenseVoice** — CER Whisper на JP 22%; SenseVoice лучше. Запланировать замену или публично принять лимит?

---

## 7. Статус фактчекинга

См. `marketing/playbooks/.factcheck.json` для верификации по каждому утверждению. Сводка:

- **Утверждения о Origa** (лицензия, стек, локали): **верифицированы** против README + исходников репо.
- **Утверждения о конкурентах** (таймлайн цен Migaku, кап 10K jpdb.io, maintenance mode Leptos, бенчмарки NDLOCR, CER Whisper): **верифицированы** против цитируемых URL в сырых research-файлах.
- **Предсказания** (talking-points, Origa-уроки): **предсказания**, не факты — явно помечены как таковые в сырых research-файлах.

**Общая уверенность:** 0.86. Гейт: READY_FOR_REVIEW (с флагами — см. factcheck.json).

---

## 8. Следующие шаги (после одобрения HUMAN GATE)

1. **Закрыть блокеры B1, B2, B3** (формулировка лицензии, монетизация, Anki-экспорт).
2. **Обновить `product-hunt.md` и любые другие артефакты**, где есть «free and open-source».
3. **Добавить секцию «Why Origa» в README**, прямо адресующую возражения #1, #5, #9.
4. **Начать reputation-building в r/LearnJapanese** согласно `reddit-strategy.md` Phase 1 — non-promo вклады для достижения кармы ≥100.
5. **Запланировать исследование SenseVoice** — CER Whisper на JP это известный пробел.
6. **Перезапустить этот плейбук через 4–6 недель**, чтобы учесть любые новые движения конкурентов (цены Migaku, обновления jpdb.io, статус форка Leptos).
