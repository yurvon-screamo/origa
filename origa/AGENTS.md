# AGENTS.md - Origa Core (`origa` crate)

## Описание

Бизнес-логика приложения: доменные модели, use cases, traits, OCR, словарь.

## Структура проекта

```text
origa/src/
├── domain/           # Доменные модели и ошибки (thiserror)
├── use_cases/        # Business logic workflows
├── traits/           # Abstracts (Repository, OCR, etc.)
├── ocr/              # NDLOCR-Lite implementation
└── dictionary/       # Лингвистический модуль (lindera)
```

## Ключевые соглашения

### Обработка ошибок

```rust
// ✅ Хорошо: thiserror для доменных ошибок
#[derive(Debug, thiserror::Error)]
pub enum CardError {
    #[error("Card not found: {0}")]
    NotFound(ULID),
}

// ❌ Плохо: unwrap в production
card.unwrap();
```

### Типизация

- Все публичные функции должны иметь явные типы
- Никаких `()` вместо Result где возможна ошибка

### Логирование

- Используйте `tracing` для всех логов

```rust
tracing::info!("Creating card {id}");
tracing::error!("Failed to process OCR: {err}");
```

## Тестирование

```bash
# Все тесты крейта
cargo test -p origa

# Конкретный тест
cargo test -p origa --test test_name

# С выводом println
cargo test -p origa -- --nocapture
```
