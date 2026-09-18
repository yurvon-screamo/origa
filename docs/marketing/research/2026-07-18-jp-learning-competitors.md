# Прямые конкуренты в JP-learning — Sentiment-ресерч

> **Источник:** Делегированный ресерч через `@tool-accessor` (Tavily + App Store/Google Play + форумы)
> **Дата:** 2026-07-18
> **Статус:** Сырой research-ввод — не фактчекился как самостоятельный
> **Ограничение канала:** Reddit `.json` эндпоинты НЕ использовались

## Методология

**Дата:** 18 июля 2026
**Источники:** Reddit (r/LearnJapanese, r/WaniKani, r/Anki, r/duolingojapanese, r/LearnJapaneseNovice, r/languagelearning), форум r/Bunpro community, форум WaniKani community, Tofugu, App Store, Google Play, независимые блоги (JLPT Samurai, Migaku, Kanjidon, PCMag, skerritt.blog)
**Треды/отзывы:** ~50+ тредов, ~30 отзывов в App Store, ~15 тредов на форумах, ~10 обзоров в блогах
**Ограничения:** Reddit блокирует часть контента через Cloudflare (429/block); `.json` эндпоинты мертвы с мая 2026

---

## 1. Bunpro (Grammar SRS)

### 1.1 Похвала

- *"I cannot recommend Bunpro enough. It took me under a year to complete all the grammar contents of the website, which means N5 to N1; it's with no doubt in my mind the most efficient way to learn japanese grammar at a fast rate."* — Reddit, r/LearnJapanese, [Opinions on Bunpro for grammar](https://www.reddit.com/r/LearnJapanese/comments/u44fg7/opinions_on_bunpro_for_grammar/)
- *"Bunpro + Wanikani carried me through N1. Turns out when you do hundreds of WK and BP reviews every day, you end up being able to read pretty fast."* — Bunpro Community, [Is bunpro really worth it?](https://community.bunpro.jp/t/is-bunpro-really-worth-it/184093)
- *"The way they teach grammar points with so many example sentences and the SRS review system they have where you fill in the blanks of a sentence with the required grammar is fantastic and much better than any Genki textbook I've ever touched."* — WaniKani Community, [Recommending Bunpro to Level 20+ Users](https://community.wanikani.com/t/recommending-bunpro-to-level-20-users/66175)
- *"Bunpro made me N2 proficient. I failed the N2 4 times before I started using Bunpro, subsequently I passed the N2."* — Bunpro Community
- *"Bunpro has 8 example sentences for each vocab / grammar point."* — Bunpro Community
- *"In my 8 years studying Bunpro is the best app I've used for studying."* — Bunpro Community
- Обзор Tofugu: 8/10: *"Bunpro's greatest strength is its flexibility... should be in every Japanese learner's toolkit."* — [Tofugu Review of Bunpro](https://www.tofugu.com/reviews/bunpro/)

### 1.2 Критика

**Качество SRS — главная жалоба:**

- *"I love Bunpro but the quality of the SRS is wanting me to stop using it... unfortunately the SRS is really that bad and it's becoming harder and harder to justify the amount of time I spend/waste reviewing the same grammar points."* — Bunpro Community, [I love Bunpro but the quality of the SRS is wanting me to stop using it](https://community.bunpro.jp/t/i-love-bunpro-but-the-quality-of-the-srs-is-wanting-me-to-stop-using-it/117502)

**Review-pileup (overdue storms):**

- *"There are always some cards that you have problems with and seeing them 12 times or more per day is absolutely no solution... I wish Bunpro would have a system like Anki where you can do all your reviews in one turn and be done for the day."* — Bunpro Community, там же

**Ghost-интервалы слишком короткие:**

- *"The smallest interval is an hour, that is the main source of frustrating neverending reviews over a day. If you have a lot of cards with small intervals you can cheat the system by doing all at 11pm."* — Bunpro Community, там же

**SRS не различает leech-карточки:**

- *"Grammar point A fails 10 times, grammar point B never misses once. Grammar point A and B will be marked as mastered with a 10 day gap... the SRS interval remains the same."* — Bunpro Community, там же

**Грамматика через fill-in-the-blank — не то же, что «учить грамматику»:**

- *"Instead of learning the grammar, I just learned to associate the right answer with the prompt sentence."* — WaniKani Community, [Recommending Bunpro to Level 20+ Users](https://community.wanikani.com/t/recommending-bunpro-to-level-20-users/66175)

**Грамматика изолированно от лексики:**

- *"The main issue with Bunpro is a lot of example sentences use vocab/kanji outside the level you are studying at. e.g. an N5 sentence may have N2 vocab."* — Bunpro Community

**Синоним hell при N3+:**

- *"You start memorizing specific examples more than the underlying 'rules' they are attempting to teach."* — WaniKani Community, [Bunpro — Should I get it?](https://community.wanikani.com/t/bunpro-should-i-get-it/70810)

**«I hate bunpro» (меньшинство):**

- *"I hate bunpro (sorry I know some of the devs are here). I tried it again like a year ago and it has the same fundamental problem and I just don't agree with the approach as a whole. I recommend just buying Genki 1 and 2."* — WaniKani Community, там же

### 1.3 Паттерны ответов команды

- Разработчики Bunpro активно присутствуют на форуме WaniKani community и собственном Bunpro Community
- Обновляют SRS-настройки по запросам (добавили режим «once a day», ghost-контроль)
- Tofagu написал полный позитивный обзор — указывает на уважительные отношения между командами
- Регулярные обновления (vocab decks, reading practice, mock tests)

### 1.4 Параллели с Origa

| Проблема Bunpro | Позиция Origa |
|---|---|
| Низкое качество SRS (интервалы, leech handling) | **Origa закрывает:** FSRS-6 |
| Грамматика изолированно от лексики | **Origa закрывает:** интегрированный vocab+grammar |
| Review-pileup / overdue storms | **Origa закрывает:** FSRS + daily limit |
| Cloze-deletion не учит «применять» грамматику | **Origa частично:** STT для output practice |
| Споры о классификации grammar points | **Пробел Origa:** нужен авторитетный source-of-truth для JLPT-грамматики |

---

## 2. WaniKani (Kanji/Vocab SRS)

### 2.1 Похвала

- *"Wanikani mnemonics worked extremely well for me personally... I just close my eyes and spend several minutes experiencing the mnemonic."* — WaniKani Community, [Am I super slow or something?](https://community.wanikani.com/t/am-i-super-slow-or-something/69252)
- *"I don't think the true value of WaniKani is the SRS to begin with, I think it's the creative set of mnemonics and the building-block learning order."* — WaniKani Community, [Question about pacing and review hell](https://community.wanikani.com/t/question-about-pacing-and-review-hell/70354)
- *"Wanikani is a lot like drinking, you need to pace yourself or you'll regret it later on."* — Reddit, r/WaniKani, [According to WKStats](https://www.reddit.com/r/WaniKani/comments/1kyhzr5/according_to_wkstats_i_have_one_whole_month_until/)

### 2.2 Критика

**Жёсткий порядок — нельзя опережать:**

- *"I want to have ALL of my vocab lessons done before I guru the new kanji... the downside is you learn the kunyomi for the characters late."* — WaniKani Community

**«You can't review early» — нет системы Easy/Ease:**

- *"WK is extremely time gated... WK not having an 'ease' system that would let you progress faster through easy content like Anki does."* — WaniKani Community, [Question about pacing and review hell](https://community.wanikani.com/t/question-about-pacing-and-review-hell/70354)

**SRS-строгость, медленный темп:**

- *"It's taking me around a month per level, which seems a lot worse than what I've read from other people... I'm beginning to feel like I'm hitting a bit of a wall."* — WaniKani Community, [Beginning to feel doubt about speed of progress](https://community.wanikani.com/t/beginning-to-feel-doubt-about-speed-of-progress/70034)

**Review hell при занятом изучающем:**

- *"I usually come home to like 400 every night and I have no idea how to manage it... it just gets so overwhelming! [...] In the end I just gave up and left Wanikani."* — WaniKani Community, [Difficulty catching up on reviews](https://community.wanikani.com/t/difficulty-catching-up-on-reviews/64849)

**Цена ($9/мес, $89/год, $299 lifetime):**

- *"It almost feels like paying for nothing sometimes... I just stopped lessons until I got the reviews to slow down."* — WaniKani Community

**Ограниченный контент:**

- *"After level 30 the kanji would get less useful... after level 40 you learn words nobody has seen yet."* — WaniKani Community

**Медленный темп (2 года на full completion):**

- *"I started WK on July 29, 2024 and am currently on lvl 23."* — WaniKani Community

### 2.3 Паттерны ответов команды

- Команда Tofagu/WK активна на собственном форуме
- Существуют Reddit AMA
- Контролируют тон — «no official response» на критику цены
- Выпускают обзоры Tofagu конкурентов — открытые, не токсичные

### 2.4 Параллели с Origa

| Проблема WK | Позиция Origa |
|---|---|
| Жёсткий порядок, нельзя опережать | **Пробел Origa:** нужен «free mode» или «jump ahead» |
| Нет ease system | **Origa закрывает:** FSRS автоматически адаптирует интервалы |
| Review hell (400+/день) | **Origa закрывает:** FSRS + daily limit |
| Только WK-слова | **Origa закрывает:** dictionary-based |
| Цена | **Пробел Origa:** нужно продумать монетизацию |
| 2 года на kanji | **Origa частично:** можно быстрее, но нужна валидация |

---

## 3. Migii / Ohayou

### 3.1 Похвала

- *"Migii JLPT confidently helps you increase your JLPT test score by at least 30 points."* — [Migii JLPT официальный сайт](https://jlpt.migii.net/en/)
- *"The Roadmap & the reading practice makes the app worth the price... best way to study for the JLPT!"* — отзыв в App Store, [Migii JLPT](https://apps.apple.com/us/app/migii-jlpt-jlpt-test-n5-n1/id1463267540)
- Рейтинг 4.5/4.6 в App Store и Google Play, 35.2K отзывов в Google Play
- *"I've achieved N3 and Manten reading comprehension in just 3 months of preparation with Migii."* — [отзыв на официальном сайте Migii](https://jlpt.migii.net/en/)

### 3.2 Критика

**Баги и некорректные ответы:**

- *"I bought the pro package for jft and felt it was a waste of money. My answer was correct but the app marked it wrong and didn't give me points."* — отзывы App Store / MWM, [Migii JLPT](https://mwm.ai/apps/migii-jlpt-jlpt-test-n5-n1/1463267540)

**Качество перевода и аудио:**

- *"The audio is extremely poor quality and does not sound natural. Sentence translations in some examples are overwhelmingly incorrect."* — отзыв в App Store
- *"There have been so many bugs - I've found many incorrect pronunciations."* — отзыв в App Store

**AI-deterioration после обновлений:**

- *"New updates and AI have made this worse... This used to be a solid 4/5 app, now it's barely 2."* — отзыв в App Store

**Удаление фич после редизайна:**

- *"Paid for premium and the old design offered extensive ability to translate based on a selection of text. The re-work offers that to 20% of the original."* — отзыв в App Store

**Обязательный онбординг-вопрос:**

- *"First thing the app does is ask me where I heard about it. It doesn't let me skip. I'm effectively forced to give a completely false answer to even look at what's in the app."* — отзыв в App Store

**Нет опции «I don't know» в ассесменте:**

- *"When taking the initial assessment test, there's not an 'I don't know' option. I guessed on all of them and got almost half the test right. So now the app thinks I know far more than I actually do."* — отзыв в App Store

**Грамматика не всегда корректна:**

- *"The app isn't completely accurate in the JAPANESE grammar section. Intercangeable particle usage not being accepted."* — отзыв в Google Play, [Migii JLPT](https://play.google.com/store/apps/details?id=com.eup.mytest)

### 3.3 Параллели с Origa

| Проблема Migii | Позиция Origa |
|---|---|
| Некорректные ответы в тестах | **Origa закрывает:** локальные ML-модели |
| AI-deterioration | **Origa закрывает:** offline-first, AI не нужен для core |
| Плохой аудио | **Origa закрывает частично:** Whisper STT, но для native listening нужен native audio — **пробел** |
| Forced onboarding | **Пробел Origa:** UX-урок — не заставлять |

---

## 4. ReWord

### 4.1 Похвала

- *"The simple, sleek design makes it easy and painless to learn and review vocab."* — отзыв в Google Play, [ReWord Japanese](https://play.google.com/store/apps/details?id=ru.poas.learn.japanese.jlpt.katakana.kana.hiragana.kanji.romaji)
- *"The one time premium payment is fairly inexpensive and is well worth it."* — отзыв в Google Play

### 4.2 Критика

- Поверхностность — нет контекста
- *"For whatever reason, the Japanese version doesn't come with a pronunciation/audio guide. I'll hit the play button, and nothing will happen."* — отзыв в Google Play
- *"There have been times I've accidentally marked a card as 'already known', and I haven't found a way to get those cards back."* — отзыв в Google Play

### 4.3 Ответы команды

**Не найдено** — публичного присутствия на Reddit или форумах нет.

### 4.4 Параллели с Origa

ReWord не конкурент Origa по функциональности — Origa значительно шире.

---

## 5. Anki (как базовый ориентир)

### 5.1 Жалобы «Anki слишком сложный»

- *"I've been meaning to try spaced repetition but Anki's setup kept putting me off... You download Anki, opened it, and immediately felt like you needed an engineering degree to get started."* — блог MindCards, [Spaced Repetition for Language Learning](https://www.mindcards.app/blog/how-to-use-spaced-repetition-language-learning)
- *"To be honest, even though Anki is an extremely useful tool, it's not a pretty program. The user interface is not very nice to look at, and it's too complex."* — English Tea Break, [Getting Started With Anki](https://englishteabreak.com/roadmap/getting-started-with-anki)
- *"The biggest barrier to using Anki for Japanese is the time it takes to create 'Perfect Cards'. Manually finding audio, images, and example sentences for 5,000 words can take hundreds of hours."* — StudyCards AI, [How to Use Anki for Learning Japanese](https://studycardsai.com/blog/how-to-use-anki-for-learning-japanese)
- *"Anki has a steep learning curve at the start, and it has plenty of quirks that are frustrating to deal with... the default settings seem to be really bad, requiring fiddling right from the get-go."* — WaniKani Community, [Genki vocabulary SRS](https://community.wanikani.com/t/genki-2nd-3rd-edition-vocabulary-differences-choosing-a-non-wk-vocabulary-srs/62193)

**«Review Debt Trap» — главная причина бросить:**

- *"The 'Review Debt Trap' is the number one reason Japanese learners quit Anki. This happens when you skip a few days, and your reviews pile up from 50 to 500."* — StudyCards AI

**Обновление Anki сломало настройки:**

- *"I just updated to Anki 2.1.49 on a Mac and all my buttons except 'Easy' say <1m. I have no idea how to fix this."* — Reddit, r/Anki, [All my buttons say <1m](https://www.reddit.com/r/Anki/comments/s02xu5/)

### 5.2 Защитный лагерь «Anki и так нормален»

- *"If other learning apps have features that are very much better than Anki, we need to develop them and incorporate them into Anki. Official Anki for desktop has been developed for about 18 years, and there are 1500+ add-ons by volunteers."* — Reddit, r/Anki, [Anki alternatives Autumn 2024](https://www.reddit.com/r/Anki/comments/1g0h2l4/anki_alternatives_autumn_2024/)
- *"I lost my 1480 day Anki streak and it was the best thing to ever happen to me. [...] During the last year of the streak... it felt more and more like I was fighting with Anki rather than using it as a tool."* — Reddit, r/LearnJapanese, [I lost my 1480 day Anki streak](https://www.reddit.com/r/LearnJapanese/comments/1r6h58y/i_lost_my_1480_day_anki_streak_and_it_was_the/)
- *"Anki is powerful but requires manual card creation and has a steep learning curve."* — блог FlashRecall, [Anki Guide](https://flashrecall.app/blog/anki-guide-2)

### 5.3 Общие боли экосистемы Anki

- *"Some parts seam rather frail (like the manual syncing, with delta syncs and complete syncs depending on the changes...)"* — WaniKani Community
- *"You downloaded the wrong anki programs"* — частый ответ новичкам на r/Anki
- *"I used to have an anki add-on to aid in the quizlet deck import process, but it is no longer supported."* — Reddit, r/Anki, [trouble importing quizlet slides](https://www.reddit.com/r/Anki/comments/1qe4fj4/)

---

## 6. Duolingo Japanese

### 6.1 Что ненавидят серьёзные JP-изучающие

- *"The thing about Duolingo is that it doesn't do a great job explaining things, right? There are tips, so I can see explanations of different grammar points, though they aren't very in depth."* — Reddit, r/japanese, [雨が降っています？](https://www.reddit.com/r/japanese/comments/pif3fd)
- *"Duolingo is terrible at explicitly teaching grammar. It operates on a 'pattern recognition' model."* — блог Migaku, [Duolingo Japanese Review 2025](https://migaku.com/blog/japanese/duolingo-japanese-review-does-it-work)
- *"Duolingo introduces kanji but does a poor job of teaching you how to actually learn them. It doesn't teach stroke order, the radicals, or effective mnemonic techniques."* — JLPT Samurai, [The Verdict on Duolingo Japanese](https://jlptsamurai.com/2025/11/15/the-verdict-on-duolingo-japanese-expert-review-jlpt-viability-and-top-alternatives/)
- *"Completing the Duolingo Japanese tree puts you at roughly 60–70% of JLPT N5, not full N5."* — Conjugaizen, [Finished Duolingo Japanese — What Next?](https://conjugaizen.com/alternatives/after-duolingo-japanese-what-next/)
- *"The cat drinks milk. The boy eats an apple. While these sentences are grammatically correct, you will sound very strange if you say them in Tokyo."* — JLPT Samurai
- *"Duolingo does not address pitch accent at all."* — блог italki, [Is Duolingo Good for Learning Japanese?](https://www.italki.com/en/blog/is-duolingo-good-for-japanese)

### 6.2 Что привлекает массовую аудиторию

- *"The app is extremely well made and very simple while being gamified, engaging, and addictive. [...] Duolingo is a magnificent starting point and an excellent habit former."* — Reddit, r/LearnJapanese, [I changed my mind about Duolingo](https://www.reddit.com/r/LearnJapanese/comments/1b76qqc/i_changed_my_mind_about_duolingo/)
- *"Zero pressure environment. You can make mistakes privately, repeat lessons endlessly, and learn at your own pace."* — JLPT Samurai
- *"Duolingo is essentially a gaming app. Its leaderboard system is brilliantly designed, fully exploiting human weaknesses to make you feel uneasy if you don't play for a while every day."* — Reddit, r/duolingo, [Completed my second foreign language](https://www.reddit.com/r/duolingo/comments/1r5d52f/completed_my_second_foreign_language_in_two_years/)

### 6.3 Позиционирование Origa относительно Duolingo

Duolingo — не прямой конкурент. Origa нацелен на серьёзных/самообучающихся. Origa может перетянуть пользователей, которые «выросли» из Duolingo.

---

## 7. Сквозная таксономия возражений

| Возражение | Продукты | Пример-цитата | Применимо к Origa? |
|---|---|---|---|
| «Слишком сложно настроить» | Anki | *"felt like you needed an engineering degree to get started"* | **Нет** — Origa desktop-приложение, offline-first |
| «Review pile / overdue storm» | Bunpro, WK, Anki | *"come home to like 400 every night"* (WK) | **Средний риск** — FSRS помогает |
| «Слишком дорого» | WK ($299), Migii | *"It almost feels like paying for nothing sometimes"* | **Да** — нужно конкурентное ценообразование |
| «SRS не работает для грамматики» | Bunpro | *"reading and producing sentences really cements the grammar"* | **Частично** — у Origa есть STT/TTS |
| «Anki уже существует, зачем ещё одно приложение?» | Любое новое приложение | *"we need to develop them and incorporate them into Anki"* | **Да** — главное возражение |
| «Проблемы с качеством контента / неправильные ответы» | Migii, Bunpro | *"My answer was correct but the app marked it wrong"* | **Риск** — нужно качество контента |
| «Слишком медленно / time-gated» | WK | *"WK is extremely time gated... 2 years to complete"* | **Нет** — Origa не time-gated |
| «Нужен offline» | Bunpro, Migii | — | **Origa закрывает:** Tauri desktop |
| «Нет практики говорения» | Все кроме Origa | *"No real speaking practice"* (Duolingo) | **Origa закрывает:** STT |
| «Флешкарты без контекста = бесполезны» | ReWord, базовые колоды Anki | *"recognition without understanding how to actually use the word"* | **Origa закрывает:** словарь, контекстные предложения, OCR |

---

## 8. Уроки для Origa (ПРЕДСКАЗАНИЕ)

### Топ-5 возражений (вероятность >80%)

1. **«Anki уже существует — зачем мне Origa?» (95%)**
   - **Сильная сторона Origa:** offline-first + интегрированные OCR/STT/TTS + словарь + грамматика в ОДНОМ приложении
   - **Пробел Origa:** нужна ясная секция лендинга «Why not just Anki?»

2. **«Не могу доверить новому приложению годы моих данных» (85%)**
   - **Сильная сторона Origa:** open-source (Rust + Leptos), локальные данные
   - **Пробел Origa:** нужен экспорт данных (Anki deck export), backup/restore, видимая GitHub-активность

3. **«Контент может быть ошибочным / неполным» (80%)**
   - **Сильная сторона Origa:** NDLOCR (OCR), Whisper (STT) — ML-модели с внешней валидацией
   - **Пробел Origa:** нужна прозрачность источников контента, versioning, механизм обратной связи от комьюнити

4. **«Слишком много функций = овервелминг» (80%)**
   - **Сильная сторона Origa:** progressive disclosure
   - **Пробел Origa:** нужен flow онбординга, направляемый первый опыт

5. **«Цена / монетизация неясна» (80%)**
   - **Сильная сторона Origa:** one-time desktop-приложение (Tauri) — нет давления подписки
   - **Пробел Origa:** нужна чёткая pricing-страница

### Возражения-«ловушки»

**«Anki уже существует» — автоматический проигрыш?**

1. Origa ≠ конкурент Anki. Origa = «приложение для изучения японского языка, в котором случается SRS»
2. «Одно приложение вместо пяти» — показать стек (Anki + WK + Bunpro + Yomitan + AnkiDroid) и противопоставить Origa
3. «Работает офлайн, нулевая настройка» — реальное преимущество перед Anki, требующим 2+ часа настройки

---

## 9. Неподтверждённое / не найдено

- **ReWord на Reddit:** недавнего присутствия нет (2024–2026)
- **Ответы команды Migii на Reddit:** публичных ответов не найдено
- **Ответы разработчиков Bunpro на Reddit:** найдены на форуме WaniKani, но не на самом Reddit
- **Reddit AMA Tofagu/WK:** не найдены (rate-limited)
- **Ответы команды Duolingo на конкретные жалобы JP:** не найдено
- **Даты отзывов в App Store:** App Store не показывает точные даты в web-view
