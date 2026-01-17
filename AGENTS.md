# AGENTS.md

This document provides guidelines for agentic coding assistants working in this repository.

## Project Overview

Origa is a Japanese language learning application built with Rust, using:
- **Dioxus 0.7** for cross-platform UI (web, desktop, mobile)
- **Tauri 2** for native desktop application wrapper
- **SQLite** with rusqlite for data persistence
- **FSRS** (Free Spaced Repetition Scheduler) algorithm for spaced repetition learning
- **Tokio** for async runtime

Workspace structure:
- `origa/` - Core domain and application logic library
- `origa_ui/` - Dioxus UI library with Tauri bindings
- `tauri/` - Tauri desktop application binary

## Build Commands

```sh
# Install Dioxus CLI
cargo install dioxus-cli --locked

# Web development (hot reload)
cd origa && dx serve

# Desktop development
cd origa && dx serve --desktop

# Android development
cd origa && dx serve --android

# Release builds
cd origa && dx bundle --desktop --release
cd origa && dx bundle --android --release --target aarch64-linux-android

# Build all workspace crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Run a single test
cargo test -p origa create_card_use_case_should_create_card_and_save_to_database
cargo test -p origa rate_card_use_case_should_add_review_and_update_schedule
cargo test --test create_card
cargo test --test rate_card
```

## Code Style Guidelines

### General Principles

- Write concise, focused functions under 50 lines when possible
- Use early returns to reduce nesting
- Prefer composition over inheritance (Rust idiomatic)
- Avoid unnecessary clones; use references or Arc/Rc where appropriate

### Naming Conventions

- **Crates**: snake_case (e.g., `origa_ui`, `rs-fsrs`)
- **Modules**: snake_case (e.g., `use_cases`, `knowledge`)
- **Types**: PascalCase (e.g., `User`, `Card`, `OrigaError`)
- **Functions**: snake_case (e.g., `get_repository`, `create_card`)
- **Variables**: snake_case (e.g., `user_id`, `card_content`)
- **Constants**: SCREAMING_SNAKE_CASE for static constants, snake_case for const items
- **Type parameters**: Short, uppercase (e.g., `T`, `E` for errors)

### Imports and Module Structure

```rust
// Standard library imports first, grouped
use std::fmt;

// External crate imports second
use serde::{Deserialize, Serialize};
use ulid::Ulid;

// Local imports last, using paths from crate root
use origa::application::UserRepository;
use origa::domain::{User, Card};
```

- Use `use` statements at module level, avoid inline paths in code
- Re-export commonly used types from parent modules for convenience
- Keep module structure flat where possible: `origa/src/domain/`, `origa/src/application/`

### Error Handling

```rust
// Define errors as an enum with Display and Error traits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrigaError {
    UserNotFound { user_id: Ulid },
    CardNotFound { card_id: Ulid },
    RepositoryError { reason: String },
    LlmError { reason: String },
}

impl fmt::Display for OrigaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrigaError::UserNotFound { user_id } => {
                write!(f, "User with id {} not found", user_id)
            }
            // ... other variants
        }
    }
}

impl std::error::Error for OrigaError {}
```

- Use `thiserror` for simpler error definitions when appropriate
- Propagate errors with `?` operator
- Include context in error messages (what failed, why, what values)
- Avoid generic error types; use specific error variants

### Async/Await Patterns

```rust
// Prefer Result<T, E> over Option for async operations
async fn get_user(&self, id: Ulid) -> Result<Option<User>, RepositoryError> {
    // Implementation
}

// Use tokio::spawn for parallel operations when appropriate
let tasks: Vec<_> = users
    .into_iter()
    .map(|u| tokio::spawn(process_user(u)))
    .collect();

let results: Vec<_> = join_all(tasks).await;
```

- Use `async-trait` for trait methods that need to be async
- Avoid blocking in async code; use async equivalents
- Handle errors explicitly; don't silently ignore them

### Dioxus 0.7 Specific Guidelines

```rust
// Components are functions with #[component] attribute
#[component]
fn CardView(card: ReadOnlySignal<Card>) -> Element {
    let mut show_details = use_signal(|| false);

    rsx! {
        div {
            class: "card",
            onclick: move |_| show_details.toggle(),
            if *show_details.read() {
                CardDetails { card }
            }
        }
    }
}

// Use Signals for reactive state
let mut count = use_signal(|| 0);
let doubled = use_memo(move || count() * 2);

// Props must be owned values, implement Clone + PartialEq
#[props]
struct CardProps {
    card: Card,
    #[props(default)]
    show_back: bool,
}
```

- Never use `cx`, `Scope`, or `use_state` from Dioxus 0.5/0.6
- Use `use_signal`, `use_memo`, `use_resource` for state management
- Components receive props as function arguments, not through context
- Wrap props in `ReadOnlySignal` for reactive parent-to-child data flow
- Use `asset!` macro for asset paths: `src: asset!("/assets/image.png")`

### Testing

```rust
#[tokio::test]
async fn test_name() {
    // Arrange
    setup_test_environment().await;
    let repository = create_test_repository().await;

    // Act
    let result = do_something(&repository).await;

    // Assert
    assert!(result.is_ok());
}

#[cfg(test)]
pub async fn create_test_repository() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    let _ = ApplicationEnvironment::from_database_path(db_path);
}
```

- Use `rstest` for parameterized tests when needed
- Keep tests isolated; use temporary directories for database tests
- Use descriptive test names: `should_create_card_and_persist_to_database`
- Group test utilities in `tests/mod.rs`

### Database (rusqlite)

```rust
// Use transactions for multi-statement operations
let tx = conn.transaction()?;
tx.execute("INSERT INTO cards (...)", params)?;

// Bind parameters by position with ? placeholders
conn.execute(
    "SELECT * FROM cards WHERE id = ?",
    params![card_id],
)?
```

- Use transactions for atomic operations
- Bind all user inputs to prevent SQL injection
- Use blob types for serialized structures when appropriate

### Type Design

```rust
// Use newtype pattern for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardId(Ulid);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Answer(String);

impl Answer {
    pub fn new(s: String) -> Result<Self, OrigaError> {
        if s.trim().is_empty() {
            return Err(OrigaError::InvalidAnswer {
                reason: "Answer cannot be empty".to_string(),
            });
        }
        Ok(Self(s))
    }
}
```

- Create wrapper types for domain concepts (CardId, UserId)
- Validate in constructors; return errors for invalid states
- Implement `Display`, `Serialize`, `Deserialize` for domain types

### File Organization

```
origa/src/
├── lib.rs              # crate root, exports
├── domain/             # core business logic
│   ├── mod.rs
│   ├── knowledge/      # Card types, vocabulary, kanji
│   ├── memory/         # FSRS memory state
│   └── value_objects/  # Question, Answer, etc.
├── application/        # use cases, services
│   ├── use_cases/      # business operations
│   ├── user_repository.rs
│   └── srs_service.rs
├── infrastructure/     # external integrations
│   ├── user_repository.rs
│   ├── llm/            # OpenAI, Gemini clients
│   └── duolingo_client.rs
└── settings.rs         # application configuration

origa_ui/src/
├── main.rs             # Dioxus app entry
├── components/         # reusable UI components
├── views/              # page-level components
│   ├── cards/
│   ├── profile/
│   └── import/
└── domain/             # UI-specific domain types
```

- Keep domain logic separate from infrastructure
- Use case files should be small (~50-100 lines) and focused
- Extract reusable UI components to `components/`

## Key Dependencies

- `dioxus = "0.7"` - UI framework with router
- `tokio = { version = "1.48", features = ["rt", "macros", "time"] }` - Async runtime
- `rusqlite = { version = "0.38", features = ["bundled"] }` - SQLite with bundled build
- `rs-fsrs = "1.2"` - Spaced repetition algorithm
- `lindera = { version = "1.4", features = ["embedded-unidic"] }` - Japanese tokenizer
- `async-openai-wasm` - OpenAI API client (WASM compatible)
- `dioxus-heroicons` - Icon components
- `rstest` - Testing framework

## Development Tips

1. **Debug logging**: Use `tracing::info!`, `tracing::debug!` macros
2. **Database inspection**: SQLite files are in user config directory; use `sqlite3` CLI to inspect
3. **Web development**: Run `dx serve` in `origa/` directory for hot reload
4. **Desktop**: Build with Tauri for native window, tray icon, menu bar integration
5. **State management**: Use Dioxus signals for local state, context provider for global state

## Common Operations

```sh
# Add new dependency to workspace
# Edit root Cargo.toml [workspace.dependencies]

# Generate new use case
# Create file origa/src/application/use_cases/my_use_case.rs
# Add to origa/src/application/use_cases.rs with pub mod my_use_case;

# Add new UI view
# Create origa_ui/src/views/my_view.rs with component
# Add route in origa_ui/src/main.rs Route enum
```

## Dioxus Resources

- Official docs: https://dioxuslabs.com/learn/0.7
- Router: https://dioxuslabs.com/learn/0.7/reference/router
- State management: https://dioxuslabs.com/learn/0.7/reference/state
