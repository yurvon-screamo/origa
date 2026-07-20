# Молодые EdTech JP-стартапы — Траектории и sentiment-ресерч

> **Источник:** Делегированный ресерч через `@tool-accessor` (Tavily + WaniKani Community + Trustpilot + App Store)
> **Дата:** 2026-07-18
> **Статус:** Сырой research-ввод — не фактчекился как самостоятельный
> **Ограничение канала:** Reddit `.json` эндпоинты НЕ использовались

## Методология

- **Дата:** 2026-07-18
- **Источники:** Web search (Tavily/exa), прямые URL-запросы Reddit-тредов, форумы WaniKani Community, Trustpilot, отзывы в App Store/Google Play, сайты продуктов, обзоры в блогах, GitHub-репозитории, профили компаний на Tracxn
- **Проанализировано тредов/страниц:** ~45 уникальных источников
- **Что НЕ нашлось:**
  - **Renji (renji.app)** как самостоятельное JP-приложение — такого продукта не существует. «Renji» в JP-learning относится к **Renji-XD**, автору инструмента texthooker UI
  - **Tenshi / Tenshi Japanese** — продукт не найден
  - Reddit JSON эндпоинты (мертвы с мая 2026)
  - Конкретная страница Migaku на Kickstarter — происхождение Migaku от бесплатных аддонов Anki (MIA), а не Kickstarter

---

## 1. Renji (Renji-XD)

### 1.1 История запуска и основатель

**Renji-XD — НЕ JP-приложение для обучения.** Renji — псевдоним community-разработчика, который создал опенсорсную **веб-страницу texthooker UI** — браузерный интерфейс для Textractor (инструмента извлечения текста из игр).

- GitHub: `Renji-XD/texthooker-ui` — open source
- Инструмент лёгкий, работает локально, часто бандлится в другое ПО, например **Game Sentence Miner (GSM)**
- Нет компании, нет монетизации, нет «запуска приложения» — чисто community-инструмент

> "Renji's texthooker is open source it is often bundled into other software like Game Sentence Miner which has added a few specific features." — skerritt.blog, «Best Japanese Learning Tools 2025 Award Show» (2025-12-08)
> URL: <https://skerritt.blog/best-japanese-learning-tools-2025-award-show/>

### 1.2 Похвала
>
> "Many people use Renji's because it's lightweight and has many settings to allow you to configure things." — skerritt.blog (2025-12-08)

### 1.3 Критика
>
> "Maybe too many settings." — skerritt.blog (2025-12-08)

### 1.4 Текущее состояние

Активен и поддерживается. Используется как компонент в Game Sentence Miner (GSM), последний релиз которого — v2026.5.7 от 22 мая 2026.

### 1.5 Параллели с Origa

**Минимум прямых параллелей.** Авторы community-инструментов, остающиеся нишевыми и опенсорсными, избегают бэклаша, с которым сталкиваются коммерческие «all-in-one» продукты. Функции texthooking/OCR Origa занимают похожее функциональное пространство, но в коммерческом продукте — позиционирование имеет значение.

---

## 2. Tenshi / Tenshi Japanese

**НЕ НАЙДЕНО.** Продукта, приложения, сайта или обсуждения в комьюнити не найдено.

---

## 3. Migaku (самое важное — глубокий разбор)

### 3.1 Запуск и основатель

**Происхождение как MIA (Mass Immersion Approach) Anki Add-ons (2017–2019):**
Migaku произошёл от набора **бесплатных опенсорсных аддонов Anki**, разработанных **Yoga (Lucas)**, участником комьюнити иммерсионного изучения японского. Первый репозиторий аддона создан **10 августа 2019** (`migaku-official/Migaku-Japanese-Addon`).

> "Migaku originated as a suite of Anki add-ons developed by Yoga (Lucas), a member of the Japanese immersion learning community." — Mikey Does, глоссарий Migaku (2026-04-04)
> URL: <https://mikeydoes.com/glossary/migaku/>

**Переход к самостоятельному продукту (2020–2021):**
Около 2020 бренд MIA был свёрнут. Методология/комьюнити перезапустились как **Refold** (языко-агностик), тогда как инструментарий продолжился как **Migaku**, отдельная коммерческая компания. Профиль на Tracxn перечисляет основателей: **Steven Schuttler, Bent Fornalczyk и Lucas**, базируются в США, основана в 2014 (хотя продукт в нынешнем виде датируется ~2020-2021).

**Текущее состояние:** Chrome-расширение + мобильное приложение, поддержка 30+ языков, подписочная модель ($9-20/мес стандартно, lifetime ~$399). Команда ~15 человек. Discord-комьюнити 20,000+.

### 3.2 Таймлайн восприятия на Reddit

**Фаза 1: Любимчик комьюнити (2017–2020)** — бесплатные аддоны Anki, повсеместно хвалят:
> "best addon ever made for learning Japanese" — отзыв в AnkiWeb о Migaku Japanese Addon (2021)
> URL: <https://ankiweb.net/shared/info/278530045>

**Фаза 2: Драма и переход (2020–2022)** — ребрендинг MIA в Migaku, коммерциализация, контроверсия Matt vs Japan:
> "I have been consistently disappointed with this addon since that time. It started with his intrusive use of the addon as a platform to get his point of view out by making a popup with links to videos about the drama." — отзыв в AnkiWeb (2021)

> "the project seems to be unmaintained with the last merged pull request being April 2021. That is nearly a year without a release." — отзыв в AnkiWeb (2021)

**Фаза 3: Восприятие коммерческого продукта (2023–2025):**
> "Is Migaku misleading users? [...] ございません is the polite form of あります, but it was broken into ござい and ません and wrong prononciation of the Kanji" — WaniKani Community (2025-01-24)
> URL: <https://community.wanikani.com/t/is-migaku-misleading-users/69129>

> "the price is really high (400$). It was 200$, less than a year ago" — WaniKani Community (2025-01-24)

> "They also announced that it will become 500$ this end of Feb 2025" — WaniKani Community (2025-01-24)

> "Migaku has now raised its prices by 25% during my 10-day trial" — Reddit r/LearnJapanese (2025-07-18)
> URL: <https://www.reddit.com/r/LearnJapanese/comments/1imwsgn/is_migaku_worth_the_money/>

**Фаза 4: Текущие смешанные сентименты (2025–2026):**
Положительные:
> "By far the most effective system I have found. [...] the convenience of having everything in one extremely simple to use app that syncs across all your devices is worth the price alone." — отзыв в App Store (2026)
> URL: <https://apps.apple.com/us/app/migaku-really-learn-languages/id1664096855>

> "You can get a lifetime license for $200 on sale, it's incredible for the price." — отзыв в App Store (2026)

Негативные:
> "A REAL disappointment. Customer 'service' has stopped responding, there are bugs that are being 'actively investigated' and never fixed [...] I have also noticed that they are now teaming up with a lot of content creators who are flogging subscriptions; I can only guess they're trying to hail-mary it at this point." — отзыв на Trustpilot от Ric (2025-06-16)
> URL: <https://ie.trustpilot.com/review/migaku.com>

> "I paid €250 for the Lifetime version and I truly regret it. The app is full of bugs and feels unreliable." — отзыв на Trustpilot от Souhail (2025-06-04)

> "Migaku on the other hand has a very 'lax' approach it seems. Basically a 'answer the card 3 times correct in a row and you are done'. Of course its quicker but I'm not sure if its effective" — Reddit r/LearnJapanese (2025-07-18)
> URL: <https://www.reddit.com/r/LearnJapanese/comments/1gcuq4d/migaku_vs_anki/>

### 3.3 Драма вокруг vaporware

Обвинения в «vaporware» в адрес Migaku следуют чёткому паттерну:

**1. Заброшенные аддоны во время коммерческого пивота:**
> "the project seems to be unmaintained with the last merged pull request being April 2021. That is nearly a year without a release." — отзыв в AnkiWeb (2021)

**2. Проблемы качества парсинга подтачивают доверие:**
> "I tried it once a few weeks ago, loaded up one of the videos they suggested, and found an error within 2 minutes." — WaniKani Community (2025-01-24)

**3. Агрессивные изменения цен:**
> "200$ less than a year ago without any discount [...] They also announced that it will become 500$ this end of Feb 2025" — WaniKani Community (2025-01-24)

**4. Снижение качества поддержки:**
> "I paid for a subscription, I do NOT want to rummage through Discord for a 'community answer' — what is this — a software development gig??" — отзыв на Trustpilot от Ric (2025-06-16)

**5. Сравнение с бесплатными альтернативами подтачивает ценность:**
> "overall this seems like a classic example of 'should I use this new all in one, nice easy UX, modernized tool that costs money or should I use several separate longstanding programs that can still work together with older but free solutions'" — Reddit r/LearnJapanese (2025-07-18)

### 3.4 Текущие сентименты (2026)

**Поляризованы.** У Migaku есть преданная фан-база (20K+ Discord, активный YouTube-канал, регулярные обновления), но и громкие критики.

> "Several Migaku users in the immersion community have reported recurring friction points around subtitles and AI-assisted features in 2026. Some context-dependent kanji readings come back with the wrong reading attached, the AI image-generation feature occasionally renders the kanji shape rather than the concept the word represents, and YouTube subtitle generation has been flagged as inconsistent on a meaningful share of videos." — immit.co, Migaku Review 2026 (2026-05-08)
> URL: <https://immit.co/blog/migaku-review-2026-is-it-worth-it-for-japanese-learners>

**Итоговая оценка:** Migaku **жив и растёт** (расширение команды до 15+, регулярные обновления, партнёрства с контент-мейкерами), но страдает от дефицита доверия со стороны ранних адоптеров, помнящих эпоху бесплатных аддонов и воспринимающих снижение качества/рост цен.

### 3.5 Стиль ответов команды

**Профессиональный, но шаблонный.** Ответы команды Migaku следуют консистентному паттерну:

> "Hi Ric, sorry to hear about your experience. We do have dedicated support staff who respond to enquiries, and many members of our developer team are active on our community Discord where we triage issue reports [...] Sometimes we miss the mark, and a powerful product suite like ours which is on the bleeding edge can be prone to issues. However we are diligently improving things [...]" — ответ команды Migaku на Trustpilot (2025-06-16)

> "Hello Souhail! Sometime our integration with third-party websites such as YouTube and Netflix can be broken when they make changes on their end. We actively monitor this and prioritise fixing those issues when they come up, with patches usually released within a few days." — ответ команды Migaku на Trustpilot (2025-06-04)

**Паттерн:** Признать → объяснить (часто обвиняя третьи стороны) → отправить в Discord → упомянуть free trial/money-back. Фрейминг «bleeding edge» обоюдоострый — он извиняет баги, но также сигнализирует о нестабильности.

### 3.6 Параллели с Origa — «ловушка all-in-one»

**Самая критичная секция для Origa.** Migaku — ближайший аналог позиционированию Origa: оба — «all-in-one» инструменты для JP-обучения. Траектория Migaku выявляет несколько ловушек:

1. **«All-in-one» = «мастер на все руки, ничего толком»** — Пользователи постоянно сравнивают отдельные фичи Migaku со специализированными бесплатными альтернативами (Yomitan для lookups, Anki для SRS, asbplayer для субтитров) и находят Migaku хуже в каждой категории.

> "much of its functionality can be replicated with yomitan and asb-player" — Reddit r/LearnJapanese (2025-07-18)

1. **Чувствительность к цене экстремальна** — Изучающие JP привыкли к бесплатным инструментам. Lifetime $200-500 / $9-20/мес у Migaku воспринимается как дорогое, когда Yomitan+Anki+asbplayer бесплатны.

2. **Толерантность к багам НИЗКАЯ для платных продуктов** — Пользователи прощают баги в бесплатных инструментах, но не в продуктах, за которые заплатили $250+.

3. **Feature creep размывает качество** — Поддержка 30+ языков и нескольких платформ распыляет команду. Качество JP-specific страдает.

4. **Обвинения в «vaporware» — дефолт для all-in-one приложений** — Любая задержка или баг запускает нарратив «они обещали X, а сделали Y».

---

## 4. jpdb.io

### 4.1 Запуск и основатель

jpdb.io — **веб-SRS для лексики и кандзи**, созданный разработчиком, известным в комьюнити как **«Kou»** (или «Stephan» в некоторых ссылках). Сайт запущен в бете и непрерывно разрабатывается как минимум с **2021**. Соло-проект.

> "the developer of the website (who we call Kou)" — WaniKani Community (2022-08-29)
> URL: <https://community.wanikani.com/t/getting-started-with-jpbd/58352>

Сайт **не зарегистрирован как компания** — нет профиля на Tracxn, нет на Crunchbase, нет раундов финансирования. Персональный проект с поддержкой на Patreon.

### 4.2 Похвала

**Content-aware SRS:**
> "jpdb allows you to create a deck from the text [...] Paste in text, and jpdb extracts the vocabulary. It then generates automatic i+1 sentence cards that show new words in context with the known words around them, drawing on a database it describes as over 130 million Japanese sentences." — J-Compass, гайд по Sentence Mining

> "I've been a jpdb user since March; it's completely replaced Anki and mostly replaced WK as my main SRS. It's, as you point out, a powerful tool with a lot of settings, but its default way of doing everything is much better than other SRSs that I've tried." — WaniKani Community (2022-08-29)

**Готовые колоды из реальных медиа:**
> "jpdb offers tens of thousands of prebuilt decks covering vocabulary from over a thousand anime. It also offers decks for visual novels, light novels, and web novels, presented in difficulty-ordered lists." — J-Compass

**Patreon-фичи ценят:**
> "I'm a Patreon user and I'd say it's absolutely worth it! If you utilise everything jpdb has to offer, it is a very powerful tool for efficiently learning words." — WaniKani Community (2022-08-29)

### 4.3 Критика

**Слабый поиск/дискавери:**
> "it can be pretty hard to search jpdb [...] To pick one obvious example, there's no way to search by author [...] there have been no new additions since last June: no 'people who added deck X also added these decks', no rating of books" — WaniKani Community (2024-02-15)
> URL: <https://community.wanikani.com/t/list-of-jpdb-prebuilt-decks/64890>

**Недокументированные лимиты — ИНЦИДЕНТ С ПОТЕРЕЙ ДАННЫХ:**
> "There's a 10k cap on decks on JPDB [...] why no info in the FAQ about 10k cap? I never knew there was a cap on decks" — WaniKani Community (2024-12-25)
> URL: <https://community.wanikani.com/t/recent-blunder-on-jpdb-theres-a-10k-cap-on-decks/68793>

> "given the dev isn't really working on it I don't think docs improvements are very likely" — WaniKani Community (2024-12-25)

**Медленная разработка / риск соло-мейнтейнера:**
> "there haven't been any new additions since last June"
> "I've asked for the export API for lists years and it's still hidden somewhere; I've basically given up."

**Качество изучения кандзи:**
> "the JPDB kanji learning support is janky anyway" — WaniKani Community (2024-07-27)

### 4.4 Текущее состояние

**Активен, но медленный темп.** jpdb.io продолжает получать обновления (записи в changelog до 2025), новые медиа-добавления, эксперимент с плагином mpv. Сайт остаётся в бете. Нет мобильного приложения. Нет юрлица. Соло-разработчик с финансированием на Patreon.

### 4.5 Параллели с Origa

**Сильные параллели.** jpdb.io доказывает, что есть рынок для **content-aware SRS** — изучения лексики, привязанной к конкретным медиа. Phrase/audio SRS Origa с контент-майнингом занимает похожее пространство. Ключевые уроки:

- **Сила jpdb в специализации** (vocab/kanji SRS из медиа)
- **Слабость jpdb в охвате** — нет мобильного приложения, нет грамматики, нет упражнений на чтение, нет OCR
- **Риск соло-разработчика реален**

---

## 5. Kitsun / Torii / MaruMori

### 5.1 Kitsun.io

**Запуск:** Декабрь 2019 соло-разработчиком (Devloop B.V., Франция). Начат как открытая бета, развился в платную SRS-платформу.

> "It's been a long time coming, but we're finally here! Kitsun has officially launched today." — анонс в Kitsun Community (2019-12-01)
> URL: <https://community.kitsun.io/t/kitsun-has-officially-launched/4983>

**Концепция:** «Anki встречает WaniKani» — гибкая SRS-платформа флешкарт, специально заточенная под японский.

**Ценообразование:** $8/мес, $64.99/год или $199.99 lifetime. 14-дневный бесплатный триал.

**Восприятие — Положительное:**
> "Kitsun has nailed all the features I wanted out of a flashcard program and offers so much more on top of that."
> "Another 1+ for kitsun.io, by far my most useful Japanese study tool. Goes beyond just vocabulary study [...] you have full access to design templates much like Anki within their platform" — WaniKani Community (2025-03-31)

**Восприятие — Негативное:**
> "Afaik my mining setup wouldn't even work on kitsun and I don't think it offers anything I want that Anki doesn't, so not only is it not worth the 6 dollars but I probably wouldn't even use it if you paid me." — WaniKani Community (2021-12-11)

**Текущее состояние:** Активен. Домен зарегистрирован до 2027. Тот же разработчик построил и MaruMori. У Kitsun есть мобильные приложения на Android/iOS.

### 5.2 Torii SRS

**Запуск:** ~2018, **Rakantor** (соло-разработчик, Вена, Австрия). Основана как нефинансируемая компания (Tracxn).

**Концепция:** Бесплатный японский SRS для лексики, сфокусированный на Core 10K. Система type-your-answer, вдохновлённая WaniKani. Web-приложение, Android, iOS.

**Ценообразование:** Бесплатное ядро с подпиской «Prime» для кастомной лексики.

**Восприятие — Положительное:**
> "I'm a bit of a Torii fan and suggest it to learners any chance I get [...] being totally free." — WaniKani Community (2023-08-11)
> URL: <https://community.wanikani.com/t/anki-core-2k6k-vs-torii-srs-10k-pros-and-cons/62920>

> "having to type it out is me being much more honest with my ability" — WaniKani Community (2023-08-11)

**Восприятие — Сдержанное/приглушённое:** Значимой критики не найдено, но и вирусного роста нет.

**Текущее состояние:** Активен, последняя Android-версия 1.4.9 (2025-11-26). Нишевый продукт с маленькой, но лояльной базой пользователей.

**Почему не «взлетел»:** Слишком узкий охват (только лексика, нет системы кандзи, сопоставимой с WK, нет грамматики, нет чтения).

### 5.3 MaruMori

**Запуск:** Декабрь 2022 тем же разработчиком, что и Kitsun.

> "We are proud and very excited to announce the launch of our new Japanese language learning platform, MaruMori. [...] We wanted to eliminate the need to use multiple platforms. So we created an all-in-one, guided, gamified Japanese learning experience." — Kitsun Community (2022-12-22)
> URL: <https://community.kitsun.io/t/double-sale-introducing-marumori-io/6907>

**Концепция:** All-in-one платформа для JP-обучения с геймифицированной картой приключений, уроками грамматики (N5→N1), SRS кандзи/лексики, упражнениями на чтение, тренировками. Явно **«NO AI»** — весь контент сделан людьми вручную.

**Восприятие — Положительное:**
> "For years I have been looking for an All-in-One website to learn Japanese and I thought that this was not possible. [...] One thing that stood out to me immediately is that the developer is really invested in this project and takes feedback very seriously." — блог MariaTheMillennial (2023-03-13)
> URL: <https://mariathemillennial.com/2023/03/13/marumori-review-complete-all-in-one-website-for-learning-japanese/>

> "Overall, I think that this is one of the strongest resources out there for learning Japanese." — обновлённый обзор на MariaTheMillennial (2025/2026)

**Текущее состояние:** Активно и растёт. Мобильные приложения на iOS и Android. Добавлены материалы N3 и N2, N1 в разработке. 9008 единиц лексики, 374 пункта грамматики. Roadmap соблюдается последовательно.

---

## 6. Иммерсионная экосистема

### 6.1 Почему растёт

1. **Наследие AJATT:** Методология «All Japanese All The Time» (Khatzumoto, ~2006) создала фундаментальное комьюнити и доктрину.

> "Sentence mining is particularly popular in the Japanese learning community (Anki/Japanese is one of the most active self-study language communities online)." — Mikey Does, глоссарий Sentence Mining (2026-04-04)

1. **Зрелость инструментария:** Yomitan (2687 звёзд на GitHub), AnkiConnect, asbplayer, mpv-скрипты, SubMiner (создан февраль 2026).

> "In r/LearnJapanese, a thorough guide to learning Japanese for free — built entirely around comprehensible input methodology — drew over 1,700 upvotes. The author advocated Yomitan, ASBPlayer for anime with Japanese subtitles, spaced repetition for vocabulary" — Mikey Does (2026-04-07)

1. **Культурный драйвер:** 4 миллиона изучающих японский во всём мире к 2024 (30x рост с 1979).

> "The number of people studying Japanese overseas was about 130,000 in 1979, but by 2024 it has grown to around 4 million." — nihongonana.com (2026-03-18)

1. **Коммерческая валидация Migaku:** Успех Migaku (даже с критикой) доказывает, что есть платящий рынок для инструментария погружения.

2. **SubMiner (2026):** Новый опенсорсный инструмент интеграции mpv+Yomitan.

> "Integrates Yomitan with MPV — featuring one click Anki mining, realtime subtitle annotations, and immersion tracking. No browser or texthooker required" — GitHub, ksyasuda/SubMiner (создан 2026-02-09)

### 6.2 Анти-structured-app / анти-«all-in-one» сентимент

**Критично для позиционирования Origa.** Иммерсионное комьюнити имеет сильный байас против структурированных «all-in-one» приложений:

> "The learners who burn out are almost always the ones who tried to skip the native-content step." — блог Migaku, «Learning Japanese in 2026: A Practitioner's Playbook»

> "In r/ajatt, mass immersion approach advocates are the dominant voice and skepticism of explicit grammar study is the norm." — Mikey Does (2026-04-07)

> "Migaku sits inside a wider culture sometimes called AJATT and its many descendants. The community ethos values massive native input, daily SRS review, and aggressive self-reliance. It has also produced burnout, perfectionism, and shame spirals for people who could not keep up." — Wordy.info, Migaku Review (2026-05-15)

**Ключевое напряжение:** Адвокаты «comprehensible input» против адвокатов структурированного изучения. Reddit r/LearnJapanese тяготеет к умеренной позиции (CI + грамматический фундамент), тогда как r/ajatt — CI-доминантный.

**Вывод для Origa:** Структурированный подход FSRS Origa в комбинации с OCR/STT ближе к концу спектра «structured study». Иммерсионная толпа, скорее всего, воспримет Origa как ещё одно «Duolingo-style приложение», если функции контент-майнинга не будут в центре позиционирования.

---

## 7. Кейсы провалов

### 7.1 Переход MIA Anki Add-ons → Migaku

**Что произошло:** Бесплатные аддоны Anki с увлечённым комьюнити → коммерческий продукт с платной подпиской → бэклаш комьюнити из-за заброшенных бесплатных инструментов и агрессивной монетизации.

**Что пошло не так:**

- Бесплатные инструменты заброшены без ясной коммуникации во время пивота
- Интрузивное использование аддона для самопиара во время драмы
- 6-месячные циклы разработки означали долгие паузы между обновлениями
- Пользователи чувствовали «bait-and-switch» с бесплатного на платное

> "Some of these comments people are leaving are ridiculous. He did not change this and convert it into another product [...] Don't install or upgrade this addon, you don't know he won't hold your Anki decks or computer for ransom." — отзыв в AnkiWeb (2021)

**Урок для Origa:** Никогда не бросать бесплатный/дешёвый тир, который построил ваше комьюнити. Лицензия BSL 1.1 у Origa — здесь сильная сторона: open source означает, что комьюнити может форкнуть, если развитие остановится.

### 7.2 Эскалация цен Migaku

**Что произошло:** $200 lifetime → $400 → анонс $500 → рост цен во время триалов пользователей.

**Что пошло не так:**

- Несколько повышений цен за короткий период
- Повышение цен во время активного триала пользователя
- Нет grandfathering для существующих пользователей
- Восприятие «доят ранних адоптеров»

> "Migaku has now raised its prices by 25% during my 10-day trial" — Reddit (2025-07-18)

**Урок для Origa:** Установить монетизацию заранее и держаться её. Если цены должны измениться — grandfathering существующих пользователей. Никогда не менять цены во время триала.

### 7.3 Инцидент с капом 10K в jpdb.io

**Что произошло:** Пользователь достиг 9999 слов в колоде «never forget», обнаружил недокументированный кап 10K, случайно удалил всю колоду, пытаясь переместить карточки, потерял месяцы прогресса.

> "There's a 10k cap on decks on JPDB. With the export script found in the jpdb discord server, it is easy to export vocab from one deck and import them to another one. [...] why no info in the FAQ about 10k cap? I never knew there was a cap on decks" — WaniKani Community (2024-12-25)

**Урок для Origa:** Документировать все лимиты заранее. Никогда не позволять пользователям обнаруживать жёсткие лимиты через потерю данных. Local-first-архитектура Origa (SQLite) избегает серверных капов.

### 7.4 Снижение качества поддержки Migaku

**Что произошло:** Пользователи, сообщающие о багах, направляются в Discord. Письма в поддержку остаются без ответа. Платящие пользователи ожидают профессиональной поддержки, а не self-service комьюнити.

> "I do NOT want to rummage through Discord for a 'community answer' — what is this — a software development gig??" — Trustpilot (2025-06-16)

**Урок для Origa:** Соло/маленькая команда Origa означает ограниченную пропускную способность поддержки. Установить ожидания заранее: GitHub Issues для багов, чёткие SLA ответа, публичный roadmap с тем, над чем работа идёт.

### 7.5 Восприятие «Hail Mary» на Trustpilot

**Что произошло:** Migaku партнёрствовало с контент-креейторами для продвижения подписок. Пользователь интерпретировал это как отчаяние.

> "they are now teaming up with a lot of content creators who are flogging subscriptions; I can only guess they're trying to hail-mary it at this point" — Trustpilot (2025-06-16)

**Урок для Origa:** Партнёрства с креейторами ценны, но должны ощущаться органичными. Сначала запуск на Reddit, построение grassroots-кредититета, затем обращение к креейторам, которые уже используют продукт.

---

## 8. Сквозные паттерны

| Паттерн | Продукты | Риск для Origa |
|---------|----------|------------|
| **«All-in-one» = посредственно во всём** | Migaku, MaruMori | **ВЫСОКИЙ** |
| **Ожидание бесплатных инструментов** | Migaku vs Yomitan+Anki, Kitsun vs Anki, jpdb vs Anki | **ВЫСОКИЙ** |
| **Жизнеспособность соло-разработчика / маленькой команды** | jpdb.io, Torii, Kitsun | **СРЕДНИЙ** |
| **Бэклаш на изменения цен** | Migaku (200→400→500) | **СРЕДНИЙ** |
| **Нетолерантность к багам в платных продуктах** | Migaku | **ВЫСОКИЙ** |
| **Иммерсионная толпа отвергает структурированные приложения** | сентимент r/ajatt против Migaku | **СРЕДНИЙ** |
| **Недокументированные лимиты = потеря данных** | кап 10K jpdb.io | **НИЗКО-СРЕДНИЙ** |
| **Партнёрства с контент-креейторами ощущаются неаутентичными** | Migaku | **НИЗКИЙ** |
| **Обвинения в «vaporware» при медленной разработке** | эпоха аддонов Migaku, паузы jpdb.io | **ВЫСОКИЙ** |
| **Anki — горилла на 800 фунтов** | Каждый SRS-продукт | **ВЫСОКИЙ** |

---

## 9. Уроки для Origa (топ-риски и митигации)

### Риск 1: «Ловушка all-in-one» — параллели с обвинениями Migaku

**Что делать СЕЙЧАС:**

- Найти 1–2 фичи, которые не даёт ни одна бесплатная комбинация (локальные OCR+STT+FSRS в одном десктоп-приложении)
- Строить нарратив запуска вокруг этой уникальной комбинации, а не «all-in-one»
- Документировать бенчмарки производительности (точность OCR vs Capture2Text, точность STT vs онлайн-сервисы)

**Что говорить на запуске:**

- «Origa делает X, Y, Z локально на вашей машине — ни один отдельный инструмент их не комбинирует»
- НЕ «Origa — единственное приложение, нужное вам для изучения японского»
- Акцент на приватности/local-first

### Риск 2: Сравнение с Anki — «очередной SRS»

**Что делать СЕЙЧАС:**

- Гарантировать, что импорт/экспорт колод Anki бесшовный
- Сравнить параметры FSRS с реализацией FSRS в Anki
- Позиционировать SRS Origa как «FSRS с контекстом» — интегрированный с OCR/STT

**Что говорить на запуске:**

- «Origa добавляет смайн-через-OCR и захваченные-через-STT карточки в ваши FSRS-ревью автоматически»
- «Работает рядом с Anki — импорт/экспорт ваших колод»
- НЕ «Origa заменяет Anki»

### Риск 3: Восприятие соло-разработчика / vaporware

**Что делать СЕЙЧАС:**

- Лицензия BSL 1.1 — сильнейшая митигация: пользователи могут форкнуть, если развитие остановится
- Поддерживать публичный, датированный changelog
- Установить реалистичный каденс релизов

**Что говорить на запуске:**

- «Origa — open source (BSL 1.1) — если я перестану поддерживать, комьюнити может продолжить»
- «Вот мой публичный roadmap с датами»
- Показать частоту коммитов и GitHub-активность

### Риск 4: Рассогласование ожиданий от монетизации

**Что делать СЕЙЧАС:**

- Решить модель монетизации до запуска
- Рассмотреть: бесплатное ядро (FSRS + базовая лексика) + платные продвинутые фичи (OCR, STT, премиум-контент)
- Никогда не менять цены во время триала

**Что говорить на запуске:**

- Если щедрый бесплатный тир: «Основные функции бесплатны навсегда. Продвинутые (OCR, STT) — [цена]»
- Если платно: «14-дневный бесплатный триал, без кредитки. Разовая покупка ИЛИ подписка»
- Избегать ошибки Migaku: никакой эскалации lifetime-цен

### Риск 5: Бэклаш запуска на Reddit

**Что делать СЕЙЧАС:**

- Изучить успешные запуски на Reddit (Kotomaji получил положительный приём с прозрачным фреймингом «my wife and I built this»)
- Подготовить детальный, технический пост, показывающий, что Origa делает иначе
- Быть готовым ответить «why not just use Anki + Yomitan + [free tool]?» конкретными ответами

**Что говорить на запуске:**

- «Я построил Origa, потому что хотел [конкретную вещь], которую не даёт ни одна существующая комбинация инструментов»
- Показывать, а не рассказывать: скриншоты/GIF'ы работы OCR на панелях манги, STT транскрибирующее аудио аниме
- Честно признавать альтернативы
- Отвечать на каждый комментарий в треде запуска первые 48 часов

---

## 10. Неподтверждённое / не найдено

| Пункт | Статус |
|------|--------|
| **Renji (renji.app)** | **НЕ НАЙДЕНО как JP-приложение.** Renji = Renji-XD, автор инструмента texthooker |
| **Tenshi / Tenshi Japanese** | **НЕ НАЙДЕНО** |
| **Migaku Kickstarter** | **НЕ НАЙДЕНО** — Migaku произошёл от бесплатных аддонов Anki |
| **Конкретные Reddit-треды запуска** Renji, jpdb.io, Torii, Kitsun | **НЕ НАЙДЕНО** — Reddit JSON эндпоинты мертвы |
| **Финансы/число пользователей Kitsun** | **НЕ НАЙДЕНО** |
| **Число пользователей Torii SRS** | **НЕ НАЙДЕНО** |
| **Финансы MaruMori** | **НЕ НАЙДЕНО** |
| **Реальная личность jpdb.io Stephan/Kou** | **ЧАСТИЧНО НАЙДЕНО** |
| **Молодые JP-приложения, исчезнувшие за 6-12 месяцев** | **НЕ НАЙДЕНО как конкретные кейсы** |
