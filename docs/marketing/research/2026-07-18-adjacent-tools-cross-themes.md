# Смежные инструменты и сквозные комьюнити-темы — Sentiment-ресерч

> **Источник:** Делегированный ресерч через `@tool-accessor` (Tavily + GitHub issues + Chrome Store + HN)
> **Дата:** 2026-07-18
> **Статус:** Сырой research-ввод — не фактчекился как самостоятельный
> **Ограничение канала:** Reddit `.json` эндпоинты НЕ использовались

## Методология

- **Дата:** 18 июля 2026
- **Инструменты:** Web search (Tavily/Exa), прямые URL-запросы, GitHub issues/reviews, отзывы в Chrome/Firefox store, треды Hacker News
- **Reddit `.json` эндпоинты:** НЕ использовались (мертвы с мая 2026)
- **Что НЕ нашлось:** см. Part H

---

## Part A — Смежные иммерсионные/словарные инструменты

### A1. Yomitan/Yomichan

**История форка:**
Yomichan (создан FooSoft/Alex Yatskov ~2016) был официально свёрнут 26 февраля 2023. Причина: unmaintained, deprecation Manifest V2 делал расширение нежизнеспособным. FooSoft отказался передавать ownership — боялся squatting'а и рекламных покупок расширений с 100K+ пользователями ("I still get weekly offers to sell out my users... five figures USD").

TheMoeWay (комьюнити японской культуры/learning) создали Yomitan как форк. FooSoft официально признал Yomitan «the definitive successor to Yomichan». Стабильный релиз — декабрь 2023. 2625+ звёзд на GitHub, 288 открытых issues.

> "It's by far one of the most important pieces of software for language learning! Yomitan has helped me personally create over 11,000 Anki cards so far" — r/LearnJapanese, июль 2025
> URL: <https://www.reddit.com/r/LearnJapanese/comments/1l7fdt7/>

**Что хвалят:**

- Speed, pop-up dictionary, anime-actor-immersion workflow
- Интеграция создания карточек Anki — «единственный лучший бесплатный инструмент для изучающих японский»
- Бесплатность, отсутствие трекинга

**Что хейтят:**

- Настройка словарей: "5-10 minutes of fumbling around multiple websites" (команда разработчиков сама признаёт: <https://www.reddit.com/r/LearnJapanese/comments/1dctzme/>)
- CPU lockups: "Extension randomly locks up and starts consuming tons of CPU until I force stop it" — Issue #1778 (<https://github.com/yomidevs/yomitan/issues/1778>)
- Acknowledgement timeout errors, ведущие к крашам — Issue #2331
- Chrome Web Store 4.8★/152 отзыва — боли: сложность настройки словарей, зависания браузера, ошибки «extension corrupted», нет интеграции с Forvo audio

**Трение миграции:**
Yomichan не поддерживал экспорт словарей — пришлось создать отдельный инструмент `yomichan-data-exporter` для миграции. "This has in fact been a sore point... Yomichan has always missed the migration story"

### A2. 10ten Japanese Reader (бывш. Rikaichamp)

**Позиционирование:** «Simple, reliable, fast, and always up-to-date.» Firefox Add-ons: 4.8★, 270+ отзывов.

**Что хвалят:**

- "FAST! It's great for quick lookup and SUPER responsive. Lacks the features of Yomichan but I like it that way as it's MUCH faster"
- "Works right after installing", "No drama"
- Отзывчивость разработчика — баги из отзывов чинит за недели
- Авто-обновление словарей (еженедельно)

**Что хейтят:**

- "Seems promising, but there's no Japanese monolingual dictionary — avoid this extension if you want to actually learn the language"
- Не работает в модals/popups (Notion calendar, заголовки тредов Reddit)
- Не anime-friendly: нет Anki-интеграции, нет продвинутых фич Yomitan
- Контроверсия смены шрифта

**Урок для Origa:** Простота = похвала. Но отсутствие глубины = потолок engagement.

### A3. Мобильные словари (Manabi Reader, Shirabe Jisho, Takoboto)

**Manabi Reader (iOS/macOS):**

- 4.5★ App Store, 219 отзывов
- **Ключевое:** Offline-first, privacy-sensitive архитектура
- "This has completely changed my approach to reading in Japanese... Fills a gap for Japanese learners on mobile" — отзыв в App Store
- "Privacy sensitive. Unlike most other software these days, Manabi Reader keeps your reading data on your device" — разработчик
- Anki-интеграция = "huge game changer"
- **Жалобы:** Попап подписки до того, как что-либо попробуешь, периодические зависания на iPad, бесплатные пользователи не могут добавлять в Anki

**Offline-first параллель с Origa:**
Manabi активно использует «offline-first + privacy» как маркетинг. Отзывы в App Store неоднократно упоминают приватность как плюс.

### A4. Иммерсионная экосистема и анти-Anki сентимент

**Migaku, ImmersionKit, AnimeCards:**

- ImmersionKit: 500K+ японских предложений из аниме/VN/дорам, бесплатный веб-инструмент. Стандарт в workflow sentence mining
- AnimeCards (animecards.site): "review data suggests anime cards can be reviewed 2 to 4 times faster than sentence cards"
- Migaku: "Anki works, but the workflow is not for everyone. You're constantly switching between consuming content, looking up words, creating cards... Migaku integrates the whole process"

**Анти-Anki сентимент:**

- "I use Anki a lot and thanks for these, this is great... Neither Ricotta nor Anki have been able to help me learn this language" — HN, тред про Ricotta
- "the system of Anki is way too primitive" — Nihondex (HN)
- "I have 0 idea how to speak or really understand when people are speaking to me in Japanese" — TalkMochi (HN)
- "There's much debate about the best Anki card type... many options exist, but most are either useless or helpful only in specific situations" — animecards.site
- "I've been doing 40 cards per day for past 14 days and it's not sustainable long term because my reviews just eventually pile up" — r/LearnJapanese

**Паттерн:** Люди устали от трения Anki, но НЕ от концепции spaced repetition.

---

## Part B — Восприятие OCR

### B1. Репутация NDLOCR

**NDLOCR** — разработан Национальной парламентской библиотекой (NDL) Японии. Полное имя: NDLOCR-Lite (GPU-free лёгкая версия, выпущена февраль 2026).

**Бенчмарки:**

- **Печатные документы (papers):** CER 0.016 (1.6% error rate) — «практически идеально»
- **Книги:** CER 0.323
- **Рукописный текст:** CER 0.268 (JaWildText benchmark, 1065 изображений)
- **Размер модели:** ~146MB (ONNX)
- **Скорость:** ~1-5 секунд на страницу на обычном ноутбуке

> "NDLOCR-Liteは、縦書き・横書き・二段組のいずれにおいても最も安定した精度を示し... 有力な選択肢になったといえます" (NDLOCR-Lite показал самую стабильную точность в вертикальной, горизонтальной и двухколоночной вёрстке — сильный кандидат)
> URL: <https://estyle.co.jp/media/エンジニアアログ/2973/>

**Репутация:** Почти неизвестен за пределами японских архивных/исследовательских сообществ. Освещение GIGAZINE: «NDLOCR-Lite, the National Diet Library's free OCR app». Лицензия CC BY 4.0. В контексте Japanese language learning — практически нулевое упоминание в Reddit/HN. Это **Возможность** для Origa.

### B2. Дебаты локального vs облачного OCR

**Namida OCR** (Chrome-расширение, Tesseract.js): "Privacy-Focused: No external servers are used. All OCR happens locally in your browser." — листинг в Chrome Web Store.

**ScanLingua** (Chrome-расширение, Google Vision API): требует API key, но 1000 бесплатных запросов/мес.

**Workflow'ы Yomitan + image OCR:**

- Namida OCR существует отдельно от Yomitan — не встроен
- manga-OCR / Scanji-Translate — отдельные инструменты

**Возражение «Локальный OCR хуже Google»:**

- Активных жалоб в jp-learning комьюнити не найдено — большинство пользователей либо используют облачный OCR (Google Vision), либо вообще не делают OCR
- Слепое пятно Origa: локальный OCR для манги/LN — нишевая потребность, но сильная дифференциация vs. приложений, отправляющих данные на сервер

---

## Part C — Восприятие STT/Whisper

### C1. Восприятие локального Whisper

**Японские ASR-бенчмарки (2026):**

| Модель | WER/CER | Скорость | Заметки |
|-------|---------|-------|-------|
| Qwen3-ASR-1.7B | 0.185 CER | 36ms | Лучшая точность |
| Whisper Large-v3-turbo | 0.218 CER | 13ms | Лучшая стабильность в шуме |
| Kotoba-Whisper-v2.0 | 0.534 CER | 8ms | Худшая в бенчмарке |

> "qwen3-asr-1.7b and whisper maintained stable accuracy even in multi-speaker and noisy conditions"
> URL: <https://neosophie.com/en/blog/20260226-japanese-asr-benchmark>

**Google Chirp 3 vs Whisper (японский бизнес):**

- Google Chirp 3: 6.4% CER (с speech adaptation)
- Whisper: 36.5% CER (Groq, тот же аудио)
- "6.4% error rate means only 6 mistakes per 100 characters. 36.5% means nearly half the content is unreliable."

> URL: <https://paulkuo.tw/en/articles/google-chirp3-japanese-stt-benchmark/>

**Основные жалобы на Whisper:**

1. "accurate model size is too big" — r/LocalLLaMA
2. "Not real time" — r/LocalLLaMA
3. Галлюцинации на тишине: "Whisper sometimes invents plausible-sounding text" — novascribe.ai
4. Japanese-specific: "Whisper model returns incorrect transcription... 'ご視聴ありがとうございました' regardless of input" — GitHub openai/whisper #2377

**SenseVoice** (FunAudioLLM): Появляется как лучший для CJK. Whisper Notes app: "For Chinese, Japanese, Korean, and Cantonese, use SenseVoice" — рекомендуется поверх Whisper для CJK.

**Риск для Origa:** Если используется старый Whisper без VAD-preprocessing, пользователи могут получать плохие результаты.

---

## Part D — Сквозные dev-комьюнити-темы

### D1. Паттерны возражения «Yet another Anki alternative» (5+ примеров)

**Пример 1 — Ricotta (HN):**
"Anki is one of my favorite pieces of software ever so I was definitely willing to try yours. Seems like the feature that defines your app is auto-generation of a card / set of cards. Didn't see any way to create your own card... if that's not working well the entire thing crumbles." + "the translations were wrong, the vocabulary was strange" (японский пользователь)
URL: <https://news.ycombinator.com/item?id=42966942>
**Паттерн:** Без уникальной killer feature — «зачем switch?»

**Пример 2 — Nihondex (HN):**
"Duolingo doesn't work... I created Nihondex because I wasn't making real progress with platforms like Duolingo or Bunpro. I tried Anki cards... but the system of Anki is way too primitive."
URL: <https://news.ycombinator.com/item?id=45412188>
**Паттерн:** Прямая критика Anki работает лучше, чем игнорирование Anki

**Пример 3 — Lingoku v1 (HN):**
"I built this because I struggle with Anki burnout and wanted a way to review words without feeling like I am 'studying'" — положительный приём
URL: <https://news.ycombinator.com/item?id=46296863>
**Паттерн:** Фрейминг «Anki burnout» = симпатизирующая аудитория

**Пример 4 — Kana Dojo (r/coolgithubprojects):**
"As someone who loves both coding and language learning... I always wished there was a free, open-source tool for learning Japanese, just like Monkeytype in the typing community" — 1K звёзд
URL: <https://www.reddit.com/r/coolgithubprojects/comments/1rpb7q5/>
**Паттерн:** «Open source + free» сильно резонирует

**Пример 5 — generic Anki alternative (HN, 2021):**
"The main reason why everyone still uses Anki despite its issues is because it is still hands-down the best solution out there... There are a million and one spaced repetition systems out there, but Anki's plugin system and shared decks make for a very strong network effect."
URL: <https://news.ycombinator.com/item?id=27662266>
**Паттерн:** Network effect / ecosystem lock-in это реальный ров

**Пример 6 — AnkiBrain (r/ankibrain):**
Пользователи жалуются на кредитную систему ("my balance was too low? i thought this was free"), проблемы создания аккаунта. Новая модель монетизации фрустрирует пользователей.
**Паттерн:** Paywall на ранее-бесплатной функциональности = мгновенная ненависть

**Что срабатывало в ответах авторов:**

- Признание силы Anki («I love Anki, I just wanted to solve X»)
- Показ, а не рассказ — demo > описание
- Прозрачность в том, что отличается (один чёткий дифференциатор)
- Open source = мгновенный кредититет

### D2. Восприятие BSL 1.1 / source-available лицензии

**Кейс HashiCorp BSL (август 2023):**
> "BSL is not open source, so this would mean moving Terraform back to the MPL license" — Манифест OpenTofu, 80+ соавторов в первый месяц

> "The problem is that switching an open source license to the BSL erodes trust in your product and your company." — блог VictoriaMetrics

> "BSL breaks that guarantee. And the reaction has been fierce." — анализ Felipe Hlibco

**Ключевая динамика:**

1. **Source-available ≠ Open Source** — OSI явно исключает BSL. Реакция dev-комьюнити: «source available» встречается с глубоким подозрением
2. **Ретроактивная смена лицензии = предательство** — когда HashiCorp переключилась с MPL на BSL спустя годы построения комьюнити, это восприняли как «маркетинговые плюсы open source без пользы для комьюнити»
3. **Форки случаются быстро** — OpenTofu форкнул за 15 дней, получил 33K звёзд за первый месяц, вошёл в Linux Foundation
4. **Исход контрибьюторов** — community PR упали с 21.12% до 9.30% после переключения на BSL

> "If a project on GitHub only has maintainers from the corporate side, you can be certain that they will ultimately drive the product for their own interest solely." — комментатор HN

**Специфичный риск для Origa:** BSL 1.1 с первого дня ≠ ретроактивное переключение (меньше гнева, чем у HashiCorp). НО: dev-комьюнити в Rust-space имеют высокие стандарты к лицензиям. Tauri (Apache-2.0/MIT), Leptos (MIT) — конкурирующие стеки используют пермиссивные лицензии. BSL = точка трения для контрибьюторов. **Митигация:** Позиционировать BSL как «защищает пользователей от форков, которые могут добавить телеметрию» (privacy-angle).

### D3. Восприятие зрелости Tauri v2

**Положительные сигналы (2026):**

- 106K звёзд на GitHub, «новый дефолт для кросс-платформенного десктопа» — Vanja Petreski
- "If you're starting a new project in 2026... start with Tauri 2. Default position. Justify not choosing it." — Vanja Petreski (май 2026)
- "AI agents handle Rust well. Claude Code and Codex both write Tauri-grade Rust competently." — статья tauri-2-deep-dive
- Бандл: 5-40MB vs Electron 80-200MB. Память: 40-90MB vs 150-400MB idle

**Негативные сигналы:**

- "I was wrong about Electron... Tauri uses WebView2... Getting the app working on Windows was not a problem. On Linux... WebKitGTK... the app just looked different" — разработчик Paseo, мигрировал ОБРАТНО на Electron (май 2026)

> URL: <https://dev.to/moboudra/i-was-wrong-about-electron-1e9g>

- "Native bug rate increased 22% (14→17 per app/month)... 70% platform-specific web view bugs" — производственная ретроспектива
- "The biggest barrier to Tauri is not bundle size... It is Rust. Honest accounting of the cost when a frontend-heavy team picks it up." — анализ youngju.dev
- "If exactly one person on the team can review Rust, that person becomes the bottleneck"
- "Stack Overflow coverage is thin. When you hit an obscure error in Electron, there's usually a Stack Overflow answer. With Tauri, you're often the first person."

**Угол Origa:** Origa — Rust-native команда, поэтому «барьер Rust» не проблема. НО: пользователи будут замечать WebView-несоответствия, особенно на Linux. **Talking point:** «Мы выбрали Tauri, потому что он держит ваши данные локально — никакой рантайм Electron не отправляет телеметрию в Google.»

### D4. Восприятие Leptos / Rust-WASM UI

**КРИТИЧЕСКОЕ РАЗВИТИЕ (май 2026):**
> "Leptos is not abandoned but will be lightly maintained going forward. I consider it feature-complete and do not expect to do any significant new development in the future." — gbj (создатель), GitHub Issue #4707
> URL: <https://github.com/leptos-rs/leptos/issues/4707>

> "The rise of LLMs and of coding agents has really fundamentally changed the cost-benefit analysis on this for me. Over the last year or so I've found that the volume of meaningful community conversation and engagement has declined significantly."
> "I consider Leptos complete. I have shipped every major feature on any of my roadmaps."

**Реакция комьюнити:** Благодарный, но обеспокоенный:
> "Leptos is awesome. I just started my 5th or 6th leptos project yesterday... hyper-active development is not, and should not be, a requirement for a project to be 'good' or 'useful'."
> "Would it make sense for Leptos to be 'given' to Apache?"

**Оценка (до объявления):**

- 21K звёзд на GitHub — самый популярный Rust WASM framework
- «Leptos is Rust's most mature full-stack web framework» — Rustify (апр 2026)
- Всё ещё v0.7 (без релиза 1.0)
- Fine-grained reactivity (модель SolidJS) = лучшая производительность среди Rust-фреймворков
- Размер WASM: ~90KB gzipped (vs React ~45KB + код)

**Риск для Origa:** Переход Leptos в maintenance mode — РЕАЛЬНЫЙ риск. Если фреймворк перестаёт развиваться, возможны проблемы совместимости в долгосрочной перспективе. **Митигация:** Leptos «feature-complete» + MIT-лицензия = форк всегда возможен. Плюс, WASM-слой у Origa относительно тонкий (UI only, бизнес-логика в чистом Rust).

### D5. Риск предвзятости к русскому языку

**Не найдено** активных примеров предвзятости к русскоязычному интерфейсу в JP-learning комьюнити.

**Риски:**

- Reddit/HN могут вызвать политические дискуссии
- 日本語-learning комьюнити (4chan /jp/) — задокументированная токсичность
- Автор AnimeCards/QM идентифицирован как русский — использовано как вектор атаки: «He's Russian» — 4chan /jp/

**Митигация:** UI Origa на английском (вероятно), с i18n-поддержкой. Разработка не в России = менее заметный geopolitical fingerprint.

---

## Part E — Паттерны приёма Show HN / r/SideProject

### E1. Недавние запуски альтернатив Anki

**Ricotta (HN):**

- Заголовок: "Ricotta – Language Learning to Replace Anki"
- Приём: Смешанный. Автор признал проблемы с AI-generated переводами для японского. Любители Anki защитили Anki.
- Upvotes: умеренные

**Manabi Reader v3 (HN):**

- Заголовок: "Native Japanese immersion reader app + Anki integration (Manabi Reader)"
- Приём: 105 поинтов, 57 комментариев — позитивный
- Что получило upvotes: offline-first архитектура, Anki-интеграция, SwiftUI native
- Что получило hate: попап подписки, изначально только iOS
- Ключевой успех: автор известен в комьюнити («I quit my job to work on this full-time»)

**Lingoku (HN):**

- Заголовок: "Learn Japanese contextually while browsing"
- Приём: "Nice concept but the design could use some love as it looks a bit on the vibe coding slop side of things"
- Upvotes: низкие (3 поинта)
- Что получило hate: использование romaji, баги произношения

**Nihondex (HN):**

- Заголовок: "Nihondex Learn Japanese Fast"
- Приём: 5 поинтов, 1 комментарий — почти нулевой
- Проблема: позиционирование «Duolingo doesn't work» без чёткого дифференциатора

**Kachika (HN):**

- Заголовок: "Kachika — Learning Japanese Through Life"
- Приём: 5 поинтов, 1 комментарий
- Проблема: Неясный value prop + только iOS

**Паттерны:**

1. **Известные в комьюнити люди** получают больше engagement'а, чем неизвестные впервые опубликовавшиеся
2. **«Offline-first»** последовательно хвалят как дифференциатор (Manabi)
3. **Anki-интеграция** = мгновенный кредититет (но не заявляйте о замене Anki напрямую)
4. **Качество UI имеет значение** — комментарии «vibe coding slop» реальны
5. **Цена** — промпты подписки до триала = мгновенный негативный отзыв

### E2. Недавние запуски Tauri/Rust-WASM

**ChatML (Tauri 2 desktop AI app):**

- "We chose Tauri 2. It was the right call for our specific situation, but it wasn't an obvious one"
- Бандл: 155MB (в основном sidecar), Память: 80-120MB idle
- Трейд-оффы задокументированы честно — хороший приём

**Paseo (мигрировал С Tauri НА Electron):**

- "I was wrong about Electron... Tauri felt like the better choice... I was building more by vibes than by looking at the tradeoffs objectively"
- Проблемы с Linux WebKitGTK стали dealbreaker'ом

**Retro (6 месяцев, 3 приложения):**

- На 60% меньшие инсталлеры, на 22% больше нативных багов
- "Tauri is the best choice for desktop apps where installer size, memory, startup time are critical, provided you're willing to invest in Rust expertise"

---

## Part F — Восприятие privacy/local-first позиционирования

### F1. Где это заходит хорошо

**Anytype (privacy-first альтернатива Notion):**

- "The open-source app finally convinced me to cancel my Notion subscription" — HowToGeek (февраль 2026)
- "Your data lives on your device. Period... If Anytype the company disappeared tomorrow, you would still have all your data and a working app" — MakerStack
- Приём на HN: изначальный скепсис («blockchain?! scam!») → конверсия после open-source релиза
- **Ключевое:** Local-first сильнее всего резонирует с пользователями, которых уже жглили облачные сервисы

**Standard Notes:**

- "People arrive at Standard Notes after something goes wrong somewhere else" — unsubbed.co
- "The pattern across third-party commentary is consistent" — privacy-комьюнити последовательно рекомендует
- Без VC-финансирования = доверенное управление

**Manabi Reader:**

- "Privacy sensitive. Unlike most other software these days, Manabi Reader keeps your reading data on your device" — разработчик
- Offline + privacy = "big win" по комментариям на HN

### F2. Где отвергают

**Ранние дни Anytype (HN):**

- "It's not privacy-first until the source is open" — комментатор HN (до open-source релиза)
- "The scrolling banner screams 'this is a scam'" — UX лендинга убил доверие
- "I've applied for an invite more than 2 years ago - still got nothing! This looks like an email honeypot!"

**Ключевой урок:** Заявление «privacy-first» без верифицируемости (open source, аудит) = подозрение. **С** верифицируемостью = сильная лояльность.

---

## Part G — Уроки для Origa (предсказания)

### Топ-5 скрытых рисков

1. **«Yet another Anki alternative»** — САМЫЙ ВЫСОКИЙ РИСК
   - **Что случится:** Первые комментарии: «Why should I switch from Anki?»
   - **Митигация Origa:** НЕ позиционировать как Anki-alternative. Позиционировать как «immersion tool с SRS» — акцент на workflow OCR/STT/интеграции, не на флешкартах.
   - **Talking point:** «Anki — отличный инструмент для того, что делает. Origa делает всё, чего Anki не делает — локальный OCR, распознавание речи, обработка нативного японского текста.»

2. **Бэклаш на лицензию BSL 1.1** (средний риск)
   - **Что случится:** Dev-savvy пользователи на HN: «Not open source. Pass.»
   - **Митигация Origa:** Проактивно объяснить выбор BSL. «BSL защищает ВАС — он не даёт никому форкнуть Origa, добавить телеметрию и продавать ваши данные.»
   - **Talking point:** «Ваши данные обучения остаются приватными. BSL гарантирует, что никакой форк не сможет это изменить.»

3. **Leptos в maintenance mode** (средний риск)
   - **Что случится:** «You're building on an abandoned framework»
   - **Митигация Origa:** Leptos «feature-complete» + MIT + 21K звёзд = безопасная ставка. Плюс, фронтенд Origa — слой над бизнес-логикой, работающей на чистом Rust.
   - **Talking point:** «Leptos стабильный, производительный и под MIT. Наша core-логика не зависит от конкретного UI-фреймворка.»

4. **Геополитический риск российского происхождения** (средний риск)
   - **Что случится:** «Is this a Russian app?» → политические дискуссии, особенно на Reddit
   - **Митигация Origa:** EN-first UI, прозрачная страница команды, фокус на миссии изучения японского. Не подсвечивать российское происхождение; не прятать тоже.
   - **Talking point:** Изучение японского универсально. Origa построена людьми, которые учат японский.

5. **Tauri WebView-несоответствия** (низко-средний риск)
   - **Что случится:** Пользователи сообщают о багах рендеринга на конкретных платформах
   - **Митигация Origa:** Тщательное тестирование в CI на 3 ОС. Модель безопасности Tauri — на самом деле ПЛЮС для privacy-позиционирования Origa.
   - **Talking point:** «Tauri держит Origa лёгким — 10-30MB vs 150-200MB для приложений Electron. Батарея вашего ноутбука говорит спасибо.»

---

## Part H — Неподтверждённое / не найдено

1. **Предвзятость к русскоязычному интерфейсу в JP-комьюнити** — конкретных примеров не найдено
2. **Конкретные комментарии «yet another Anki» на недавних запусках конкурентов Origa** — найдены общие паттерны, но не исчерпывающие 5+ прямых цитат с URL для каждого случая
3. **Использование NDLOCR в западных JP-learning комьюнити** — практически нулевое
4. **Специфичные жалобы на Whisper в JP-learning (не-tech) комьюнити** — большинство изучающих используют облачные API (Google STT)
5. **Анти-Anki сентимент конкретно в r/LearnJapanese** — найден общий сентимент, но не исчерпывающий тред-за-тредом анализ
6. **Зрелость Tauri v2 mobile для production-приложений** — ограниченные production-примеры
7. **Сентимент комьюнити Migaku/ImmersionKit за пределами маркетинговых страниц** — ограниченные органичные обсуждения пользователей
