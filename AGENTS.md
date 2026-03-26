# AGENTS.md - Origa Development Guide

## Project Overview

Origa — приложение для изучения японского языка с интервальными повторениями (FSRS),
OCR и токенизацией.
**Tech stack**: Rust workspace (крейты `origa`, `origa_ui`, `tokenizer`),
Leptos/WASM, Tauri v2.
**Архитектура**: Clean Architecture (Use Cases → Domain → Traits).

## Просмотр всех доступных задач

```bash
cargo make --list-all-steps
```

### Development

```bash
# Frontend dev сервер (Leptos) — основной вариант
cargo make dev

# Tauri dev (desktop приложение)
cargo make dev-tauri
```

### Build

```bash
# Сборка всех workspace-крейтов (debug)
cargo make build

# Production-сборка frontend
cargo make build-ui

# Production-сборка Tauri (native)
cargo make build-tauri

# Docker build (web-версия)
cargo make build-docker
```

### Testing

```bash
# Все тесты workspace
cargo make test

# Тесты с выводом
cargo make test-verbose

# Coverage report (терминал)
cargo make test-cov-report
```

### Linting & Formatting

```bash
# Все проверки
cargo make lint
```

### Code Quality (QLTY)

```bash
# Полная проверка со всеми плагинами
cargo make qlty-full
```

### E2E Testing (Playwright)
>
> Сервер запускается автоматически, вручную запускать не нужно.

```bash
cargo make e2e          # Все тесты (headless)
cargo make e2e-headed   # В видимом браузере
cargo make e2e-debug    # Режим отладки
```

### Environment Variables для E2E

Скопировать `.env.example` → `.env` и настроить:

- `TRAILBASE_URL` — URL TrailBase API (по умолчанию `https://origa.uwuwu.net`)
- `ADMIN_EMAIL` — Email админа
- `ADMIN_PASSWORD` — Пароль админа (обязателен для создания пользователей)

Тестовые пользователи настраиваются в:

- `end2end/.env`
- `end2end/config.ts`

## Code Style & Conventions

### Imports

```rust
// ✅ Правильно
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{debug, info};

use crate::domain::OrigaError;
use crate::traits::UserRepository;
```

### Naming Conventions

| Элемент       | Конвенция              | Пример             |
|---------------|------------------------|--------------------|
| Struct/Enum   | `PascalCase`           | `RateCardUseCase`  |
| Enum variants | `PascalCase`           | `UserNotFound`     |
| Functions     | `snake_case`           | `find_by_id()`     |
| Variables     | `snake_case`           | `user_id`          |
| Constants     | `SCREAMING_SNAKE_CASE` | `KANJI_DICTIONARY` |
| Trait         | `PascalCase`           | `UserRepository`   |

### Formatting

Rustfmt по умолчанию.  
Проверка: `cargo make fmt-check`  
Исправление: `cargo make fmt`

### Error Handling

Единый `OrigaError` с именованными полями:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrigaError {
    UserNotFound { user_id: Ulid },
    CardNotFound { card_id: Ulid },
    InvalidQuestion { reason: String },
}
```

### Use Case Pattern

```rust
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
        // business logic
    }
}
```

### Repository Traits (Async)

```rust
pub trait UserRepository {
    fn find_by_id(&self, user_id: Ulid)
        -> impl Future<Output = Result<Option<User>, OrigaError>>;

    fn save(&self, user: &User)
        -> impl Future<Output = Result<(), OrigaError>>;
}
```

### Comments & Documentation

- `// TODO:` — только для незавершённой работы
- Код должен быть самодокументируемым через понятные имена
- `///` (doc comments) — только когда действительно нужны

## Project Structure

```
origa/
├── origa/                  # Бизнес-логика
│   └── src/
│       ├── domain/         # Entities, Value Objects
│       ├── use_cases/      # Application layer
│       ├── traits/         # Repository traits
│       └── ocr/            # ONNX OCR модуль
├── origa_ui/               # Leptos frontend (WASM)
│   └── src/
│       ├── components/
│       ├── pages/
│       └── repository/     # TrailBase client
├── tauri/                  # Tauri v2 desktop
└── utils/                  # Утилиты
```

## Testing Principles (по книге Владимира Хорикова)

### Обязательные принципы

1. **Чёрный ящик** — тестируем только публичный API
2. **Устойчивость к рефакторингу** — тесты не ломаются при изменении реализации
3. **Изоляция** — `InMemoryRepository` вместо моков
4. **AAA** — Arrange → Act → Assert
5. **Детерминированность** — seeded RNG

### Структура тестов

```
origa/src/use_cases/tests/
├── fixtures/           # InMemoryRepository + реальные словари
└── journeys/           # Полные сценарии (card lifecycle и т.д.)
```

### Параметризованные тесты

Использовать `rstest`.

### Детерминированные тесты с RNG

```rust
use rand::{rngs::StdRng, SeedableRng};

let mut rng = StdRng::seed_from_u64(42);
```

### Coverage Goals

- `domain/` → 80%+
- `use_cases/` → 85%+

### Типы тестов

- **Unit** — domain entities
- **Integration** — use cases + `InMemoryRepository`
- **Journey** — полные end-to-end сценарии

**Запрещено**:

- Тестировать private методы
- Использовать mock-библиотеки
- `unwrap()` в production-коде
- Недиабетические тесты

## UI Components & E2E Testing

### test_id Паттерн (обязательно для всех интерактивных компонентов)

Все интерактивные компоненты **должны** получать `test_id: Signal<String>`
prop.

```rust
#[component]
pub fn Button(
    #[prop(optional, into)] test_id: Signal<String>,
    // ...
) -> impl IntoView {
    view! {
        <button data-testid=move || test_id.get()>
            {children()}
        </button>
    }
}
```

### Автогенерация test_id

- **Toast** → `toast-{id}` / `toast-{id}-close`
- **Breadcrumbs** → `${test_id}-item-{idx}`
- **Table** → `${test_id}-row-{row.id}`

Полный список префиксов для контейнеров и дочерних элементов — в исходном коде
компонентов (не дублировать здесь).

## Git Workflow

- Default branch: `master`

## Commit

Использовать `@git-commit-push` subagent.

## Critical Boundaries

### ✅ ALWAYS DO

- Возвращать `Result<T, OrigaError>`
- Добавлять тесты в `use_cases/tests/` при любом новом функционале
- Использовать `tracing::{debug, info}`
- Параметризованные тесты через `rstest`

### ⚠️ ASK FIRST

- Изменения в `Cargo.toml`
- Изменения в CI/CD
- Изменения в domain layer

### 🚫 NEVER DO

- Коммитить без прохождения всех тестов
- Использовать `unwrap()` в production-коде
- Использовать `#[async_trait]`
- Коммитить `console.log` / `println!`
- Удалять тесты

```markdown
