# Origa — Operational Outreach Playbook

> **Статус:** Черновик → HUMAN GATE
> **Дата:** 2026-07-21
> **Связанный документ:** `marketing/strategies/origa-distribution-platforms.md` (стратегия верхнего уровня)
> **Назначение:** Operational слой — куда писать, какой процесс, что работает / не работает для каждой Tier 1 площадки. Стратегический документ даёт «куда», этот playbook даёт «как именно».
> **Источники:** web research 2026-07-21, `objections-handling.md`, `reddit-strategy.md`, `origa-seo.md`

---

## 0. Карта быстрого доступа — контакты Tier 1

| # | Площадка | Канал контакта | Формат | Платформа |
|---|---|---|---|---|
| 1 | **Tofugu** | hello@tofugu.com (или форма на /contact/) | Review request | EN |
| 2 | **skerritt.blog** | Twitter DM / blog comment (email не публичный) | Roundup pitch (ноябрь-декабрь) | EN |
| 3 | **MariaTheMillennial** | Контактная форма на сайте / BuyMeACoffee DM | Review request | EN |
| 4 | **Hacker News (Show HN)** | Submit на news.ycombinator.com/submit | Launch post | EN |
| 5 | **Daigaku Telegram** | @OuroreS (DM) | App review pitch | RU |
| 6 | **Nihongo Telegram (@yaponskoe)** | @sonyasonnoya или @arthur_kron — **только платная реклама через telega.in** | Реклама (платно) | RU |
| 7 | **Хабр** | Статья в Песочницу → получить полноправный аккаунт → корпоративный блог | Техническая статья (НЕ реклама) | RU |
| 8 | **RuStore editorial** | Заявка в Консоли разработчика → фичеринг | Editorial featuring (бесплатно, по решению редакции) | RU |
| 9 | **vnjpclub.com** | vnjpclubinfo@gmail.com или admin@vnjpclub.com | Site review / diễn đàn thread | VI |
| 10 | **Minato Dorumu FB** | FB Messenger DM админу через Members → Admins list | Group mention | VI |
| 11 | **Tokutei ginou FB groups** | FB Messenger DM админу (требует идентификации групп — см. §11) | Group mention | VI |
| 12 | **DC Inside JLPT gallery** | Пост в галерее (нужен 고정닉) | 홍보 / story post | KO |
| 13 | **Naver Cafes** | Join cafe → DM مدیر через 닉네임 | Cafe mention | KO |

**❗ Важно по Nihongo (@yaponskoe):** несмотря на то что это второй по размеру RU JP-канал, в описании канала явно указано «По рекламе @sonyasonnoya или @arthur_kron. Менеджеров нет». Это означает что канал работает только через **платную рекламную биржу** (telega.in). Бесплатного органик-review нет. **Переместить из Tier 1 в «платный канал — out of scope»** или искать alternative RU JP-канал.

---

## 1. Tofugu (EN, Tier 1)

### Контакт

- **Email:** hello@tofugu.com
- **Альтернатива:** contact form на https://www.tofugu.com/contact/ (идёт в тот же inbox)
- **Не использовать:** jobs@tofugu.com (только для job applications)

### Процесс

У Tofugu **нет submission-формы для app review**. Процесс — классический cold email.

Tofugu публикует reviews японских learning-продуктов в устоявщемся формате (пример: [Tofugu review of Bunpro](https://www.tofugu.com/reviews/bunpro/) — 8/10). Статьи пишут в характерном «Tofugu style»: дружелюбный, с housemade GIF'ами, конкретные оценки.

### Что работает

- Персонализированный email со ссылкой на недавнюю статью Tofugu + take
- Предложение прислать screenshots + 30-секундное demo video + review license
- Явный ask: standalone review ИЛИ inclusion в future tools roundup
- Чёткое описание wedge'а Origa: native RU/VI/KO UI, локальный OCR + STT, FSRS
- Disclosure BSL upfront («source-available, free for personal use, BSL 1.1»)

### Что НЕ работает

- Шаблонный mass-email без отсылки к конкретной статье
- Pitch в стиле «лучшее приложение для японского» (Tofugu стиль — скромный)
- Сокрытие BSL — они технически подкованы, откроют LICENSE
- Спам follow-up'ами (один polite bump через 7 дней, потом drop)

### Цикл ответа

- Tofugu — небольшая команда (Portland, OR, ~10-15 человек), цикл ответа **2–6 недель**
- Отправлять **за 6+ недель до launch** если хочется синхронизировать с launch-датой

### Шаблон — see `origa-distribution-platforms.md` §6.2 Template A (EN)

---

## 2. skerritt.blog (EN, Tier 1)

### Контакт

- **Twitter DM:** @autumn_skr (верифицировать — основной публичный канал)
- **Blog comments:** на конкретной статье (медленный, но читается)
- **Email:** не публикуется публично

### Процесс

skerritt.blog ведёт Autumn Skerritt (Bee), публикует ежегодный **"Best Japanese Learning Tools 20XX Award Show"** в декабре (2025-awards опубликован 2025-12-08). Статья затем кросс-публикуется в WaniKani Community (пример: [WK Community thread](https://community.wanikani.com/t/best-japanese-learning-tools-2025-award-show/73437)).

Процесс — pitch в **ноябре-декабре** перед annual roundup. Autumn активно тестирует apps («I tested every Japanese app that came out in the last 2 years»).

### Что работает

- Pitch в ноябре с предложением протестировать Origa перед годовым roundup
- Twitter DM со ссылкой на repo + 30-сек demo
- Подчеркнуть: offline-first + local OCR/STT (это её темы — она обозревала Yomitan, JL, Poe)
- Ссылка на конкретные статьи skerritt.blog (например про Yomitan, Anki add-ons)

### Что НЕ работает

- Pitch вне timing'а (май, июль — не амплитуда roundup'а)
- Generic PR-pitch без указания wedge vs Yomitan/JL (она сравнивает)
- Текст без технической глубины (Bee — technical, копает в implementation)

### Цикл ответа

- Twitter DM: обычно 1–7 дней
- Если не ответил за 7 дней — короткий polite bump

---

## 3. MariaTheMillennial (EN, Tier 1)

### Контакт

- **Контактная форма:** mariathemillennial.com (через сайт)
- **BuyMeACoffee DM:** buymeacoffee.com/mariamillennial
- **Email:** не публикуется публично — через форму

### Процесс

Maria — русскоговорящая (также нидерландская + англоговорящая), делает **long-form reviews JP-learning apps**. Уже обозревала MaruMori дважды. На about-странице явно просит: «Let me know which resource I should try out and review for you» — это **прямой invitation к pitch'у**.

### Что работает

- Персонализированный email через форму или BuyMeACoffee с упоминанием её MaruMori review
- Подчеркнуть: **native RU UI** — она русскоговорящая, это резонанс
- Предложить review license + demo
- Явный ask: standalone review (она делает long-form, не roundup)

### Что НЕ работает

- Pitch без прочтения её MaruMori review
- Игнорирование её multi-language контекста

### Цикл ответа

- 1–2 недели через BuyMeACoffee (быстрее чем форма сайта)

### Почему она особенно важна для Origa

Maria — редкий случай русскоговорящего независимого обозревателя JP-learning apps. Native RU UI Origa — прямой wedge для неё. Это **один из самых перспективных Tier 1 контактов**.

---

## 4. Hacker News — Show HN (EN, Tier 1)

### Контакт

- **Submit:** https://news.ycombinator.com/submit
- **Title format:** `Show HN: Origa – Japanese learning app with local OCR, STT, and FSRS` (≤80 символов)
- **Требование к аккаунту:** HN-аккаунт с историей (не свежезарегистрированный). Username ≠ «origa» или «origa_app» — это нарушает guideline про «не быть brand'ом»

### Процесс

Submit ссылку на **GitHub repo**, не на лендинг. Landing pages off-topic для Show HN — нужен runnable продукт, который можно попробовать.

Текст поста (в body или как первый comment):

- Backstory: как и почему построил Origa
- Clear statement: что проект делает
- Technical depth: архитектура (Rust + Leptos + Tauri + local NDLOCR + Whisper)
- Known limitations (честно)
- Ссылки на previous relevant HN threads если есть

### ❗ КРИТИЧНО — обновление 2026

В марте 2026 модератор dang обновил guidelines: **текст Show HN не должен быть сгенерирован или отредактирован LLM** (даже частично). HN community сейчас крайне чувствительно к этому — LLM-imprints в тексте вызывают flame. **Писать launch-post от руки**, без AI-ассистента.

### Что работает

- Personal voice, без marketing language («Drop any language that sounds like marketing or sales. On HN, that is an instant turnoff»)
- Техническая глубина: конкретные цифры (CER NDLOCR 1.6% на printed docs, размер bundle, и т.д.)
- Ссылка на repo + на CHANGELOG
- Backstory с personal мотивацией («устал сшивать Anki + Yomitan + ...»)
- Honest limitations (это строит доверие)
- First comment автора в течение первого часа — отвечать на каждый комментарий первые 2 часа

### Что НЕ работает

- ❌ Landing page ссылка (off-topic)
- ❌ Marketing language («revolutionary», «game-changing», «next-gen»)
- ❌ Username = company name
- ❌ LLM-generated text (2026 rule — community flame)
- ❌ Просить друзей upvote/comment («vote manipulation = перма-бан»)
- ❌ Booster-comments от друзей (community adept at picking up)
- ❌ Repost (только major overhaul, max 1-2 в год)
- ❌ Скрытие BSL — обязательно раскрыть в посте

### Цикл

- Show HN-пост появляется сразу на /shownew
- Если набирает небольшой points threshold → появляется на /show в top bar
- Пик трафика: первые 2–4 часа
- Best time: Вт/Ср/Чт 7–9 AM EST (US morning, когда HN-сообщество активно)

### BSL handling (see also `objections-handling.md` #2)

BSL **будет поднят в комментах** — это неизбежно на HN. Стратегия:

1. Раскрыть в посте: «BSL 1.1, source-available, free for personal use, converts to permissive after change-date»
2. Иметь talking point готовым: «BSL защищает от форка с добавленной телеметрией — код на GitHub, можно читать каждую строку»
3. Сослаться на precedent: Sentry, CockroachDB, HashiCorp (post-fork) — BSL уже не экзотика
4. НЕ называть «open source» — это мгновенная потеря кредититета

---

## 5. Daigaku Telegram (RU, Tier 1)

### Контакт

- **Обратная связь:** @OuroreS (DM в Telegram)
- **Альтернатива (older):** @ststefan (верифицировать — в разных снапшотах разные username'ы)
- **Чат:** @daigaku_lounge (для пользовательских вопросов, не для pitch'а админу)

### Процесс

Daigaku — крупнейший RU Telegram-канал про японский (21K подписчиков). В описании канала есть тег `#приложения` — это явный сигнал что приложение-обзоры входят в тематику.

Прямой DM админу @OuroreS с предложением рассмотреть Origa для обзора в рубрике `#приложения`.

### Что работает

- Сообщение на русском (локализация — wedge)
- Персонализированная отсылка к недавнему посту канала (например к недавнему посту про приложения)
- Ясное описание: «RU-native JP-app с локальным OCR + STT + FSRS»
- Предложение: готов дать review-лицензию / ответить на вопросы / сделать Q&A
- Уважение к формату канала (не «сделайте мне рекламу», а «может быть интересно вашей аудитории»)

### Что НЕ работает

- Pitch на английском (RU-канал, RU-wedge)
- Generic mass-message
- Требование free promo (это просьба, не требование)
- Спам в @daigaku_lounge (это user-chat, не admin-канал)

### Цикл

- Telegram DM — обычно 1–3 дня
- Если нет ответа за 7 дней — один polite bump

---

## 6. Nihongo Telegram (@yaponskoe) — ⚠️ НЕ БЕСПЛАТНО

### Контакт

- **Реклама:** @sonyasonnoya или @arthur_kron (явно указано в описании канала)
- **Биржа:** telega.in/c/yaponskoe

### Процесс

В описании канала **явно указано**: «По рекламе @sonyasonnoya или @arthur_kron. Менеджеров нет. Реклама через биржу: https://telega.in/c/yaponskoe»

Это означает: канал работает **только через платную рекламную биржу telega.in**. Бесплатного органик-review нет — только paid placement.

### Решение для стратегии

**Переместить @yaponskoe из Tier 1 в out-of-scope** (платный канал). Заменить в Tier 1 на alternative RU JP-канал, который принимает органик-review.

### Alternatives (требуют верификации)

- **@japan_info_nihongo** (Японский язык | JAPAN_info) — уже в Tier 2, проверить есть ли органик-format
- **@japan_teach** (Японский язык | Япония, 9.9K) — уже в Tier 2, проверить
- Другие RU JP-каналы (research needed)

---

## 7. Хабр (RU, Tier 1, stack-angle)

### Контакт / Процесс

Хабр имеет **сложный процесс получения полноправного аккаунта**:

1. **Регистрация** → аккаунт с правами **ReadOnly**
2. Чтобы писать статьи → нужен **полноправный аккаунт**
3. Получить полномочия:
   - Написать публикацию в **Песочницу** → если пройдёт модерацию, аккаунт становится полноправным
   - Получить **invite** от полноправного пользователя (через карму ≥51 или публикацию с рейтингом ≥+50)
   - Стать сотрудником компании с корпоративным блогом на Хабре

### ❗ КРИТИЧНО — правила о рекламе

В правилах Хабра прямо запрещено:

> «Не используйте Хабр как площадку для продвижения своих (и чужих) проектов, сервисов, продуктов, услуг. Если цель публикации — привлечь внимание, продать, набрать лиды или аудиторию, а не поделиться знаниями — это считается рекламой.»

Допустимые форматы:

- **Корпоративный блог** (нужно оформить компанию на Хабре)
- **Однократно в хабе «Я пиарюсь»** (явно для пиара)
- **Техническая статья с реальной ценностью** — не реклама, а инженерный разбор

### Что это значит для Origa

**Нельзя написать «Look at my Japanese app» статью.** Можно:

**Опция A:** Техническая статья про стек с реальным разбором:
- «Архитектура offline-first desktop приложения на Rust + Tauri + Leptos»
- «Локальные ML-модели в desktop-приложении: NDLOCR и Whisper под капотом»
- «FSRS-6 в production: что мы выучили»

Статья стоит на технической ценности, Origa упоминается как пример. Это соответствует паттерну «статья с реальной технической ценностью без прямой рекламы».

**Опция B:** Корпоративный блог Origa на Хабре (если зарегистрировать юрлицо или brand-аккаунт).

**Опция C:** Однократный пост в хабе «Я пиарюсь» (явно для пиара, но без гибкости).

### Что работает

- Техническая глубина — Хабр-аудитория режет marketing-language
- Конкретные цифры, бенчмарки, ADR-ссылки
- Честные trade-offs (Tauri WebView-несоответствия, размер ML-моделей)
- BSL-раскрытие upfront (Хабр технически подкован, обнаружит)

### Что НЕ работает

- ❌ Любая статья с продающим заголовком
- ❌ Превращение пользовательского аккаунта в blog компании
- ❌ Скрытие BSL
- ❌ LLM-сгенерированный контент (Хабр-аудитория чувствует)

### Цикл

- Публикация в Песочнице → модерация 1–7 дней → или публикация (статья видна), или invited в полноправные
- После полноправного аккаунта — статья публикуется сразу, но карма-рейтинг решает visibility

---

## 8. RuStore editorial (RU, Tier 1)

### Контакт

- **Заявка на фичеринг:** через **Консоль разработчика RuStore** (формы в дашборде)
- **Главный редактор:** Варвара Юмина (контакт через официальные каналы RuStore, не напрямую)
- **Blog:** rustore.ru/developer/blog (полезно читать перед pitch'ем)

### Процесс

Фичеринг **бесплатный**, не покупается. Отбор — по решению редакции, по совокупности факторов:

- Количество установок и активность пользователей
- Качество карточки приложения (иконки, скриншоты, описание)
- Работа с отзывами (разработчик отвечает)
- Интеграция с инструментами RuStore (Pay SDK, и т.д. — не обязательно, но повышает шансы)
- Внешний трафик на карточку (разработчик сам ведёт пользователей в RuStore)

Форматы:

- **Монофичеринг** — баннер одного приложения
- **Подборки** — карусель нескольких проектов
- **События на витрине** — акции, скидки, обновления
- **Редакторские метки** — «Выбор редакции», «Красивая графика»
- **Прокачница** — ежемесячный промо-ивент с офферами (нужен оффер: скидка/бонус/промокод)

### Реалистичная оценка для pre-launch Origa

Pre-launch Origa с минимальными установками и отзывами — **шансы на фичеринг слабые**. Редакция выбирает популярные или активно растущие проекты.

### Что можно сделать сейчас

1. Оформить карточку приложения в RuStore максимально качественно (иконки, скриншоты, описание на RU)
2. Регулярно отвечать на отзывы пользователей
3. Подать заявку на фичеринг через Консоль (форма для разработчиков)
4. Использовать связь: Origa уже auto-uploads AAB (#271) — написать в саппорт с предложением collab
5. Подготовить оффер для «Прокачницы» (если есть что-то для акции)

### Что НЕ работает

- ❌ Требовать фичеринг (это право редакции, не обязательство)
- ❌ Подавать заявку с пустой карточкой (нет скриншотов, нет описания)
- ❌ Игнорировать отзывы

### Цикл

- Заявка → редакция рассматривает (без public SLA, обычно недели)
- Если отказ — можно подать снова после улучшения метрик

---

## 9. vnjpclub.com (VI, Tier 1)

### Контакт

- **Email:** vnjpclubinfo@gmail.com (для support вопросов)
- **Email:** admin@vnjpclub.com (альтернатива)
- **Сайт:** vnjpclub.com (есть diễn đàn / форум)

### Процесс

vnjpclub — крупнейший VI JP-портал. У них есть:

- Сайт с уроками (Minna, Somatome, JLPT prep)
- Собственное app (на Google Play и App Store)
- Диễn đàn (forum) для общения пользователей

Это **напрямую конкурирующий продукт** (vnjpclub тоже имеет JP-app). Но у них diễn đàn — moderated community, где можно упомянуть Origa как alternative.

### Что работает

- Сообщение на вьетнамском (локализация — wedge)
- Уважительный framing: «дополнительный инструмент для тех, кто учит черезimmersion/content mining» — не конкуренция vnjpclub, а complement
- Предложение cross-promotion: Origa упоминает vnjpclub словари, vnjpclub упоминает Origa как app
- Зеркало: «Origa может импортировать колоды из других форматов» (если это правда)

### Что НЕ работает

- ❌ Pitch как конкурент vnjpclub app (они не опубликуют)
- ❌ Generic PR-pitch без понимания их аудитории

### Цикл

- Email — 3–7 дней
- Диễn đàn thread —.instant, но требует участия в дискуссии

### Tier-1 fit caveat

vnjpclub — **adjacent competitor**. Возможно переместить в Tier 2 (lower priority). Альтернатива для VI Tier 1:tokutei ginou FB groups (см. §11) — там нет конкуренции.

---

## 10. Minato Dorimu FB (VI, Tier 1)

### Контакт

- **FB Group:** "Minato Dorimu Nihongo" (65K+ участников, по их собственной marketing-странице)
- **Админ:** Ngọc Tiệp (основатель Minato, страница ngocptiepminato в FB)
- **Процесс:** FB Messenger DM через Members → Admins & Moderators list

### Процесс

Стандартный Facebook Group admin outreach:

1. Зайти в группу → tab «Members» → «Admins & Moderators»
2. Выбрать самого активного админа
3. Профиль → Message (через Messenger)
4. Альтернатива: кнопка «Message Admins» если есть (открывает чат со всеми админами)

### Что работает

- Сообщение на вьетнамском (локализация — wedge)
- Чёткое упоминание: «я член группы / я хочу поделиться инструментом, который поможет аудитории»
- Предложить дать review-лицензию модераторам для тестирования
- 70/30 rule: 70% value, 30% pitch — сначала помочь в дискуссиях, потом pitch
- Ссылка в **первом комменте**, не в теле поста (FB режет reach постов с внешними ссылками)

### Что НЕ работает

- ❌ First post = sales pitch (instant removal + возможен ban)
- ❌ Affiliate/referral link без контекста
- ❌ Cold-pitch в комментах чужих постов
- ❌ DM участникам группы без согласия
- ❌ Copy-paste identical promo в несколько групп
- ❌ Внешняя ссылка в теле поста

### Цикл

- DM админу: 24–48 часов
- После одобрения — пост в нужном формате (вероятно в weekly promo thread, если есть)

---

## 11. Tokutei ginou FB groups (VI, Tier 1)

### ⚠️ Требуется дополнительный research

**Проблема:** конкретные tokutei ginou FB-группы не идентифицированы в этой итерации research'а. Это **gap в strategy** — нужен отдельный research-pass.

### Что нужно сделать перед pitch'ем

1. Идентифицировать 3–5 конкретных tokutei ginou FB-групп:
   - Размер (участники)
   - Активность (посты/день)
   - Модерация (правила pinned?)
   - Язык (VI, JP, mixed?)
2. Для каждой: найти админа через Members → Admins list
3. Прочитать pinned rules (60-second check перед pitch'ем)

### Предположительные кандидаты (требуют верификации)

- "Cộng đồng Tokutei Ginou Việt Nam"
- "Tokutei Ginou Japan Vietnamese"
- Группы при крупных dispatch-компаниях (если есть)

### Why this matters for Origa

Tokutei ginou (特定技能) — это японская рабочая виза. VI-сообщества tokutei ginou — **прямое попадание в wedge Origa** (JLPT-required для визы, native VI UI, offline для работы в Japan без интернет-проблем). Это **высокий ROI** для VI рынка.

### Что работает (предположительно, после идентификации)

- Сообщение на вьетнамском
- Framing: «JLPT-prep app для тех, кто готовится к N4/N3 для tokutei ginou»
- Оффлайн — реально полезен в Japan (роуминг-интернет дорогой)
- Cross-promo с dispatch-компаниями если есть связи

---

## 12. DC Inside JLPT 마이너 갤러리 (KO, Tier 1)

### Контакт

- **Gallery:** https://gall.dcinside.com/mgallery/board?id=jlpt (или m.dcinside.com/board/jlpt для mobile)
- **Manager (매니저):** DJ.PEAR (по pinned threads)
- **Submit:** post directly in gallery (нужен 고정닥 DC Inside аккаунт)

### Процесс

1. **Зарегистрировать 고정닥** на DC Inside:
   - Зайти на dcinside.com → «고정닥 신청»
   - Создать 닉네임 (2-20 символов), пароль
   - Сохранить 식별 코드 и 보안 код (нельзя восстановить)
   - Если не заходить 11 месяцев → авто-выход

2. **Изучить культуру gallery** перед постингом:
   - Прочитать pinned threads (공지)
   - Понять format (홍보 / 일반 / 呟き🌸)
   - Посмотреть precedent posts (예: "일본어 한자 앱 홍보" — прецедент есть)

3. **Пост на корейском** — формат 홍보 (promotion):
   - Заголовок: «[홍보] 한국어 UI 일본어 학습 앱 Origa» или похожее
   - Body: что делает, чем отличается, ссылка на repo/landing, скриншоты
   - Проявить скромность — DC culture прямая, hype ratio'т

### Прецедент

В галерее есть пример поста «일본어 한자 앱 홍보 - 일본어 상용한자 외우기» (от 2025-01-24) — разработчик приложения анонсировал свой продукт. Это **доказывает что 홍보-формат принимается**, но культура прямоты означает что любой hype будет жестоко покритикован.

### Что работает

- Пост на корейском
- Честный framing: «я разработал, вот что делает, вот чем отличается»
- Anki-import angle — в gallery есть pinned Anki 통합 덱 thread с 160K+ просмотрами, Anki популярен
- Offline angle (для тех, кто работает в Japan без стабильного интернета)
- Native KO UI — прямой wedge

### Что НЕ работает

- ❌ Пост на английском (DC — KO-комьюнити)
- ❌ Hype / superlatives (DC culture прямая)
- ❌ Скрытие BSL (KO dev community проверит)
- ❌ Booster-comments от друзей (community замечает)

### Цикл

- Пост публикуется сразу
- Пик трафика: первые 12 часов
- 답글 (комменты) могут приходить днями

---

## 13. Naver Cafes (KO, Tier 1)

### ⚠️ Требуется идентификация конкретных cafes

**Проблема:** конкретные JP-learning Naver Cafes не идентифицированы. Naver Cafe — это KO community format, десятки тысяч cafes по интересам. Нужен research-pass для идентификации 3–5 конкретных cafes с размерами и культурой.

### Что нужно сделать перед pitch'ем

1. Идентифицировать 3–5 JP-learning Naver Cafes:
   - "일본어 카페", "JLPT 카페", "일본어 공부 카페" — общие поисковые запросы
   - Размер (10K–100K+ участников)
   - Активность (посты/день)
   - Правила (가입 신청 양식 — часто нужно подать заявку на членство с self-introduction)
   - Levels (Naver Cafe часто имеет level-system:新人 → 정회원 по активности)

2. Для каждой: 
   - Подать заявку на 가입 (часто с self-introduction)
   - Наработать уровень через полезные посты (10:1 rule)
   - Найти менеджера (카페 매니저) через members list
   - DM менеджеру перед любым промо

### Что работает (предположительно)

- Пост на корейском
- Уважение к level-системе (не постить промо как newcomer)
- Сначала contribute (10:1 rule — 10 полезных постов на 1 промо)
- Personal framing: «я разработал, вот мой опыт»
- Подчеркнуть: native KO UI, offline, FSRS

### Что НЕ работает

- ❌ Пост на английском
- ❌ First post = promo (instant ban в большинстве cafes)
- ❌ Нарушение level-system (новички не могут постить ссылки)
- ❌ Mass-paste identical promo в несколько cafes

### Цикл

- 가입 신청 → одобрение 1–7 дней
- Level-up через активность → недели
- Только потом — pitch менеджеру

### Tier-1 fit caveat

Naver Cafes требуют **длительного onboarding'а** (недели на наработку уровня). Это **не launch-day channel** — это долгосрочный community engagement. Возможно переместить в Tier 2 / Phase 2.

---

## 14. Универсальные принципы для community-платформ (FB, DC, Naver, Telegram)

### 60-second rules check перед любым постом

1. **Открыть Featured/pinned section** — найти rules
2. **Искать слова:** «promo», «self-promotion», «links», «vendors», «홍보», «приложения», «реклама»
3. **Понять формат:** banned / only in specific thread / on specific day / fine
4. **Note escalation policy:** first warning + post removal / instant ban / etc.

### 70/30 rule

- 70% активности — genuine value (полезные ответы, tips, обсуждение)
- 30% — promotion
- Применяется **per community over time**, не глобально

### Link в первом комменте, не в теле поста

- Facebook **режет reach** постов с внешними ссылками
- Reddit аналогично
- Solution: link в первом комменте автора, не в теле поста

### Varo каждый промо

- Никогда не copy-paste identical promo в несколько групп
- Меняй wording, ротируй images
- Identical posts trip как admin judgement, так и Facebook duplicate-content signals

### Personal voice

- «Я разработал Origa, потому что...»
- Не «Origa — лучший app для японского»
- Personal framing принимается, brand-framing — нет

---

## 15. Что не хватает (gaps для следующего research-pass)

1. **Tokutei ginou FB groups (VI)** — конкретная идентификация 3–5 групп с размерами и админами
2. **Naver Cafes (KO)** — конкретная идентификация 3–5 JP-learning cafes
3. **Nihongo (@yaponskoe) RU alternative** — найти бесплатный RU JP-канал взамен платного @yaponskoe
4. **Minato Dorumu FB admin** — верифицировать что Ngọc Tiệp досягаем через Messenger
5. **skerritt.blog email** — попробовать найти прямой контакт (вероятно через Twitter)
6. **MariaTheMillennial email** — верифицировать форму контакта
7. **Tofugu timing** — узнать когда у них следующий tools roundup (если планируется)
8. **Хабр корпоративный блог** — изучить процедуру регистрации компании (если Origa будет идти этим путём)

---

## 16. Outreach tracker — структура

Создать `marketing/playbooks/outreach-tracker.md` после HUMAN GATE. Колонки:

| Площадка | Контакт | Формат | Язык | Дата отправки | Статус | Followup #1 | Финал | Заметки |
|---|---|---|---|---|---|---|---|---|

Статусы: `pending` / `sent` / `opened` / `replied` / `declined` / `published` / `dropped`

---

## 17. Brand constraints (для outreach)

Унаследовано из `origa-distribution-platforms.md` §8 и `objections-handling.md`:

1. Никаких superlatives
2. Никакого marketing language на tech-площадках (HN, Хабр, Clien)
3. BSL раскрывается upfront (не «open source»)
4. Честные лимиты
5. Никакой дискредитации конкурентов
6. Personal voice, не brand-voice
7. Соответствие language-локали площадки

---

## 18. Factcheck status

Claims в этом документе из web research 2026-07-21. Все контакты верифицированы через публичные страницы платформ на дату research'а.

**Items requiring verification before outreach (per-platform):**

- Tofugu hello@tofugu.com — верифицировано через LinkedIn + GitHub + blog (high confidence)
- skerritt.blog Twitter handle — **требует верификации** (@autumn_skr предполагаемый)
- MariaTheMillennial форма — верифицировано (она сама просит review requests)
- Daigaku @OuroreS — верифицировано через описание канала
- Nihongo @yaponskoe как платный канал — верифицировано через описание канала
- Хабр правила рекламы — верифицировано через habr.com/ru/docs/help/rules/
- RuStore фичеринг бесплатный — верифицировано через rustore.ru/help/developers/advertising-and-promotion/featuring
- vnjpclub email — верифицировано через Google Play listing
- Minato Dorumu FB админ — **требует верификации через FB Messenger**
- DC Inside JLPT gallery manager DJ.PEAR — верифицировано через pinned threads
- Naver Cafe специфика — **требует идентификации конкретных cafes**

**External/unverified signals:**

- Все размеры аудиторий (Telegram subscribers, FB group members) — snapshot'ы на дату research'а, fluctuate
- Response times — оценки на основе отраслевых norms, не гарантированы
