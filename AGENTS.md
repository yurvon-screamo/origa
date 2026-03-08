# AGENTS.md

Guidelines for agentic coding assistants working in this repository.

## Project Overview

Origa is a Japanese language learning application built with Rust using FSRS algorithm for spaced repetition.

Workspace structure:
- `origa/` - Core domain and application logic library
- `origa_ui/` - Leptos UI library with Tauri bindings (WASM)
- `tokenizer/` - Japanese text tokenization service
- `tauri/` - Tauri desktop application binary

Architecture: Domain-driven with traits for repositories, use cases for business logic.

## Build Commands

```bash
cargo build                        # Build entire workspace
cargo build -p origa               # Build specific crate
cargo build --release              # Release build
cargo check                        # Check compilation (faster)
cargo clippy -- -D warnings        # Lint with clippy
cargo fmt -- --check               # Check formatting
```

## Development Commands

```bash
cd origa_ui && trunk serve --port 8080    # Frontend dev server
cd tauri && cargo tauri dev               # Tauri development
```

## Test Commands

```bash
cargo test                              # All tests
cargo test -p origa                     # Tests for origa crate (137 tests)
cargo test test_name                    # Single test by name
cargo test -p origa -- journeys::test_name  # Test in specific journey
cargo test -- --nocapture               # With output
cargo test --target wasm32-unknown-unknown  # WASM tests (origa_ui)
npm test                                # All e2e tests
npx playwright test journeys/full-learning-cycle.spec.ts  # Single e2e file
npm run test:ui                         # Playwright UI mode
```

### Test Structure

**Journey tests** (`origa/src/use_cases/tests/journeys/`):
- `onboarding.rs` - 5 tests (user creation, profile)
- `card_lifecycle.rs` - 13 tests (kanji, favorites, delete, well-known sets)
- `grammar.rs` - 3 tests (grammar rules loading and card creation)
- `learning_lesson.rs` - 8 tests (standard lesson rating)
- `learning_fixation.rs` - 7 tests (fixation lesson rating)

**Fixtures** (`origa/src/use_cases/tests/fixtures/`):
- `InMemoryUserRepository` - in-memory user storage
- `FileWellKnownSetLoader` - loads well-known sets from `origa_ui/public/`
- `init_real_dictionaries()` - loads vocabulary, kanji, grammar from files

**Test principles** (Vladimir Khorikov):
- Test behavior, not implementation (black box)
- Mocks only for external dependencies
- Use real objects for internal collaborators
- All tests use real dictionaries from `origa_ui/public/domain/`

## Code Style Guidelines

### General Principles

- Write concise functions under 50 lines
- Use early returns to reduce nesting
- Prefer composition over inheritance
- Avoid unnecessary clones; use references or Arc/Rc
- No comments unless absolutely necessary

### Imports Organization

Group imports in order, separated by blank lines: 1) `std`, 2) external crates, 3) workspace/internal crates.

```rust
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::traits::user_repository::UserRepository;
use crate::domain::{Card, JapaneseLevel, OrigaError};
```

### Naming Conventions

- **Types**: PascalCase (`User`, `JlptProgress`)
- **Functions/Methods**: snake_case (`create_card`, `recalculate_progress`)
- **Variables**: snake_case (`user_id`, `memory_state`)
- **Constants**: SCREAMING_SNAKE_CASE (`KANJI_DICTIONARY`)
- **Modules/Files**: snake_case (`jlpt_progress.rs`)

### Type Design

- Use newtype pattern for domain primitives (`Question`, `Answer`)
- Prefer enums over bools for state (`JapaneseLevel::N5` not `level: u8`)
- Use `Option<T>` for optional values, never sentinel values
- Use `Result<T, E>` for fallible operations

```rust
pub struct Question(String);

impl Question {
    pub fn new(value: String) -> Result<Self, OrigaError> {
        if value.trim().is_empty() {
            return Err(OrigaError::InvalidQuestion {
                reason: "Question cannot be empty".to_string(),
            });
        }
        Ok(Self(value))
    }
}
```

### Error Handling

- Use `thiserror` for error definitions
- Define errors as enums with context in variants
- Propagate errors with `?` operator
- Never use `unwrap()` in production; use `expect()` only in tests

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrigaError {
    UserNotFound { user_id: Ulid },
    CardNotFound { card_id: Ulid },
    InvalidQuestion { reason: String },
}
```

### Async/Await Patterns

- Use `impl Future<Output = Result<T, E>>` in trait definitions
- Use `tokio::spawn` for parallel operations
- Avoid blocking in async code; handle errors explicitly

```rust
pub trait UserRepository {
    fn find_by_id(&self, user_id: Ulid) -> impl Future<Output = Result<Option<User>, OrigaError>>;
    fn save(&self, user: &User) -> impl Future<Output = Result<(), OrigaError>>;
}
```

### Logging

- Use `tracing` crate with macros: `debug!`, `info!`, `warn!`, `error!`
- Include structured fields: `debug!(user_id = %user_id, "Processing user")`

## UI Component Library (origa_ui)

**Always prefer library components over custom implementations.**

Located in `origa_ui/src/ui_components/`:
- **Layout**: `PageLayout`, `CardLayout`
- **Forms**: `Input`, `Button`, `Checkbox`, `Toggle`, `Radio`
- **Typography**: `Heading`, `Text`, `DisplayText`
- **Containers**: `Card`, `Divider`, `Avatar`, `Badge`
- **Feedback**: `Alert`, `Modal`, `Toast`, `Tooltip`
- **Navigation**: `Navbar`, `Breadcrumbs`, `Pagination`, `Tabs`

### Leptos Component Pattern

Components use enum-based props with `#[prop(optional)]`. Callbacks use `Callback<T>` from `leptos::prelude`.

```rust
#[component]
pub fn Button(
    #[prop(optional, into)] variant: Signal<ButtonVariant>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class="btn"
            disabled=move || disabled.get()
            on:click=move |ev| {
                if let Some(on_click) = on_click {
                    on_click.run(ev);
                }
            }
        >
            {children()}
        </button>
    }
}
```

### Design System

- Colors: CSS variables (`--accent-olive`, `--bg-primary`, `--fg-muted`)
- Typography: `font-serif` for headings, `font-mono` for code
- Spacing: Tailwind utilities (`space-y-5`, `mb-4`, `p-6`)
- Responsive: `sm:`, `md:`, `lg:` breakpoints

## TrailBase Backend

- **URL**: `https://trailbase.uwuwu.net`
- **Auth**: OAuth-only (Google, Yandex via Keycloak)
- `TrailBaseClient` - WASM-compatible HTTP client
- `TrailBaseUserRepository` - Record API implementation
- `HybridUserRepository` - Syncs local storage with TrailBase

## Important Reminders

- Run `cargo check` after changes to verify compilation
- Run `cargo test -p <crate>` after modifying logic
- Run `npm test` for e2e tests after UI changes
- Use existing `ui_components/` instead of custom HTML
- Run `cargo clippy` before committing

## Architecture Notes

### Card Storage Strategy

All card types (Vocabulary, Kanji, Grammar) store only minimal key data and fetch details from dictionaries at runtime:

**VocabularyCard**: stores `word: Question`, optional `original_word: Question` (for reversed mode)
**KanjiCard**: stores only `kanji: Question`
**GrammarRuleCard**: stores only `rule_id: Ulid`

Methods like `answer()`, `question()`, `description()`, `example_words()` fetch data dynamically from dictionaries. This eliminates data duplication and ensures consistency with dictionary updates.

**Constants**: `FALLBACK_ANSWER = "—"` used for missing translations/descriptions

**Shared utilities**: `is_word_known()` in `use_cases/shared.rs` checks if a word exists in user's vocabulary
