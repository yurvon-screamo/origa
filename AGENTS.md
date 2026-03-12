# AGENTS.md - Origa AI Assistant Guide

## Project Overview

Origa — приложение для изучения японского языка с интервальными повторениями (FSRS), OCR и токенизацией. Tech stack: Rust workspace с крейтами origa (бизнес-логика), origa_ui (Leptos/WASM frontend), tauri (Tauri v2 desktop), tokenizer. Architecture: Clean Architecture с Use Cases, Domain, Traits.

## Quick Start Commands

### Setup
```bash
# Установка Rust (если нет)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Добавление WASM target для frontend
rustup target add wasm32-unknown-unknown

# Установка Trunk (bundler для Leptos)
cargo install trunk --version 0.21.14

# Установка Tauri CLI (для desktop)
cargo install tauri-cli --version "^2"
```

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
└── tokenizer/                # Токенизатор для японского текста
```

## Git Workflow

### Commit Messages

```
type(scope): краткое описание

# Примеры:
feat(ocr): add cascade text detection
fix(srs): correct interval calculation for hard rating
refactor(use_cases): extract card creation logic
test(domain): add vocabulary card tests
```

### Branch Naming

- `feature/description` — новые функции
- `fix/description` — исправления багов
- `refactor/description` — рефакторинг

### PR Process

1. Создать branch от master
2. Сделать изменения + тесты
3. Запустить `cargo test --workspace && cargo clippy --workspace`
4. Создать PR с описанием изменений

## Critical Boundaries (IMPORTANT!)

### ✅ ALWAYS Do

- Запускать `cargo test --workspace` перед коммитом
- Запускать `cargo clippy --workspace -- -D warnings` перед PR
- Использовать `Result<T, OrigaError>` для всех fallible операций
- Добавлять тесты в `use_cases/tests/` для нового функционала
- Использовать `tracing::{debug, info}` для логирования
- Использовать `rstest` для параметризованных тестов

### ⚠️ ASK FIRST

- Изменения в `origa/src/domain/error.rs` (OrigaError)
- Изменения в trait definitions в `origa/src/traits/`
- Изменения в Cargo.toml (dependencies, features)
- Изменения в CI/CD workflows
- Удаление кода из domain layer

### 🚫 NEVER Do

- Коммитить без прохождения тестов
- Использовать `unwrap()` в production коде (только в тестах)
- Использовать `#[async_trait]` — использовать `impl Future`
- Коммитить console.log или println! в production коде
- Изменять Dockerfile без согласования
- Удалять test fixtures

## Security & Secrets

- Секреты не хранятся в репозитории
- OAuth tokens обрабатываются через deep-link и хранятся в TrailBase
- Environment variables передаются через CI/CD:
  - `ORIGA_VERSION` — версия сборки
  - `ORIGA_COMMIT` — хеш коммита
  - `ORIGA_BUILD_DATE` — дата сборки

## Gotchas & Common Pitfalls

### WASM vs Native

```rust
// Условная компиляция для OCR
#[cfg(not(target_arch = "wasm32"))]
use ort::Session;  // native

#[cfg(target_arch = "wasm32")]
use ort_web::Session;  // WASM
```

### Async в traits

```rust
// ❌ НЕ использовать
#[async_trait]
pub trait UserRepository { async fn find(...) -> ...; }

// ✅ Использовать
pub trait UserRepository {
    fn find(&self, id: Ulid) -> impl Future<Output = Result<...>>;
}
```

### Tauri JS Interop

```rust
// Доступ к __TAURI__ из Leptos (WASM)
let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__")).ok();
```

### Japanese Dictionary Loading

- `lindera-dictionary` требует `build_rs` feature
- Словарь unidic загружается при build time
- В Docker словарь удаляется для уменьшения размера

## Deployment

### Web (Docker/Railway)
```bash
docker build -f origa_ui/Dockerfile -t origa:latest .
docker run -p 4000:4000 origa:latest
```

### Desktop (Tauri)
```bash
cd tauri && cargo tauri build
# Результат: NSIS/MSI (Windows), AppImage/deb (Linux), .app (macOS)
```

### Mobile (Android)
```bash
cd tauri && cargo tauri android build
# Результат: APK в tauri/gen/android/
```

## Workspace Commands Quick Reference

```bash
# Сборка всего workspace
cargo build --workspace

# Сборка конкретного крейта
cargo build -p origa

# Запуск конкретного бинарника
cargo run -p tokenizer

# Проверка без сборки
cargo check --workspace

# Документация
cargo doc --workspace --open
```
