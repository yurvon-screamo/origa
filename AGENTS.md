# AGENTS.md - Origa AI Assistant Guide

## Project Overview

Origa — приложение для изучения японского языка с интервальными повторениями (FSRS), OCR и токенизацией. Tech stack: Rust workspace с крейтами origa (бизнес-логика), origa_ui (Leptos/WASM frontend), tauri (Tauri v2 desktop), tokenizer. Architecture: Clean Architecture с Use Cases, Domain, Traits.

## Просмотр всех доступных задач

```bash
cargo make --list-all-steps
```

### Development

```bash
# Frontend dev сервер (origa_ui) - основной вариант
cargo make dev

# Tauri dev (desktop приложение)
cargo make dev-tauri
```

### Build

```bash
# Build всех workspace крейтов (debug)
cargo make build

# Frontend production build
cargo make build-ui

# Tauri production build (native)
cargo make build-tauri

# Docker build (web)
cargo make build-docker
```

### Testing

```bash
# Все тесты workspace
cargo make test

# Тесты с выводом
cargo make test-verbose

# Coverage report (terminal)
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

### E2E Testing

> Для запуска e2e не нужно запускать вручную сервер, он запустится автоматически.

```bash
cargo make e2e                    # Все тесты (headless)
cargo make e2e-headed            # В видимом браузере
cargo make e2e-debug             # Режим отладки
```

### Environment Variables для E2E

Скопировать `.env.example` в `.env` и настроить:

- `TRAILBASE_URL` — URL TrailBase API (default: `https://origa.uwuwu.net`)
- `ADMIN_EMAIL` — Email админа
- `ADMIN_PASSWORD` — Пароль админа (обязателен для создания пользователей)

Тестовые пользователи:

- end2end\.env
- end2end\config.ts

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

Rustfmt default. Проверить: `cargo make fmt-check`. Исправить: `cargo make fmt`.

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
cargo make test-cov
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

## UI Components & E2E Testing

### test_id Паттерн

Все интерактивные UI компоненты ДОЛЖНЫ иметь `test_id` prop для e2e тестирования (Playwright).

#### Стандартный паттерн

```rust
#[component]
pub fn Button(
    #[prop(optional, into)] test_id: Signal<String>,
    // ... другие props
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <button data-testid=test_id_val>
            {children()}
        </button>
    }
}
```

#### Использование в Playwright

```typescript
// Page Object
this.submitButton = page.getByTestId("login-submit");

// Test
await expect(page.getByTestId("email-input")).toBeVisible();
await page.getByTestId("login-submit").click();
```

#### test_id для контейнеров с внутренними элементами

| Компонент | Контейнер | Внутренние элементы |
|-----------|-----------|---------------------|
| Modal | `${test_id}` | `${test_id}-close`, `${test_id}-backdrop` |
| Drawer | `${test_id}` | `${test_id}-close`, `${test_id}-backdrop` |
| Dropdown | `${test_id}` | `${test_id}-trigger`, `${test_id}-search`, `${test_id}-option-{value}` |
| Pagination | `${test_id}` | `${test_id}-prev`, `${test_id}-next`, `${test_id}-page-{n}` |
| Tabs | `${test_id}` | `${test_id}-{tab.id}` |
| Stepper | `${test_id}` | `${test_id}-step-{idx}` |
| Search | `${test_id}` | `${test_id}-input` |
| Navbar | `${test_id}` | `${test_id}-signin`, `${test_id}-cart` |
| Breadcrumbs | `${test_id}` | `${test_id}-item-{idx}` |
| Card | `${test_id}` | — |
| Table | `${test_id}` | `${test_id}-row-{row.id}` |
| CardHistoryModal | `${test_id}` (Modal) | `${test_id}-chart` |
| DeleteConfirmModal | `${test_id}` (Modal) | `${test_id}-cancel`, `${test_id}-confirm` |
| UpdateDrawer | `${test_id}` | `${test_id}-update` (Button), `${test_id}-progress` |
| ProgressBar | `${test_id}` | — |
| LoadingStageItem | `${test_id}` | — |

**Typography:**
| Text | `${test_id}` | — |
| Heading | `${test_id}` | — |
| DisplayText | `${test_id}` | — |

**Static Display:**
| Avatar | `${test_id}` | — |
| AvatarGroup | `${test_id}` | — |
| Badge | `${test_id}` | — |
| Divider | `${test_id}` | — |
| Skeleton | `${test_id}` | — |
| Stamp | `${test_id}` | — |
| Tooltip | `${test_id}` | — |

**Content:**
| FuriganaText | `${test_id}` | — |
| MarkdownText | `${test_id}` | — |
| ReadingGroup | `${test_id}` | — |
| LineChart | `${test_id}` | — |

**Layout:**
| PageLayout | `${test_id}` | — |
| CardLayout | `${test_id}` | — |
| Footer | `${test_id}` | — |
| AppSkeleton | `${test_id}` | — |
| LabelFrame | `${test_id}` | — |
| Spinner | `${test_id}` | — |
| LoadingOverlay | `${test_id}` | — |
| KanjiAnimation | `${test_id}` | — |
| KanjiWritingSection | `${test_id}` | — |

#### Автогенерация test_id

Toast компонент использует `toast.id` для автогенерации (нет optional prop):

- Toast container: `toast-{id}`
- Close button: `toast-{id}-close`

Breadcrumbs компонент автогенерирует `test_id` для каждого item на основе индекса:

- Item container: `${test_id}-item-{idx}`

Table компонент автогенерирует `test_id` для каждой row на основе row.id:

- Row: `${test_id}-row-{row.id}`

#### Компоненты БЕЗ test_id (не имеют view)

- text_to_speech.rs — утилита для генерации речи, не имеет UI компонента

## Git Workflow

Default branch - `master`

## Commit

Use @git-commit-push subagent

## Critical Boundaries (IMPORTANT!)

### ✅ ALWAYS Do

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
