# План: «Знаю»-сегодня гасит карту в канале компаньонов на день

Статус: ревью раунд 1 (not_ready, 1 High + 3 Medium + 6 Low) — правки внесены,
ждёт ревалидации.

## Контекст

Жалоба владельца (2026-09-20): «к сложной карте влетает компаньоном простой
кандзи и дальше приебывается пиздец со своими словами».

Механика `domain/knowledge/kanji_companions.rs` не смотрит на состояние памяти
вообще:

- **reverse** (`find_reverse_companions`): кандзи-карта колоды с совпадающим
  знаком из vocab-слова урока добавляется, если только JLPT ≤ уровня юзера;
- **forward** (`find_companion_cards`): до `MAX_COMPANION_WORDS` (3)
  популярных слов кандзи урока, существующих в колоде.

Кнопка «Знаю» (`MarkCardAsKnownUseCase` → `KnowledgeSet::mark_card_as_known`)
сидирует память: `MemoryState(stability 22, difficulty 3, review_date = вчера)`
+ `apply_review(Rating::Easy)`. Карта становится `is_known_card()`, но факт
отметки из истории не выводим: `apply_review` кладёт `last_review_date`
моментом вызова (неотличимо от настоящего Easy-ревью) и перетирается любым
следующим рейтингом; бэктик на день сидированного `MemoryState` сдвигает
`next_review_date` в прошлое (раньше due), а не прячет событие. Отсюда
необходимость явного поля-штампа.

## Решения владельца (зафиксированы)

1. **Трактовка «Знаю»: только сегодня.** Карта молчит в канале компаньонов
   ровно до конца календарного дня отметки; назавтра канал снова открыт.
   Альтернатива «пока stability > 21» отклонена сознательно.
2. **Слова известного кандзи-источника не фильтруются.** Когда известный
   кандзи законно пришёл в core по сроку FSRS (~раз в 3 недели), его
   популярные слова притягиваются как раньше.
3. **Повторное «Знаю» считается.** Семантика «сегодня юзер на них уже ставил
   "Знаю"» охватывает каждое нажатие, включая нажатие на уже известной карте:
   гард-скип use case не должен глушить штамп (High из ревью раунда 1).

## Контракт

### Поле

`MemoryHistory.marked_known_at: Option<DateTime<Utc>>` — таймстемп последней
отметки «Знаю».

- `#[serde(default)]` — legacy-данные десериализуются с `None`;
- мерж: LWW по таймстемпу (паттерн `ghost`: `Some` с более поздним временем
  побеждает, `None` + `Some` → `Some`, тай — `>=`, правая);
- ставится доменом `KnowledgeSet::mark_card_as_known`.

### Поведение mark_card_as_known (домен + use case)

Домен `mark_card_as_known(card_id)`:

- карта НЕ известна (`!is_known_card()`): сидировать память (как сейчас) И
  поставить штамп `marked_known_at = now`;
- карта УЖЕ известна (`is_known_card()`): **только обновить штамп** —
  памяти не трогаем (без повторного сидирования и `apply_review`:
  не раздуваем reps и не сбрасываем стабильность реальным ревью).

Use case `MarkCardAsKnownUseCase`: гвард `is_known_card → skip` УДАЛЯЕТСЯ
(его работу берёт на себя домен); осмысленный warn заменяется на обычный
debug-трейс пути (свежая отметка / обновление штампа известной карты).

### Предикат

`MemoryHistory::marked_known_today(now: DateTime<Utc>) -> bool`:
`marked_known_at` в том же календарном дне, что `now`. День = сравнение
`date_naive()` по UTC — тот же паттерн, что у `stats_tracker` (дневные лимиты).

### Фильтры компаньонов

`add_kanji_companions` получает параметр `now: DateTime<Utc>` (единственный
production-вызов — пайплайн `cards_to_lesson_with_policy`, передаёт
`Utc::now()`):

- reverse-кандидат (кандзи) скипается при `marked_known_today(now)`;
- forward-кандидат (слово) скипается при `marked_known_today(now)`;
- forward-источник (кандзи урока) НЕ фильтруется — решение владельца №2.

**Точка вставки forward-фильтра:** в проверке кандидата рядом с
`already_in_lesson`/`seen_companion_ids` (`kanji_companions.rs`, условие
кандидата в `find_companion_cards`). Отфильтрованный кандидат слот ПОТРЕБЛЯЕТ
без замещения из глубины списка — семантика ровно как у соседнего
`already_in_lesson` (тише для юзера: заглушённое слово не заменяется четвёртым
популярным).

Рука знакомства (`select_acquaintance_hand.rs::attach_kanji_companions`) не
трогается: её пул — строго новые карты (`memory().is_new()`), штамп на новой
карте by construction `None`, предикат всегда false.

### Тест-сьюит (cfg(test))

`MemoryHistory::set_marked_known_at_for_test(Option<DateTime<Utc>>)` —
доступ через существующий `StudyCard::memory_history_mut_for_test()`
(паттерн ghost: time-travel «вчера/сегодня» и сборка состояний отбора).

## Срезы

- **S0**: глоссарий (термин «Отметка „знаю“ [marked_known_at]», явное
  разведение с существующим «уже знаю» / `RatingContext::Implicit`) → поле +
  serde-дефолт + мерж LWW + предикат + `set_marked_known_at_for_test` +
  тесты слоёв 1–2 (unit + merge/back-compat).
- **S1**: ветвление домена `mark_card_as_known` (сидирование+штамп /
  только-штамп) + удаление гварда use case + тесты (включая повторную
  отметку: штамп обновился, память нетронута — reps/stability неизменны).
- **S2**: `now`-параметр `add_kanji_companions` (+механический ripple ~12
  тестовых вызовов) + оба фильтра + тесты отбора (слой 3), включая пин
  «marked-today карта, будучи due, всё равно входит в core».
- **S3**: journey-тест (слой 4): «Знаю» → карта молчит в компаньонах до конца
  дня → назавтра возвращается (time-travel сеттером штампа). Полные проверки:
  fmt, clippy `-D warnings`, `cargo test -p origa`.

## Тест-план

Имена по канону `<subject>_<condition>_<expected_result>`; 3+ кейсов с общим
инвариантом — один `#[rstest]` с именованными кейсами:

- слой 1 (rstest): `marked_known_today_<case>` — сегодня true / вчера false /
  None false / граница полуночи; serde round-trip legacy-JSON без поля;
- слой 2: мерж LWW (позже побеждает / None-пропагация / тай),
  `mark_card_as_known_seeds_memory_and_stamps`,
  `mark_card_as_known_on_known_card_refreshes_stamp_only` (память нетронута),
  `mark_card_as_known_leaves_active_ghost_intact`,
  use-case journey
  `repeat_mark_known_on_known_card_refreshes_stamp_and_keeps_memory_intact`;
- слой 3: `reverse_skips_kanji_marked_known_today`,
  `reverse_includes_kanji_marked_known_yesterday`,
  `forward_skips_word_marked_known_today`,
  `forward_words_of_known_source_kanji_still_attach` (пин решения №2),
  `marked_today_due_card_still_enters_core` (пин scope: фильтр только
  компаньоны);
- слой 4: journey `mark_known_silences_companion_same_day_returns_next_day`.

## Не трогаем

- `create_companion_vocab_cards` (авто-создание слов при создании кандзи —
  другой механизм, blocklist `deleted_companion_words` при нём);
- forward-источник (слова известного кандзи в core) — решение №2;
- руку знакомства;
- доки лендинга (компаньоны в `/docs/*` не описаны — проверено grep'ом;
  «companion» в блоге — про роль Origa, не про механику);
- UI/i18n (механика невидима).

## Риски

- **День по UTC, а не локальный:** у юзера глубокой ночью Азии отметка
  «сегодня» живёт до утра по локальному. Консистентно с дневными лимитами
  приложения (везде UTC) — принято.
- **Взаимодействие с ghost (#592):** компаньонские показы рейтятся
  `RatingContext::Explicit` и двигают лестницу добиваний; глушение
  известной-сегодня карты сокращает её Explicit-поток вне core — карта с
  Active-ghost продолжит лестницу через core/избранное. `mark_card_as_known`
  сам ghost не порождает (пин: существующий `mark_card_as_known_spawns_no_ghost`)
  и НЕ резолвит существующий Active-ghost (лестница живёт до 3 успехов/TTL —
  добавить пин `mark_card_as_known_leaves_active_ghost_intact`).
- **Штамп не сбрасывается ничем, кроме более поздней отметки:** поле
  персистится вечно (один `Option<DateTime>`, дешево); семантика «тот же день»
  делает старые значения инертными.
