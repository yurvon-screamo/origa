# AGENTS.md - Origa AI Assistant Guide

## Project Overview

Origa — приложение для изучения японского языка с интервальными повторениями (FSRS), OCR и токенизацией. Tech stack: Rust workspace с крейтами origa (бизнес-логика), origa_ui (Leptos/WASM frontend), tauri (Tauri v2 desktop), tokenizer. Architecture: Clean Architecture с Use Cases, Domain, Traits.

## Quick Start Commands

### Development

```bash
# Frontend dev сервер (origa_ui)
cd origa_ui && trunk serve

# Tauri dev (desktop приложение)
cd tauri && cargo tauri dev
```

### Build

```bash
# Frontend production build
cd origa_ui && trunk build --release

# Tauri production build (native)
cd tauri && cargo tauri build

# Docker build (web)
docker build -f origa_ui/Dockerfile -t origa:latest .
```

### Testing

```bash
# Все тесты workspace
cargo test --workspace

# Тесты конкретного крейта
cargo test -p origa

# Один тест (по имени)
cargo test -p origa test_name

# Тесты с выводом
cargo test --workspace -- --nocapture

# Coverage report
cargo llvm-cov --workspace --html
```

### Linting & Formatting

```bash
# Clippy (linting)
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --check

# Format fix
cargo fmt
```

### E2E Testing

```bash
# Установка (первичный запуск)
cd end2end
npm install
npx playwright install

# Запуск тестов
npm test                    # Все тесты (headless)
npm run test:ui            # С UI Playwright
npm run test:headed        # В видимом браузере
npm run test:debug         # Режим отладки
npm run report             # Просмотр отчёта

# Для CSR-проектов (запуск вручную)
# Терминал 1: trunk serve
cd origa_ui && trunk serve

# Терминал 2: playwright tests
cd end2end && npm test
```

### Environment Variables для E2E

Скопировать `.env.example` в `.env` и настроить:
- `TRAILBASE_URL` — URL TrailBase API (default: `https://origa.uwuwu.net`)
- `ADMIN_EMAIL` — Email админа (default: `admin@localhost`)
- `ADMIN_PASSWORD` — Пароль админа (обязателен для создания пользователей)

Тестовые пользователи (hardcoded):
- `TEST_USER_EMAIL=e2e-test@origa.local`
- `TEST_USER_PASSWORD=e2e-test-password-123`

## Code Style & Conventions

### Imports

```rust
// ✅ Правильно: внешние крейты первыми, группировка в {}
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{debug, info};

// Затем внутренние модули через crate::
use crate::domain::OrigaError;
use crate::traits::UserRepository;

// ❌ Неправильно: внутренние модули первыми
use crate::domain::OrigaError;
use serde::{Deserialize, Serialize};
```

### Naming Conventions

| Элемент | Конвенция | Пример |
|---------|-----------|--------|
| Struct/Enum | `PascalCase` | `RateCardUseCase`, `OrigaError` |
| Enum variants | `PascalCase` | `UserNotFound`, `CardNotFound` |
| Functions | `snake_case` | `find_by_id()`, `rate_card()` |
| Variables | `snake_case` | `user_id`, `card_id` |
| Constants | `SCREAMING_SNAKE_CASE` | `KANJI_DICTIONARY` |
| Trait | `PascalCase` | `UserRepository` |

### Formatting

Rustfmt default. Проверить: `cargo fmt --check`. Исправить: `cargo fmt`.

### Error Handling

Единый enum `OrigaError` для всех ошибок проекта:

```rust
// ✅ Правильно: named fields для контекста
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrigaError {
    UserNotFound { user_id: Ulid },
    CardNotFound { card_id: Ulid },
    InvalidQuestion { reason: String },
}

// Использование в use cases
let user = self.repository
    .find_by_id(user_id)
    .await?
    .ok_or(OrigaError::UserNotFound { user_id })?;

// ❌ Неправильно: tuple variants без контекста
pub enum OrigaError {
    UserNotFound(Ulid),
}
```

### Use Case Pattern

```rust
// ✅ Стандартный паттерн UseCase
#[derive(Clone, Copy)]
pub struct RateCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> RateCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        card_id: Ulid,
        mode: RateMode,
        rating: Rating,
    ) -> Result<(), OrigaError> {
        // business logic here
    }
}
```

### Trait Definitions (Async)

```rust
// ✅ Использовать impl Future вместо #[async_trait]
pub trait UserRepository {
    fn find_by_id(&self, user_id: Ulid) 
        -> impl Future<Output = Result<Option<User>, OrigaError>>;
    fn save(&self, user: &User) 
        -> impl Future<Output = Result<(), OrigaError>>;
}
```

### Comments & Documentation

```rust
// TODO: для незавершённой работы
// TODO: Implement import_anki_pack

// Комментарии только если код неочевиден
// Код должен быть самодокументируемым через понятные имена

// ❌ Не использовать doc comments (///) без необходимости
```

## Project Structure

```
origa/
├── origa/                    # Основной крейт с бизнес-логикой
│   └── src/
│       ├── domain/           # Domain layer (entities, value objects)
│       │   ├── dictionary/   # Словари (kanji, vocabulary, radical)
│       │   ├── knowledge/    # Knowledge entities (cards, progress)
│       │   ├── grammar/      # Грамматика
│       │   ├── memory/       # FSRS алгоритм
│       │   └── error.rs      # OrigaError
│       ├── use_cases/        # Application layer (use cases)
│       │   └── tests/        # Тесты с fixtures и journeys
│       ├── traits/           # Repository traits
│       └── ocr/              # OCR модуль (ONNX)
├── origa_ui/                 # Leptos frontend (WASM/CSR)
│   └── src/
│       ├── app.rs            # Главный компонент + AuthContext
│       ├── components/       # Общие компоненты
│       ├── pages/            # Страницы приложения
│       ├── routes.rs         # Роутинг
│       └── repository/       # TrailBase client, repositories
├── tauri/                    # Tauri v2 desktop application
│   ├── src/                  # Tauri commands и setup
│   └── tauri.conf.json       # Tauri конфигурация
└── utils/                    # Support-утилиты проекта
```

## Testing Principles (Владимир Хориков)

Тесты следуют принципам из "Unit Testing Principles, Practices, and Patterns":

### ✅ Обязательные принципы

1. **Черный ящик** — тестировать только публичный API, не private методы
2. **Устойчивость к рефакторингу** — тесты не должны ломаться при изменении реализации
3. **Изоляция** — использовать InMemoryRepository вместо mock-библиотек
4. **AAA паттерн** — Arrange, Act, Assert структура тестов
5. **Детерминированность** — использовать seeded RNG для предсказуемых результатов

### Структура тестов

```
origa/src/use_cases/tests/
├── fixtures/              # Тестовые данные и моки
│   ├── in_memory_repository.rs   # InMemory реализация репозитория
│   └── real_dictionaries.rs      # Реальные словари для интеграционных тестов
└── journeys/              # End-to-end сценарии использования
```

### Параметризованные тесты

Использовать `rstest` для сокращения дублирования:

```rust
use rstest::rstest;

#[rstest]
#[case("食べる", "たべる")]
#[case("飲む", "のみ")]
fn test_reading_conversion(#[case] input: &str, #[case] expected: &str) {
    // test body
}
```

### Детерминированные тесты с RNG

```rust
use rand::{rngs::StdRng, SeedableRng};

#[test]
fn test_with_seeded_rng() {
    let mut rng = StdRng::seed_from_u64(42);  // Предсказуемый seed
    let result = some_random_operation(&mut rng);
    assert_eq!(result, expected_value);
}
```

### Coverage Goals

| Модуль | Цель coverage |
|--------|---------------|
| `domain/` | 80%+ |
| `use_cases/` | 85%+ |

Проверка coverage:
```bash
cargo llvm-cov --workspace --html
```

### Типы тестов

1. **Unit тесты** — изолированные тесты domain entities и value objects
2. **Integration тесты** — use cases с InMemoryRepository
3. **Journey тесты** — полные сценарии использования (learning_lesson, card_lifecycle)

### Что НЕ делать в тестах

- ❌ Тестировать private методы — рефакторить код для тестируемости
- ❌ Использовать mock библиотеки — использовать InMemoryRepository
- ❌ Недиабетические тесты — всегда seed RNG
- ❌ Дублировать тестовые данные — выносить в fixtures
- ❌ Тесты реализации вместо поведения — тестировать outcome, не implementation details

## Git Workflow

Default branch - `master`

## Commit

Use @git-commit-push subagent

## Critical Boundaries (IMPORTANT!)

### ✅ ALWAYS Do

- Запускать `cargo test --workspace` перед коммитом
- Запускать `cargo clippy --workspace -- -D warnings` перед PR
- Использовать `Result<T, OrigaError>` для всех fallible операций
- Добавлять тесты в `use_cases/tests/` для нового функционала
- Использовать `tracing::{debug, info}` для логирования
- Использовать `rstest` для параметризованных тестов

### ⚠️ ASK FIRST

- Изменения в Cargo.toml (dependencies, features)
- Изменения в CI/CD workflows
- Изменения кода domain layer

### 🚫 NEVER Do

- Коммитить без прохождения тестов
- Использовать `unwrap()` в production коде (только в тестах)
- Использовать `#[async_trait]` — использовать `impl Future`
- Коммитить console.log или println! в production коде
- Удалять test
