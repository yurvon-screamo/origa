# AGENTS.md

This document provides guidelines for agentic coding assistants working in this repository.

## Project Overview

Origa is a Japanese language learning application built with Rust.

Workspace structure:

- `origa/` - Core domain and application logic library
- `origa_ui/` - Leptos UI library with Tauri bindings
- `tokenizer/` - Japanese text tokenization service
- `tauri/` - Tauri desktop application binary

## Build Commands

### Rust

```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p origa
cargo build -p origa_ui

# Build for release
cargo build --release

# Check compilation (faster than build)
cargo check
```

### UI (Trunk)

```bash
# Development server
cd origa_ui && trunk serve --port 8080

# Build for production
cd origa_ui && trunk build --release
```

### Tauri Desktop App

```bash
cd tauri && cargo tauri dev
```

## Test Commands

### Unit Tests (Rust)

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p origa

# Run single test by name
cargo test test_name

# Run single test in specific module
cargo test -p origa -- mod::test_name

# Run tests with output
cargo test -- --nocapture

# Run specific test file pattern
cargo test --test integration_tests
```

### E2E Tests (Playwright)

```bash
# Run all e2e tests
npm test

# Run specific test file
npx playwright test journeys/full-learning-cycle.spec.ts

# Run with UI mode
npm run test:ui

# Run in headed mode
npm run test:headed

# Debug mode
npm run test:debug
```

## Code Style Guidelines

### General Principles

- Write concise, focused functions under 50 lines when possible
- Use early returns to reduce nesting
- Prefer composition over inheritance (Rust idiomatic)
- Avoid unnecessary clones; use references or Arc/Rc where appropriate

### Imports Organization

Group imports in this order, separated by blank lines:

```rust
use std::collections::HashMap;                          // 1. std
use chrono::{DateTime, Utc};                            // 2. External crates
use serde::{Deserialize, Serialize};                    // 2. External crates (continued)
use ulid::Ulid;

use crate::application::jlpt_content_loader::JlptContent;  // 3. Workspace crates
use crate::domain::{Card, JapaneseLevel, OrigaError};      // 3. Workspace crates (continued)
```

- Use `use` with braces for multiple items from same crate
- Import types directly, not their modules
- Use `crate::` for internal imports, not `super::` when possible

### Naming Conventions

- **Types**: PascalCase (`User`, `KnowledgeSet`, `JlptProgress`)
- **Functions/Methods**: snake_case (`create_card`, `recalculate_jlpt_progress`)
- **Variables**: snake_case (`user_id`, `memory_state`)
- **Constants**: SCREAMING_SNAKE_CASE (`KANJI_DICTIONARY`)
- **Modules**: snake_case (`jlpt_progress`, `value_objects`)
- **Files**: snake_case (`user.rs`, `jlpt_progress.rs`)

### Type Design

- Use newtype pattern for domain primitives (`Question`, `Answer`)
- Prefer enums over bools for state (`JapaneseLevel::N5` not `level: u8`)
- Use `Option<T>` for optional values, never use sentinel values
- Use `Result<T, E>` for fallible operations with specific error types

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

- Use `thiserror` for error definitions (already in workspace)
- Define errors as enums with context in variants
- Propagate errors with `?` operator
- Never use `unwrap()` in production code; use `expect()` with clear message only in tests

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrigaError {
    UserNotFound { user_id: Ulid },
    CardNotFound { card_id: Ulid },
    InvalidQuestion { reason: String },
    RepositoryError { reason: String },
}

impl fmt::Display for OrigaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrigaError::UserNotFound { user_id } => {
                write!(f, "User with id {} not found", user_id)
            }
        }
    }
}

impl std::error::Error for OrigaError {}
```

### Async/Await Patterns

- Prefer `Result<T, E>` over `Option` for async operations
- Use `tokio::spawn` for parallel operations when appropriate
- Avoid blocking in async code; use async equivalents
- Handle errors explicitly; don't silently ignore them

### Testing

- Use `rstest` for parameterized tests (already in workspace dev-deps)
- Place unit tests in same file with `#[cfg(test)] mod tests`
- Test names should describe behavior: `user_new_creates_default_jlpt_progress`
- Use helper functions for common test setup

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn create_test_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
            Answer::new("meaning".to_string()).unwrap(),
        ))
    }

    #[test]
    fn user_new_creates_default_jlpt_progress() {
        let user = User::new("test@example.com".to_string(), NativeLanguage::Russian, None);
        assert_eq!(user.current_japanese_level(), JapaneseLevel::N5);
    }
}
```

## UI Component Library (origa_ui)

### Architecture Philosophy

The UI library provides reusable, styled components for consistent design across the application. **Always prefer library components over custom implementations**.

### Library Structure

Located in `origa_ui/src/ui_components/`:

- **Layout Components**: `PageLayout`, `CardLayout`
- **Form Components**: `Input`, `Button`, `Checkbox`, `Toggle`, `Radio`
- **Typography**: `Heading`, `Text`, `DisplayText`
- **Container Components**: `Card`, `Divider`, `Avatar`, `Badge`
- **Feedback Components**: `Alert`, `Modal`, `Toast`, `Tooltip`
- **Navigation**: `Navbar`, `Breadcrumbs`, `Pagination`, `Tabs`

### Component Props

Components use enum-based props for type safety:

```rust
#[component]
pub fn Button(
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView
```

### Event Handling

- Callbacks use `Callback<T>` type from `leptos::prelude`
- Event types from `leptos::ev` module (e.g., `MouseEvent`, `Event`, `KeyboardEvent`)
- Pass callbacks directly: `on_click=Callback::new(|_| ...)`

### Export Policy

All components are exported via `pub use` in `ui_components/mod.rs`:

```rust
use crate::ui_components::{Button, ButtonVariant, Input, Heading, HeadingLevel};
```

### Design System

- Colors use CSS variables: `--accent-olive`, `--bg-primary`, `--fg-muted`, `--border-color`
- Typography: `font-serif` for headings, `font-mono` for code/data
- Spacing: Tailwind utility classes (e.g., `space-y-5`, `mb-4`, `p-6`)
- Responsive: `sm:`, `md:`, `lg:` prefixes for breakpoints

Full CSS config placed in `origa_ui/input.css` file.

## JLPT Progress System

### Architecture

The JLPT progress system automatically tracks user progress across all five JLPT levels (N5-N1):

- **Domain Layer** (`origa/src/domain/jlpt_progress.rs`): Core models
- **Application Layer** (`origa/src/application/jlpt_content_loader.rs`): Content loader
- **UI Layer** (`origa_ui/src/pages/home/jlpt_progress_card.rs`): Progress visualization

### Automatic Level Detection

1. **New users**: Start at N5 by default
2. **Level progression**: When average progress reaches 90%, advance to next level
3. **Maximum level**: N1 is the highest level

### Progress Recalculation

Progress is recalculated automatically when:
- User adds a new study card
- User updates an existing card
- User removes a card

## Important Reminders

- Run `cargo check` after making changes to verify compilation
- Run `cargo test -p <crate>` after modifying logic
- Run `npm test` for e2e tests after UI changes
- Use existing components from `ui_components/` instead of creating custom HTML
