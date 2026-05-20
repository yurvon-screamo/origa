# ADR-004: Использование оригинальной FSRS State Machine

## Status

Accepted

## Date

2026-05-20

## Context

В Origa при реконструкции FsrsCard из сохранённого MemoryState всегда использовался `state = FsrsState::Review`. Это пропускало FSRS Learning/Relearning стадии и формулу `short_term_stability`. При Again интервал форсировался в 0, игнорируя FSRS-результат.

### Проблемы

1. **Learning-стадия пропущена** — новые карточки сразу получали Review-интервалы (дни) вместо Learning-цикла (минуты)
2. **Relearning-стадия пропущена** — после Again карточка не входила в relearning loop
3. **short_term_stability не используется** — веса w[17]=0.5034, w[18]=0.6567 обучены, но бесполезны
4. **Again-override** — `interval=0` вместо FSRS-расчёта (1-5 минут)

## Decision

### Собственный CardState enum

Доменный `CardState` (New/Learning/Review/Relearning) без зависимости от rs-fsrs. Конверсия CardState ↔ FsrsState только в srs.rs.

### Убрать Again-override

Интервалы при Again теперь определяются FSRS полностью:

- New → Again → Learning (1 минута)
- Learning → Again → Learning (5 минут)
- Review → Again → Relearning (5 минут)

### Backward-compatible миграция

`#[serde(default)]` на CardState — все существующие данные получают Review.

## Alternatives Considered

### Использовать FsrsState напрямую

Нарушает Clean Architecture — доменный слой зависит от библиотеки FSRS.

### Оставить два FSRS-инстанса (short_term/long_term)

ShortTerm FSRS не нужен — FSRS state machine сама управляет Learning→Review переходами.

## Consequences

- FSRS использует полную state machine: Learning/Relearning с short_term_stability
- Интервалы при Again: минуты вместо zero
- CardState сохраняется в MemoryState — backward compatible
- 5 FSRS-инстансов сохранены для разных типов карт (vocab, phrase, grammar, kanji, short_term)
- UI dashboard (is_*/CardStatus) не затронут — CardState для FSRS scheduling, is_* для UI categorization
