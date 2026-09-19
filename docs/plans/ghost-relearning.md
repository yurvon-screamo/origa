# Добивания (ghost relearning) — план реализации

> **Статус:** план валидирован code-quality-reviewer (`ready`, раунд 2:
> 0 High / 0 Medium / 5 Low — остаточные Low закрыты попутно в тексте).
> Направление утверждено владельцем (диалог 2026-09-18).
> В реализацию не взят — ожидает команды владельца.
> Исходная ишью: <https://github.com/yurvon-screamo/origa/issues/592>.
> Код ещё не реализован; документ — опора для срезов S0–S4.

---

## 1. Суть

SRS слабо закрепляет «свежие» провалы: после `Again` FSRS укорачивает
интервал, но нет плотной консолидации конкретного проваленного пункта и
критерия «пункт добит». Фича вводит **добивания**: карта, провалившаяся
дважды подряд, закрепляется по распределённой лестнице показов
(12ч → 1д → 3д) до трёх успехов подряд. Научная основа — successive
relearning (Rawson & Dunlosky 2011/2013; Vaughn et al. 2016): критерий
3 успехов + 3 распределённые сессии.

Одновременно **удаляется существующий дубль-механизм** (`expand_repeated_views`):
massed-дубли внутри урока дают краткосрочный эффект, их работу целиком
забирает лестница добиваний; начальный критерий для новых карт уже закрыт
режимом знакомства (`docs/acquaintance-mode.md`).

## 2. Зафиксированные решения (утверждены владельцем)

| # | Решение |
|---|---|
| 1 | Scope: **все типы карт** (Vocabulary, Kanji, Grammar, Phrase) |
| 2 | Лестница: **12ч → 1д → 3д** (Origa поддерживает n-уроков в день) |
| 3 | Лимит: **без отдельного капа** — добивания = высший приоритет core-отбора внутри `MAX_LESSON_SIZE = 22`. Реально вытесняют high-difficulty due и due in_progress/known; новые карты в UI-уроках и так исключены политикой (`NewCardPolicy::Exclude`, `lesson/content.rs`), в Inject-пути вытеснение действует аналогично |
| 4 | Критерий успеха: `Good` (defensively `Good\|Easy`), провал — `Again` (`Again\|Hard`). UI бинарный (`rating_buttons.rs`), остальные значения — доменная полнота |
| 5 | 3 успеха подряд закрывают добивание **молча** — без UX-сигнала (не погружаем юзера в механики) |
| 6 | Спавн: **2×`Again` подряд** по явным рейтингам карты; гейт — карта прошла первое ревью (`!is_new`), знакомство (`seed`) не спавнит |
| 7 | Неявный двойной рейтинг (`handle_grammar_dual_rating`) **не трогает** добивания — только явные показы карты |
| 8 | Показ вне окна лестницы двигает только FSRS, добивание не меняет |
| 9 | TTL: **30 дней с последнего шага** — списание брошенного; режима каникул нет, TTL — единственный сборщик; фриз часов добавим при появлении режима паузы |
| 10 | Офлайн-мерж: счётчики по `max()`, состояние LWW по времени — неточность при дивергенции принята (та же политика, что `reps`/`lapses`) |
| 11 | Дубль-механизм `expand_repeated_views` **удаляется целиком**; high-difficulty приоритет *отбора* сохраняется |
| 12 | FSRS-планировщик не модифицируется: один рейтинг двигает FSRS и добивание независимо |

Взаимодействие с рукой знакомства: ghost-карты — обычные due-долги и
участвуют в `due_debt_capacity_exceeded` (`select_acquaintance_hand.rs`)
по действующим правилам — статус-кво, поведение не меняется (зафиксировано,
чтобы не «переоткрыть» при реализации S3).

## 3. Архитектура

```text
origa/src/domain/memory/ghost.rs          — НОВОЕ: GhostState + машина состояний
origa/src/domain/memory/mod.rs            — MemoryHistory: поля + мерж (неразделимо, S0)
origa/src/domain/knowledge/mod.rs         — rate_card: точка переходов (гейт явности)
origa/src/domain/knowledge/lesson_builder/selection.rs — рука добиваний
origa/src/domain/knowledge/lesson_builder/expansion.rs — УДАЛЯЕТСЯ
origa/src/domain/knowledge/lesson_builder/mod.rs       — пайплайн без expansion
origa/src/use_cases/rate_card*.rs         — контекст явности рейтинга
origa_ui/                                 — без нового UI (рука невидима)
```

Персистенция/синк — бесплатно: ghost живёт в `MemoryHistory` →
сериализуется со `StudyCard` → IndexedDB/sync/merge на существующих рельсах.
**Поля и правила мержа вводятся одним срезом (S0)** — сериализационная
совместимость и merge-совместимость не расходятся во времени.

### 3.1 Контракт GhostState

```rust
/// Шаг лестницы добивания. Позиция в лестнице одновременно кодирует
/// число накопленных успехов: First = 0 успехов, Second = 1, Third = 2;
/// успех на Third — третий подряд — закрывает добивание. Отдельного поля
/// successes нет: rung — единственный источник истины.
pub enum GhostRung { First, Second, Third }   // интервалы: 12ч, 1д, 3д

pub enum GhostState {
    Active {
        rung: GhostRung,
        due_at: DateTime<Utc>,             // «окно»: шаг засчитывается при due_at <= now
        last_transition_at: DateTime<Utc>, // база TTL (30 дней)
    },
    Resolved { at: DateTime<Utc> },        // терминальный: «добито» (маркер для LWW-мержа)
    Expired  { at: DateTime<Utc> },        // терминальный: списано по TTL (для LWW)
}

// MemoryHistory (все новые поля #[serde(default)] — обратная совместимость):
//   consecutive_again: u8   — предспавновый счётчик подряд-провалов
//   ghost: Option<GhostState>
```

### 3.2 Машина состояний (чистые функции, инъекция времени)

`apply_ghost_transition(&mut MemoryHistory, rating, now)` — вызывается
ТОЛЬКО из пути явного показа (§3.3). Правила:

- **Предспавн** (ghost не Active): `Again|Hard` → `consecutive_again += 1`;
  спавн при `consecutive_again >= 2 && !is_new && ghost != Active`
  (терминальные `Resolved`/`Expired` **замещаются** новым спавном —
  повторная серия провалов легитимно начинает новый цикл);
  спавн: `Active { rung: First, due_at: now+12ч, last_transition_at: now }`,
  `consecutive_again = 0`;
  `Good|Easy` → `consecutive_again = 0`.
- **Active, окно открыто** (`due_at <= now`) — любой шаг обновляет
  `last_transition_at = now` (база TTL):
  `Good|Easy` → переход на следующую ступень: `First → Second` с
  `due_at = now + 1д`, `Second → Third` с `due_at = now + 3д` (интервал
  **новой** ступени); успех на `Third` — третий подряд — → `Resolved { at: now }`;
  `Again|Hard` → `rung = First`, `due_at = now+12ч` (лестница рестартует,
  добивание живёт);
  любой терминальный переход (`Resolved`/`Expired`) → `consecutive_again = 0`.
- **Active, окно закрыто и TTL не истёк**: состояние добивания не меняется
  (FSRS обновляется). При истёкшем TTL ленивая проверка в начале
  `apply_ghost_transition` списывает добивание до веток окна — состояние
  меняется на `Expired` (см. TTL).
- **TTL (лениво, при отборе и рейтинге)**: `now - last_transition_at > 30д` →
  `Expired { at: now }`, `consecutive_again = 0`.
- **Терминальные состояния** переходов не совершают; единственный выход —
  новый спавн после нового накопления серии (см. предспавн).
- **Сбросы `consecutive_again` = 0**: любой `Good|Easy`, спавн, вход в
  `Resolved`/`Expired`. Иначе: один `Again` сразу после списания по TTL
  порождал бы мгновенный ре-спавн от «хвостов» старой серии.

Трассинг lifecycle-событий (правило проекта — только `tracing`, домен
уже использует): `ghost_spawned`, `ghost_advanced`, `ghost_restarted`,
`ghost_resolved`, `ghost_expired` — структурированные записи с `card_id`
и `rung`. Это же — источник наблюдения за механикой после релиза (вместе
с долей `Again` в `DailyHistory`); отдельных продуктовых метрик не вводим.

### 3.3 Рейтинг: гейт явности

`KnowledgeSet::rate_card` получает контекст `RatingContext::Explicit |
Implicit`. Ghost-переходы применяются только при `Explicit`. Сценарий 4
решается архитектурно: неявный путь физически обходит ghost-переходы.

**Исчерпывающий аудит писателей `MemoryHistory`** (все call-sites
`apply_review`/`seed`, не только `RateCardUseCase`):

| Путь | Контекст | Обоснование |
|---|---|---|
| UI урока → `RateCardWithSideEffectsUseCase::execute` (primary) | **Explicit** | явный показ карты |
| `handle_grammar_dual_rating` (карта правила) | **Implicit** | неявный рейтинг со слова-мутации (сценарий 4) |
| `mark_card_as_known` («Уже знаю» онбординга, прямой `card.apply_review(…, Easy)`) | **Implicit** | рейтинга-показа нет; путь заводит готовые карты в SRS. Сегодня new-only — гейт `!is_new` закрывает; разметка defensive + тест-защита от рефакторинга |
| `RateMode::OnboardingScoring` | **Explicit** | путь не вызывается из UI (только конфиг/статистика/тесты), разметка по смыслу режима |
| `complete_acquaintance_hand` (`seed`) | **Implicit** | карта до первого ревью; гейт `!is_new` и так защищает, разметка defensive |

Смена сигнатуры `RateCardUseCase::execute` (параметр контекста) затронет
~15 существующих вызовов в journey-тестах (`learning_lesson.rs`,
`learning_short_term.rs`, `yesno_journey.rs`) — масштаб правки учтён в S1.

### 3.4 Рука добиваний [GhostHand] (отбор)

В `build_lesson_core` порядок заполнения бюджета `MAX_LESSON_SIZE`:

1. Избранные (как сегодня, pin).
2. **Рука добиваний**: карты с `ghost = Active`, не истёкшим TTL и открытым
   окном (`due_at <= now`, после ленивой TTL-проверки), сортировка по
   `due_at` asc (наиболее просроченные первыми).
3. High-difficulty due → 4. Новые (дневной лимит) → 5. Due in_progress/known
   (все как сегодня).
6. Padding до `MIN_LESSON_SIZE` — только из high-difficulty без активного
   добивания.

Инварианты отбора:

- **Предикат исключения из core/padding** (`расширение is_core_candidate`):
  карта исключается, если `ghost = Active` **и** TTL не истёк
  (`now - last_transition_at <= 30д`) — TTL-условие входит в предикат,
  иначе списанная карта навсегда выпала бы из пайплайна. Единственный
  канал входа — рука (или избранное).
- Закрытое окно → карта отсутствует **из руки/core/padding** (избранное —
  намеренное исключение: pin остаётся, сценарий 8).
- Favorite + ghost: дедуп — карта входит как избранное один раз; показ
  засчитывается в лестницу, если окно открыто.
- Рука входит в общий бюджет (favorites + рука + core ≤ 22), без отдельного
  капа; переполнение — старейшие первыми, хвост ждёт (сценарий 11).
- Карты руки проходят стандартный пайплайн: views из обычного генератора
  (включая `Phrase` — `select_phrase_view` поддерживает не-новые;
  `add_phrases` дедупит против in_lesson — задвоения фраз не будет),
  interleave, шаффл. Невидимы для юзера (сценарий 16).

### 3.5 Удаление дублей

- `expansion.rs` удаляется целиком (`expand_repeated_views`,
  `target_showings`, `MIN_REPEAT_SPACING`, `compute_expansion_views`,
  `drain_pending`, `distribute_pending_with_spacing`).
- `build_lesson_core` больше не возвращает `primary_card_ids` (единственный
  потребитель — expansion; проверено grep-ом).
- `spacing.rs`: импорт `MIN_REPEAT_SPACING` из expansion — константа
  переезжает в `spacing.rs` либо проход упрощается. Сам проход
  `redistribute_core_for_spacing` СОХРАНЯЕТСЯ: он пере-выводит инвариант
  «фраза после слова» (PR #203) и разводит одинаковые `card_id`. После
  удаления дублей его именная миссия (spacing повторов) вырождается в
  re-placement фраз — **осознанный остаточный вес**, зафиксирован как
  кандидат на будущее упрощение отдельным PR (не scope этой работы).
- Тесты expansion удаляются вместе с механизмом; добавляется инвариант
  `no_card_appears_twice_in_lesson`.

## 4. Срезы

Порядок устраняет окна деградации: удаление дублей **до** руки (иначе
ghost-карта, почти всегда high-difficulty, получала бы massed-дубль — ровно
то, что объявлено вредным), правила мержа **вместе** с полями (иначе
кросс-девайс-синк в окне S0→SN терял бы состояние добиваний).

### S0 — Глоссарий + контракт + машина состояний + мерж

- Термины в `origa/docs/glossary.md` ДО кода: **Добивание [GhostState]**,
  **Ступень [GhostRung]**, **Окно [ghost window, due_at]**,
  **Счётчик подряд-провалов [consecutive_again]**, **Спавн [ghost spawn]**,
  **Списание [GhostState::Expired]**, **Рука добиваний [GhostHand]**,
  **Контекст рейтинга [RatingContext: Explicit | Implicit]**.
- `domain/memory/ghost.rs`: `GhostRung`, `GhostState`, интервалы ступеней
  (12ч/1д/3д), TTL-константа; чистые функции переходов с инъекцией `now`;
  tracing-события lifecycle.
- `MemoryHistory`: поля `consecutive_again`, `ghost` (`#[serde(default)]`);
  метод `apply_ghost_transition(rating, now)`; **правила мержа сразу**:
  `consecutive_again` — `max()`, `ghost` — LWW по
  `last_transition_at`/`at` (терминальные маркеры участвуют, resolve/expired
  не теряются при дивергенции).

Критерии приёмки: все переходы §3.2 покрыты юнит-тестами (§5, слой 1),
включая ре-спавн после `Resolved`/`Expired` и сбросы счётчика; тесты мержа
(слой 2); тест back-compat десериализации (fixture без новых полей →
`ghost == None`, `consecutive_again == 0`); `cargo test -p origa` зелёный.

### S1 — Интеграция рейтинга

- `RatingContext` (Explicit/Implicit) через `rate_card` до `MemoryHistory`.
- Разметка всех путей по таблице §3.3 (аудит call-sites `apply_review`/`seed`
  grep-ом, включая `mark_card_as_known`); правка ~15 вызовов
  `RateCardUseCase::execute` в journey-тестах.
- TTL-проверка лениво в начале `apply_ghost_transition`.

Критерии приёмки: journey-тесты §5 слой 4 (спавн → лестница → резолв;
рестарт; dual rating не трогает; квиз на собственной грамматике — ровно
один переход; показ вне окна — только FSRS; `mark_card_as_known` не спавнит).

### S2 — Удаление дублей (до руки)

- Удаление `expansion.rs` + шага пайплайна + `primary_card_ids`;
  починка `spacing.rs`; удаление тестов дублей; инвариант-тест уникальности.
- Обновить устаревший факт в базе знаний: `lesson-repeat-count-by-status`
  → «любая карта ≤ 1 показа за урок; закрепление провалов — добивания».

Критерии приёмки: `grep -rn "expand_repeated_views\|target_showings"` пуст
вне git-истории; clippy/fmt/test зелёные; wasm-тесты урока скорректированы
(если завязаны на дубли).

### S3 — Рука добиваний в отборе

- `collect_ghost_hand` + интеграция в `build_lesson_core` (приоритет 2),
  расширение `is_core_candidate` предикатом исключения с TTL-условием,
  дедуп с favorites, ленивый TTL при отборе.
- Проверить: phrase-карта в ядре проходит `redistribute_core_for_spacing`
  (partition фраз корректен) — тест.

Критерии приёмки: тесты слоя 3 §5 зелёные; бюджет урока соблюдён
(favorites + рука + core ≤ 22).

### S4 — Доки лендинга (Definition of Done, тот же PR что код)

- `content/docs/{en,ru,ko,vi}/fsrs.md`: качественное дополнение к абзацу о
  составе урока — проваленный материал систематически возвращается в
  ближайшие уроки до закрепления. **Без чисел лестницы** (меняющиеся
  параметры под запретом tone-rules), без упоминания конкурентов, без
  описания внутренних механик.
- `lesson.md` — если описывает состав урока (проверить все 4 локали).
- `limitations.md` — без изменений (бинарность рейтинга уже задокументирована
  и согласована с критерием успеха).

## 5. Тест-план

Имена — канон `<subject>_<condition>_<expected_result>`, термины из глоссария.

**Слой 1 — машина состояний** (`ghost.rs`):
`second_consecutive_again_spawns_ghost_at_first_rung`,
`good_between_two_agains_resets_consecutive_counter`,
`easy_counts_as_success_and_hard_counts_as_failure` (rstest, 4 рейтинга),
`good_on_open_window_advances_to_next_rung` (rstest: 12ч→1д, 1д→3д),
`success_when_window_closed_is_ignored`,
`third_consecutive_good_resolves_ghost`,
`resolved_ghost_is_terminal_further_ratings_change_no_ghost_state`,
`respawn_after_terminal_state_starts_new_cycle`
(rstest: после `Resolved` и после `Expired`),
`single_again_after_ttl_expiry_does_not_respawn`,
`again_at_second_rung_resets_streak_and_restarts_from_first_rung`,
`ladder_never_skips_or_exceeds_third_rung`,
`ghost_untouched_30_days_is_written_off`,
`ghost_at_29_days_23h_stays_active`,
`any_transition_resets_ttl_clock`,
`consecutive_again_resets_on_spawn_and_terminal_transition`.

**Слой 2 — MemoryHistory**: `rating_moves_fsrs_and_ghost_independently`
(MemoryState с Active-ghost == без него),
`merge_takes_ghost_counters_via_max_and_state_via_lww`,
`merge_preserves_ghost_when_other_side_has_none`,
`merge_terminal_marker_wins_over_stale_active`,
`seed_from_acquaintance_spawns_no_ghost`,
`legacy_serialized_history_deserializes_with_default_ghost_fields`
(back-compat, S0).

**Слой 3 — отбор** (`lesson_builder/tests.rs`):
`ghost_cards_fill_lesson_before_high_difficulty_and_new`,
`ghost_overflow_enters_oldest_first_rest_waits_next_lesson`,
`ghost_with_closed_window_absent_from_hand_core_and_padding`,
`expired_ghost_card_returns_to_normal_core_selection`,
`ghost_card_enters_once_via_hand_not_via_core`,
`favorite_and_ghost_deduped_to_single_showing`,
`ghost_selection_applies_to_all_card_types` (rstest, 4 типа),
`after_expansion_removal_no_card_shows_twice`,
`ghost_card_gets_views_from_standard_generator`,
`ghost_phrase_card_places_via_hand_respecting_phrase_after_word`.

**Слой 4 — journeys** (`use_cases/tests/journeys/ghost_lifecycle.rs`):
`twice_failed_card_walks_ladder_and_returns_to_normal_pipeline`,
`again_at_third_rung_restarts_ladder`,
`abandoned_ghost_written_off_and_card_returns_to_pipeline`,
`dual_rating_does_not_touch_grammar_ghost`,
`showing_outside_window_updates_fsrs_only`,
`quiz_on_own_grammar_card_transitions_ghost_exactly_once`,
`mark_card_as_known_spawns_no_ghost`.

**Слой 5 — UI/wasm**: множество `data-testid` экрана урока не изменяется
относительно бейзлайна (снапшот существующих test-id до/после — добивания
не добавляют разметку, сценарий 16); существующие wasm-тесты урока зелёные
после S2.

Соответствие Gherkin-сценариям — таблица в Приложении А.

## 6. Риски и компромиссы

| Риск | Митигация |
|---|---|
| Кросс-версионный синк: старый клиент перезапишет данные без ghost-полей | Принято: стандартно для аддитивных полей; окно между обновлениями короткое, потеря = повторный спавн при новых провалах |
| `spacing.rs` сцеплен с константой expansion | S2 явно разбирает; проход сохраняется (инвариант фраз), вырождение задокументировано (§3.5) |
| Phrase в ядре урока — новый путь для фраз | Тест `ghost_phrase_card_places_via_hand…`; генератор views уже поддерживает; `add_phrases` дедупит |
| «Окно открыто в момент рейтинга» vs «в момент показа» (секунды) | Принято: проверка `due_at <= now` на момент рейтинга, семантически эквивалентно |
| Раздувание `MemoryHistory` в wire-формате | Терминальные маркеры — один на карту; ADR-034 не затронут |
| Регресс от удаления дублей для high-difficulty карт без подряд-провалов | Принято владельцем: научное обоснование в §1; наблюдение — tracing-события lifecycle + доля Again в DailyHistory (§3.2) |

## 7. NOTICED BUT NOT TOUCHING

- Факт `lesson-repeat-count-by-status` в базе знаний устарел ещё до этой
  работы (код даёт max 2 показа, факт говорил 3/2/1) — обновляется в S2.
- Ресёрч-док `docs/research-grammar-relearning-ghosts-2026-09.md` из ишью
  №592 отсутствует в репо — закоммитить владельцу отдельно (не scope плана).
- Leech-подобная статистика (подряд-спавны после TTL) — возможное будущее
  расширение, не входит.
- Каникулы/фриз часов — нет механизма паузы; добавить при его появлении.
- Упрощение `redistribute_core_for_spacing` после вырождения — отдельный
  будущий PR (§3.5).
- e2e (Playwright) на добивания — после стабилизации механики, отдельным
  PR (end2end сегодня не покрывает SRS-состояния).

## Приложение А — Gherkin-сценарии (утверждены владельцем)

```gherkin
Функция: Добивания — закрепление проваленных карт

  Сценарий 1: Спавн добивания
    Допустим карта уже прошла первое ревью (не новая)
    И пользователь ответил Again на неё два раза подряд
    Когда применяется второй Again
    Тогда у карты появляется активное добивание на ступени 1
    И следующий показ по добиванию назначен через 12 часов

  Сценарий 2: Good между провалами срывает спавн
    Допустим пользователь ответил Again на карту
    Когда пользователь отвечает Good на её следующий показ
    Тогда счётчик подряд-провалов сбрасывается и добивание не появляется

  Сценарий 3: Карта до первого ревью не спавнит добивание
    Допустим карта новая или находится в режиме знакомства
    Когда пользователь дважды отвечает Again в тренировке знакомства
    Тогда добивание не появляется

  Сценарий 4: Неявный двойной рейтинг не трогает добивания
    Допустим у грамматического правила есть активное добивание
    Когда пользователь отвечает Good на слово-мутацию этого правила
    Тогда FSRS карты правила обновляется через двойной рейтинг как обычно
    Но добивание правила не продвигается и не сбрасывается

  Сценарий 5: Успех продвигает лестницу
    Допустим у карты активное добивание на ступени 1 (12ч) и окно наступило
    Когда пользователь отвечает Good
    Тогда добивание переходит на ступень 2 и следующий показ — через 1 день

  Сценарий 6: Три успеха подряд закрывают добивание молча
    Допустим у карты добивание на ступени 3 (3д) и уже 2 успеха подряд
    Когда пользователь отвечает Good на показ из руки добиваний
    Тогда добивание закрывается без какого-либо специального сигнала
    И карта возвращается в обычный пайплайн урока

  Сценарий 7: Again на добивании рестартует лестницу
    Допустим у карты добивание на ступени 3 и 2 успеха подряд
    Когда пользователь отвечает Again
    Тогда прогресс успехов сбрасывается: лестница рестартует со ступени 1
    И добивание остаётся активным

  Сценарий 8: Показ вне окна лестницы не двигает добивание
    Допустим у избранной карты активное добивание, окно ещё не наступило
    Когда карта показывается как избранная и получает Good
    Тогда FSRS карты обновляется как обычно
    Но состояние добивания не меняется

  Сценарий 9: Рука добиваний — высший приоритет урока
    Допустим у пользователя 5 карт с активными добиваниями с наступившим окном
    Когда собирается урок
    Тогда эти карты входят в урок с высшим приоритетом
    И при нехватке бюджета вытесняют прочий материал, а не наоборот
    И отдельного дневного лимита нет — ограничитель только MAX_LESSON_SIZE

  Сценарий 10: Дубль-механизм удалён
    Допустим карта является сложной (high-difficulty) без активного добивания
    Когда собирается урок
    Тогда карта входит в урок один раз
    И этап дублирования expand_repeated_views больше не существует

  Сценарий 10a: Карта с добиванием — единственный показ через руку
    Допустим у карты активное добивание
    Когда собирается урок
    Тогда карта входит в урок один раз — через руку добиваний
    И исключена из core-отбора

  Сценарий 11: Переполнение руки переносится на следующий урок
    Допустим добиваний с наступившим окном больше, чем влезает в урок
    Когда собирается урок
    Тогда входят старейшие добивания, остальные ждут без потери состояния

  Сценарий 12: TTL списывает брошенное добивание
    Допустим 30 дней не было ни одного шага добивания
    Когда срабатывает проверка TTL
    Тогда добивание списывается без последствий для FSRS-состояния карты
    И карта возвращается в обычный пайплайн урока

  Сценарий 13: FSRS не трогается
    Допустим у карты активное добивание
    Когда пользователь отвечает на любой показ карты
    Тогда FSRS-состояние обновляется по действующим правилам без модификаций

  Сценарий 14: Все типы карт участвуют
    Когда карта любого типа (слово, кандзи, грамматика, фраза)
    дважды подряд проваливается после первого ревью
    Тогда у неё появляется добивание по общим правилам

  Сценарий 15: Офлайн-мерж не теряет добивания критично
    Допустим одно и то же добивание менялось на двух устройствах офлайн
    Когда knowledge set мержится
    Тогда счётчики мержатся по max(), состояние — LWW по времени
    И возможная неточность подряд-счётчика при дивергенции принята

  Сценарий 16: Добивания невидимы в UI
    Допустим в уроке есть карты с активными добиваниями
    Когда пользователь проходит урок
    Тогда эти карты неотличимы от обычных: те же views, кнопки, поведение
    И никакая разметка «это добивание» не отображается

  Сценарий 17: Порядок заполнения бюджета урока
    Допустим у пользователя 3 избранного, 5 добиваний с открытым окном
    и достаточно сложных/новых/due карт
    Когда собирается урок (бюджет 22)
    Тогда входят 3 избранного, 5 добиваний (наиболее просроченные первыми)
    и остаток слотов — по действующим приоритетам core

  Сценарий 18: Карта с закрытым окном не входит в урок
    Допустим у карты активное добивание, окно не наступило
    Когда собирается урок
    Тогда карта не входит ни через руку добиваний, ни через core-отбор,
    ни через padding (избранное — исключение: pin остаётся, сценарий 8)
```

### Таблица соответствия: сценарий → тесты

| Сценарий | Тесты |
|---|---|
| 1 | слой 1 `second_consecutive_again_spawns…`; journey `twice_failed_card…` |
| 2 | слой 1 `good_between_two_agains…` |
| 3 | слой 2 `seed_from_acquaintance_spawns_no_ghost`; journey `mark_card_as_known_spawns_no_ghost` |
| 4 | journey `dual_rating_does_not_touch_grammar_ghost` |
| 5 | слой 1 `good_on_open_window_advances_to_next_rung` |
| 6 | слой 1 `third_consecutive_good_resolves_ghost`; journey `twice_failed_card…` |
| 7 | слой 1 `again_at_second_rung_resets…`; journey `again_at_third_rung_restarts_ladder` |
| 8 | слой 3 `favorite_and_ghost_deduped…`; journey `showing_outside_window_updates_fsrs_only` |
| 9, 17 | слой 3 `ghost_cards_fill_lesson_before…` |
| 10 | слой 3 `after_expansion_removal_no_card_shows_twice` |
| 10a | слой 3 `ghost_card_enters_once_via_hand_not_via_core` |
| 11 | слой 3 `ghost_overflow_enters_oldest_first…` |
| 12 | слой 1 `ghost_untouched_30_days…`; journey `abandoned_ghost_written_off…`; слой 3 `expired_ghost_card_returns_to_normal_core_selection` |
| 13 | слой 2 `rating_moves_fsrs_and_ghost_independently` |
| 14 | слой 3 `ghost_selection_applies_to_all_card_types` |
| 15 | слой 2 `merge_takes_ghost_counters…`, `merge_terminal_marker_wins…` |
| 16 | слой 3 `ghost_card_gets_views_from_standard_generator`; слой 5 снапшот test-id |
| 18 | слой 3 `ghost_with_closed_window_absent_from_hand_core_and_padding` |
