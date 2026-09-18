# FSRS / SRS-экосистема — Sentiment-ресерч

> **Источник:** Делегированный ресерч через `@tool-accessor` (Tavily `site:reddit.com`)
> **Дата:** 2026-07-18
> **Статус:** Сырой research-ввод — не фактчекился как самостоятельный (claims интегрированы в главный плейбук и фактчекинг там)
> **Ограничение канала:** Reddit `.json` эндпоинты НЕ использовались (мертвы с мая 2026)

## Методология

- **Дата исследования:** 18 июля 2026
- **Источники:** Tavily web search с `site:reddit.com` запросами (Tier 1). PRAW/Playwright — не потребовались.
- **Количество найденных тредов:** ~25 уникальных Reddit-тредов, охватывающих r/Anki, r/LearnJapanese, r/languagelearning, r/Japaneselanguage, r/remNote, r/studytips, r/commandline, r/Slatestarcodex
- **Что НЕ нашлось (явно):** отзывы о Mochi.cards на r/LearnJapanese и r/languagelearning; упоминания Mnemosa на Reddit; RemNote среди JP-изучающих; отзывы в App Store / Google Play; тред r/Anki «To people still using SM2 instead of FSRS: why?» заблокирован network security Reddit

---

## 1. Anki + FSRS

### 1.1 Похвала (с цитатами)

> "Nah, FSRS is the way imo. Lots of hesitation around it, but it's quite superior. Enough so that the transition is more than worth it. If you just tick the box then it just instantly turns on. There isn't really any transition if you don't want there to be. [...] Even without optimising FSRS is better than the old SM2 so the one checkbox is worth hitting even if you do nothing else." — r/Anki, [Are there others who don't use FSRS intentionally?](https://www.reddit.com/r/Anki/comments/1cbf2o3/are_there_others_who_dont_use_fsrs_intentionally/), ~2024

> "FSRS On. Anki v. 25.07 or higher (FSRS-6). Both of the above are non-negotiable musts. Google FSRS if you want to learn more about it. It's good and should be used. Anki 25.07 was the deployment of FSRS-6 which offers significant benefits over previous versions, esp. in the DR<80% region." — u/[unknown], r/LearnJapanese, [The Anki settings I used to improve my efficiency by ~350%](https://www.reddit.com/r/LearnJapanese/comments/1p66osq/the_anki_settings_i_used_to_improve_my_efficiency/), ~2025-12

> "FSRS 6 adds custom 'forgetting' curves to the algorithm. [...] The new version will factor in how you personally forget information, so it will be able to suit each individual user better. [...] the charts floating around are impressive." — r/Anki, [What is changed in v6 of FSRS?](https://www.reddit.com/r/Anki/comments/1lxbmlv/what_is_changed_in_v6_of_fsrs/), ~2025

> "I love Anki, but I really missed the ability to connect my notes. Anki also feels a bit too bloated for many use cases, and while FSRS is great, I wanted something more integrated with my notes [...] FSRS is a perfect system to set best fit due date for you." — r/studytips, [I built a tool for flashcard making with a spaced repetition system embedded in your notes](https://www.reddit.com/r/studytips/comments/1rl7pvz/i_built_a_tool_for_flashcard_making_with_a_spaced/), ~2026

> "FSRS is more accurate for users who only use two buttons (lower RMSE is better). The graph is based on 20 thousand collections." — мейнтейнер FSRS LMSherlock (Jarrett Ye), r/Anki, [FSRS is more accurate if you only use Again and Good](https://www.reddit.com/r/Anki/comments/1d0fmsz/fsrs_is_more_accurate_if_you_only_use_again_and/), 2024-07 (позже отозван автором — дальнейший анализ был неокончательным)

### 1.2 Критика (с цитатами)

**Интервалы слишком длинные:**
> "Recent common complaint about FSRS in the Anki community is that the time interval is too long. The average student Anki user uses cramming for exams, and therefore needs to be sure that they remember the algorithm by the time of the exam, so users do not like long intervals very much." — r/Anki (японский перевод треда), [Do you think FSRS is the best algoritm for spaced repetition?](https://www.reddit.com/r/Anki/comments/1kk0ble/do_you_think_fsrs_is_the_best_algoritm_for_spaced/), ~2025

**Hard button misuse → сломанные интервалы:**
> "Making FSRS the default will be a horrible mistake. It will screw up every person who uses Hard as 'fail', which is at least 10% of all Anki users." — r/Anki, [FSRS will (almost) certainly become the default algorithm](https://www.reddit.com/r/Anki/comments/1h8ss2p/fsrs_will_almost_certainly_become_the_default/), ~2025

> "Remember that the only button you should press if you couldn't recall the answer is 'Again'. 'Hard' is a passing grade, not a failing grade. If you misuse 'Hard', all of your intervals will be excessively long." — цитата autobot/wiki в r/Anki, [What is changed in v6 of FSRS?](https://www.reddit.com/r/Anki/comments/1lxbmlv/what_is_changed_in_v6_of_fsrs/), ~2025

**«Не чувствую, что учусь»:**
> "I've seen a few complaints/comments saying they felt like they weren't even learning anymore with FSRS and switched back. Given my sort of time-constrained situation on wanting to specifically use it for my application aptitude tests, I don't think I really have the luxury of experimenting." — u/[novice], r/Anki, [For a Complete Anki Novice, Yes or No to FSRS Settings?](https://www.reddit.com/r/Anki/comments/1s1y6om/for_a_complete_anki_novice_yes_or_no_to_fsrs/), 2026-04

> "I hate to sounds so gloomy about FSRS, but I just feel so much uncertainty and lack of control/stability with the idea of leaving the learning/relearning steps blank for FSRS. I also no longer see the 'compute minimum recommended retention' option in my latest Anki version." — u/[novice], r/Anki, [For a Complete Anki Novice, Yes or No to FSRS Settings?](https://www.reddit.com/r/Anki/comments/1s1y6om/for_a_complete_anki_novice_yes_or_no_to_fsrs/), 2026-04

**Непонятные параметры и статистика:**
> "At least about the retention rate, I personally think 82% *feels fine* (as in 'I'm not too frustrated yet with how many cards I get wrong') but seeing the statistics show an actually significantly lower rate makes me heavily doubt everything has been going well. [...] it still isn't really clear to me on what basis FSRS learns" — r/Anki, [Switching from 2 Buttons to 4 / FSRS Adjustment Help](https://www.reddit.com/r/Anki/comments/1r5oluo/switching_from_2_buttons_to_4_fsrs_adjustment_help/), ~2026

> "Cards that are easy appear too often / need too long to be pushed into the future. Cards that are kinda difficult take too long to appear again after I got them right a few times." — r/Anki, [Switching from 2 Buttons to 4 / FSRS Adjustment Help](https://www.reddit.com/r/Anki/comments/1r5oluo/switching_from_2_buttons_to_4_fsrs_adjustment_help/), ~2026

**Нужно 1000+ ревьюсов для оптимизации:**
> "I was reading the discussion on Anki's FSRS auto-optimize. Realized that while that issue gets resolved, we could all use a little reminder. So I made an addon! [...] It's a simple pop-up reminder that only triggers when your review history has doubled (1k, 2k, 4k, 8k...)." — u/SiddharthShyniben, r/Anki, [I made an addon to remind you to optimize your presets](https://www.reddit.com/r/Anki/comments/1rv9gy5/i_made_an_addon_to_remind_you_to_optimize_your.json), ~2026

**Устаревший UI Anki / «bloated»:**
> "Anki also feels a bit too bloated for many use cases [...] I was tired of using the Obsidian-to-Anki or just simple Anki." — r/studytips, [I built a tool for flashcard making with a spaced repetition system embedded in your notes](https://www.reddit.com/r/studytips/comments/1rl7pvz/i_built_a_tool_for_flashcard_making_with_a_spaced/), ~2026

### 1.3 Паттерны ответов команды

**LMSherlock / Jarrett Ye (мейнтейнер FSRS — один человек, volunteer):**

- Активно отвечает в тредах r/Anki, включая технические разъяснения (изменения FSRS v6, ограничения same-day review, обсуждения RMSE).
- Прямая цитата: "same-day reviews have a very small impact on long-term memory. Don't waste your time with learning steps like `15m 30m 1h 2h 4h`." — r/Anki, [Anki 24.11: one of the biggest updates ever](https://www.reddit.com/r/Anki/comments/1h2otym/anki_2411_one_of_the_biggest_updates_ever/), ~2025
- Отношение к FSRS vs SuperMemo: "FSRS is the open source algorithm. Any developer or researcher can use and customize the algorithm. So perhaps FSRS does not intend to compete with and beat its competitors. It is intended to be used by developers of other learning apps like RemNote or to be examined and improved by researchers." — r/Anki, [Do you think FSRS is the best algoritm for spaced repetition?](https://www.reddit.com/r/Anki/comments/1kk0ble/do_you_think_fsrs_is_the_best_algoritm_for_spaced/), ~2025
- Контекст: LMSherlock — volunteer, не фултайм. "The developer of FSRS LM-Sherlock is a volunteer. He collects donations from official Anki and the community, but does not raise a budget comparable to a full time job." — r/Anki, тот же тред.
- Важно: Jarrett Ye = LMSherlock = один и тот же человек (также разработчик Anki, а не только аддона FSRS).

**Проблема auto-optimize перед включением по умолчанию:**
> "Let's not make FSRS the default before automatic optimization. Realistically, how many users do you expect to click 'Optimize' at least once in their lifetime? I'd say 50% at best, likely less. [...] For an average user who is using Anki with out of the box settings won't realize that optimization has to be done at all." — r/Anki, [FSRS will (almost) certainly become the default](https://www.reddit.com/r/Anki/comments/1h8ss2p/fsrs_will_almost_certainly_become_the_default/), ~2025

**Общий паттерн:** Команда Anki/FSRS чинит реальные проблемы (FSRS-5 добавил same-day reviews, FSRS-6 — персональные кривые забывания, встроен auto-optimize), но процесс занимает месяцы/годы. Мейнтейнер один — это риск.

---

## 2. Mochi.cards

### 2.1 Похвала (с цитатами)
>
> "I recently switched to mochi for my cards from anki, since it provides the possibility to ai generate cards, uses markdown and provides internal links." — r/Anki, [Mochi.cards](https://www.reddit.com/r/Anki/comments/178jd8q/mochicards/), ~2024

> "Closest alternative I've found is mochi, there's a free version but for syncing and fancier features it's $5 a month." — r/Anki, [Anki alternatives? (Autumn 2024)](https://www.reddit.com/r/Anki/comments/1g0h2l4/anki_alternatives_autumn_2024/), ~2024

> "Users already have the ability to 'generate cards from note', but for me, the ideal extension of that is to be able to embed / integrate cards directly into the notes." — ответ основателя Mochi в r/Slatestarcodex, [Spaced repetition allows you to remember anything better](https://www.reddit.com/r/slatestarcodex/comments/c7w8km/spaced_repetition_allows_you_to_remember_anything/), 2019

### 2.2 Критика (с цитатами)
>
> "Although i need to get used to the fact there are only two options 'forgot/remembered' while studying. I feel like this makes it longer to study cards for me." — r/Anki, [Mochi.cards](https://www.reddit.com/r/Anki/comments/178jd8q/mochicards/), ~2024

> "The biggest pain point for me so far is the UI. Most of the UI takes a little while to get used to, and other parts of the UI, such as templates and fields, are really confusing. [...] the learning curve is a little steep. A video tutorial on their end or more examples of how to use fields (especially AI fields) would help." — r/Anki, [Mochi.cards](https://www.reddit.com/r/Anki/comments/178jd8q/mochicards/), ~2024

> "I rlly like the idea of gamifying addons but i rlly like mochi's UI." — r/Anki, [Mochi or Anki?](https://www.reddit.com/r/Anki/comments/1obsnp0/mochi_or_anki/), ~2025

> "Even though Mochi is a great alternative, I used it for some months especially due to the UI, but ended up coming back to Anki because I wanted to build knowledge long-term without the sudden risk of the website/system not working anymore, aside from that, Mochi it is not as powerful as Anki." — r/Anki, [Mochi or Anki?](https://www.reddit.com/r/Anki/comments/1obsnp0/mochi_or_anki/), ~2025

### 2.3 Паттерны ответов команды

Нет свидетельств, что команда Mochi публично отвечает на Reddit в найденных тредах. Основатель Mochi ответил один раз в треде r/Slatestarcodex 2019 года. В r/remNote, когда пользователи жаловались на баги после обновления, разработчик RemNote ответил: "Hey, we understand that the recent update has caused some issues, but we have been working around the clock to get them fixed" — r/remNote, [RemNote Alternatives](https://www.reddit.com/r/remNote/comments/w72sga/remnote_alternatives/), ~2022.

---

## 3. RemNote

### 3.1 Похвала
>
> "RemNote is really great for taking notes and directly using them for flashcards. I love the search function and how it is structured. Even if it wouldn't have the flashcards feature it would be a superior database to Anki." — r/remNote, [Anki vs RemNote](https://www.reddit.com/r/remNote/comments/16pulsi/anki_vs_remnote/), ~2024

> "The ui. It's easier to get to know how to learn remnote and even exploring it feels a little bit more intuitive then the coding approach to Anki." — r/remNote, [Anki vs RemNote](https://www.reddit.com/r/remNote/comments/16pulsi/anki_vs_remnote/), ~2024

### 3.2 Критика
>
> "I hate that I can only upload certain amounts of pictures and it is buggy sometimes. That's why I still use Anki just for flashcards while I take notes on RemNote and use the flashcards only sometimes." — r/remNote, [Anki vs RemNote](https://www.reddit.com/r/remNote/comments/16pulsi/anki_vs_remnote/), ~2024

> "With the recent update, everything has been so laggy that I'm scared to lose all my data while trying to study for an exam." — r/remNote, [RemNote Alternatives](https://www.reddit.com/r/remNote/comments/w72sga/remnote_alternatives/), ~2022

**Проникновение среди JP-изучающих:** Не найдено значимых упоминаний RemNote среди изучающих японский в r/LearnJapanese или r/languagelearning.

---

## 4. Сквозные темы возражений

| Тема возражения | Где встречается | Пример-цитата | Типичная реакция |
|---|---|---|---|
| **«Интервалы слишком длинные — забуду к экзамену»** | r/Anki (общая) | "Recent common complaint about FSRS in the Anki community is that the time interval is too long" — [r/Anki](https://www.reddit.com/r/Anki/comments/1kk0ble/), ~2025 | "For a high-stakes exam you might want to bump this to 92-93%" — [r/Anki](https://www.reddit.com/r/Anki/comments/1s1y6om/), 2026-04 |
| **«Не чувствую, что учусь — всё забылось»** | r/Anki (новички) | "I've seen a few complaints/comments saying they felt like they weren't even learning anymore with FSRS and switched back" — [r/Anki](https://www.reddit.com/r/Anki/comments/1s1y6om/), 2026-04 | "Yes, enable FSRS." — стандартный совет комьюнити |
| **«Параметры непонятны / слишком сложно»** | r/Anki (новички и mid-level) | "it still isn't really clear to me on what basis FSRS learns" — [r/Anki](https://www.reddit.com/r/Anki/comments/1r5oluo/), ~2026 | "Beep boop, human! If you have a question about FSRS, please refer to the pinned post" — autobot в r/Anki |
| **«Нужно 1000+ ревьюсов для optimizer»** | r/Anki | Community-made аддон как workaround | Auto-optimize в Anki 24.11+ частично решает это |
| **«Hard button misuse ломает интервалы»** | r/Anki (10%+ юзеров) | "Making FSRS the default will be a horrible mistake" — [r/Anki](https://www.reddit.com/r/Anki/comments/1h8ss2p/), ~2025 | FAQ/warnings + напоминания autobot |
| **«Де-факто vendor lock-in / риск shutdown»** | Пользователи Mochi | "ended up coming back to Anki because I wanted to build knowledge long-term without the sudden risk of the website/system not working anymore" — [r/Anki](https://www.reddit.com/r/Anki/comments/1obsnp0/), ~2025 | Команда Mochi не отвечает |
| **«DR по умолчанию 90% — неэффективно»** | r/Anki, r/LearnJapanese | "The default value of 90% is… horribly inefficient" — [r/LearnJapanese](https://www.reddit.com/r/LearnJapanese/comments/1p66osq/), ~2025-12 | Симулятор: 90→85% = −30% reviews, −3% запомненных карточек |
| **«UI Anki устарел / bloated»** | r/Anki, r/studytips | "Anki also feels a bit too bloated for many use cases" — [r/studytips](https://www.reddit.com/r/studytips/comments/1rl7pvz/), ~2026 | Community-аддоны, не от команды Anki |
| **«Streak anxiety / SRS becomes a chore»** | r/LearnJapanese | "it felt more and more like I was fighting with Anki rather than using it as a tool" — [r/LearnJapanese](https://www.reddit.com/r/LearnJapanese/comments/1r6h58y/), ~2026 | «делайте перерывы, streaks мотивируют, но не позволяйте им владеть вами» |

---

## 5. Уроки для Origa (предсказания)

### 5.1 Какие возражения ПОЛНОСТЬЮ ПРИМЕНИМЫ к Origa

| Возражение | Применимость | Митигация в Origa |
|---|---|---|
| **«Нужно 1000+ ревьюсов для optimizer»** | ⚠️ ВЫСОКАЯ | **Auto-optimize** без отдельной кнопки. Ключевое преимущество перед Anki. |
| **«Hard button misuse → сломанные интервалы»** | ⚠️ ВЫСОКАЯ | Упрощённый UI оценки (3 кнопки: Again/Good/Easy), Hard скрыт или не по умолчанию |
| **«Интервалы слишком длинные — забуду к экзамену»** | ⚠️ СРЕДНЯЯ | JLPT-specific пресет с DR=90%+. Lifelong-дефолт = 85%. |
| **«Не чувствую, что учусь»** | ⚠️ СРЕДНЯЯ | Показывать retention + weekly progress («за неделю вы выучили X новых слов») |
| **«Параметры непонятны»** | ⚠️ СРЕДНЯЯ | Скрыть жаргон FSRS под капотом. Только DR с понятным описанием. |

### 5.2 Сильные стороны Origa: что хвалят в FSRS и у Origa уже есть

- **FSRS-6 с персональными кривыми забывания** — Origa использует rs-fsrs
- **Нативное десктоп-приложение (Tauri)** — Mochi фейлит, потому что web-only (риск vendor lock-in). Origa — desktop-first.
- **OCR (NDLOCR-Lite) + STT (Whisper)** — Уникальное преимущество
- **Японский-фокус** — lindera + UniDic, pitch accent, kanji animations, grammar data
- **Современный UI (Leptos/WASM)** — Адресует «UI Anki устарел»

### 5.3 Слепые пятна Origa: что Origa НЕ покрывает (= будущий риск)

| Слепое пятно | Риск | Рекомендация |
|---|---|---|
| **«Vendor lock-in / портативность данных»** | Пользователи боятся потери данных | **Anki-compatible экспорт.** Маркетить «ваши данные — ваши, экспорт в любой момент» |
| **«Streak anxiety / review burden»** | Пользователи Anki жалеют о «fighting with Anki» | НЕ делать streaks видимыми по умолчанию |
| **«Только 2 кнопки (forgot/remembered) — скучно / долго»** | Mochi критикуют за 2 кнопки | 3 кнопки — компромисс (Again/Good/Easy) |
| **«Пользователи SM-2 не хотят переходить»** | Бывшие пользователи Anki с привычками SM-2 | «FSRS — современнее SM-2, но вам не нужно ничего настраивать» |
| **«DR по умолчанию 90% = неэффективно»** | Power users будут жаловаться | DR=85% по умолчанию |
| **«Community / add-ons / ecosystem»** | Anki — 20 лет экосистемы | Landing + docs — шаг. Долгосрочные усилия. |

---

## 6. Другие упомянутые SRS-стартапы

| Продукт | Что нашёл | Статус |
|---|---|---|
| **repeater** (Rust CLI) | "Written with care in Rust to be super fast" — [r/commandline](https://www.reddit.com/r/commandline/comments/1q89ztq/repeater_the_fastest_most_powerful_anki.json), ~2026 | Нишевый CLI, не конкурент |
| **Deckademy** | "developed by the Refold team" — [r/Anki](https://www.reddit.com/r/Anki/comments/1g0h2l4/), ~2024 | Не найдено отзывов в JP-контексте |
| **Fresh Cards 2.0** (macOS) | One-time purchase — [r/macapps](https://www.reddit.com/r/macapps/comments/r929gv/fresh_cards_20_is_out_now_for_macos_revamped_ui.json), ~2021 | Только macOS |
| **StartMemorizing** | AI-powered web app — [r/memorization](https://www.reddit.com/r/memorization/comments/1kjg7ag/start_memorizing.json), ~2026 | Generic |
| **Mnemosa** | **Значимого присутствия на Reddit не найдено** | Возможно неактивен |
| **NerdSip** | 200 пользователей / $80 MRR — [r/EntrepreneurRideAlong](https://www.reddit.com/r/EntrepreneurRideAlong/comments/1s7x7ar/), ~2026 | Generic science facts |

---

## 7. Неподтверждённое / не найдено

- **Тред «To people still using SM2 instead of FSRS: why?»** на r/Anki — заблокирован network security Reddit
- **RemNote среди JP-изучающих** — значимых упоминаний нет
- **Mnemosa** — нулевое присутствие на Reddit
- **Статус shutdown Mochi.cards** — не найдено объявления
- **Отзывы в App Store / Google Play** — не собирались
- **r/fsrsers** — Tavily-поиск не вернул контент (возможно, маленький приватный сабреддит)
