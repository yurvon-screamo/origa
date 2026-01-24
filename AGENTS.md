# AGENTS.md

This document provides guidelines for agentic coding assistants working in this repository.

## Project Overview

Origa is a Japanese language learning application built with Rust, using:

- **Leptos 0.7** for reactive web UI with CSR (Client-Side Rendering)
- **Thaw 0.4** for UI component library
- **Tauri 2** for native desktop application wrapper
- **SQLite** with rusqlite for data persistence
- **FSRS** (Free Spaced Repetition Scheduler) algorithm for spaced repetition learning
- **Tokio** for async runtime

Workspace structure:

- `origa/` - Core domain and application logic library
- `origa_ui/` - Leptos UI library with Tauri bindings
- `tokenizer/` - Japanese text tokenization service
- `tauri/` - Tauri desktop application binary

## Build Commands

```sh
# Web development (hot reload)
cd origa_ui && cargo trunk watch

# Desktop development
cd tauri && cargo tauri dev

# Release builds
cd origa_ui && cargo trunk build --release
cd tauri && cargo tauri build

# Build all workspace crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Run a single test
cargo test -p origa create_card_use_case_should_create_card_and_save_to_database
cargo test -p origa rate_card_use_case_should_add_review_and_update_schedule
cargo test --test create_card
cargo test --test rate_card

# Lint and format
cargo clippy --workspace -- -D warnings
cargo fmt --check --all
cargo fmt --all
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

- Define errors as enums with Display and Error traits
- Use `thiserror` for simpler error definitions when appropriate
- Propagate errors with `?` operator
- Include context in error messages (what failed, why, what values)
- Avoid generic error types; use specific error variants

### Async/Await Patterns

- Prefer Result<T, E> over Option for async operations
- Use tokio::spawn for parallel operations when appropriate
- Use `async-trait` for trait methods that need to be async
- Avoid blocking in async code; use async equivalents
- Handle errors explicitly; don't silently ignore them

### Leptos 0.7 Specific Guidelines

- Use `create_signal`, `create_memo`, `create_resource` for state management
- Components are functions with `#[component]` attribute receiving props as arguments
- Use `view!` macro for UI rendering
- Access signal values with `.get()` and set with `.set()`
- Props must implement Clone + IntoView

### Testing

- Use `rstest` for parameterized tests when needed
- Keep tests isolated; use temporary directories for database tests
- Use descriptive test names: `should_create_card_and_persist_to_database`
- Group test utilities in `tests/mod.rs`

### Database (rusqlite)

- Use transactions for atomic operations
- Bind all user inputs to prevent SQL injection with ? placeholders
- Use blob types for serialized structures when appropriate

### Type Design

- Create wrapper types for domain concepts (CardId, UserId)
- Validate in constructors; return errors for invalid states
- Implement `Display`, `Serialize`, `Deserialize` for domain types

### File Organization

- Keep domain logic separate from infrastructure
- Use case files should be small (~50-100 lines) and focused
- Extract reusable UI components to `components/`
- Keep module structure flat where possible: `origa/src/domain/`, `origa/src/application/`

## Key Dependencies

- `leptos = "0.7"` - UI framework with router
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
3. **Web development**: Run `cargo leptos watch` in `origa_ui/` directory for hot reload
4. **Desktop**: Build with Tauri for native window, tray icon, menu bar integration
5. **State management**: Use Leptos signals for local state, context provider for global state

## Common Operations

- Add new dependencies to root Cargo.toml [workspace.dependencies]
- Create use cases in `origa/src/application/use_cases/` and add to mod.rs
- Add UI views in `origa_ui/src/views/` and routes in main.rs

## Leptos Resources

- Official docs: <https://book.leptos.dev/>
- Router: <https://book.leptos.dev/view/09_router.html>
- State management: <https://book.leptos.dev/view/02_reactivity.html>
