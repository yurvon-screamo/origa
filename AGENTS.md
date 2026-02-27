# AGENTS.md

This document provides guidelines for agentic coding assistants working in this repository.

## Project Overview

Origa is a Japanese language learning application built with Rust.

Workspace structure:

- `origa/` - Core domain and application logic library
- `origa_ui/` - Leptos UI library with Tauri bindings
- `tokenizer/` - Japanese text tokenization service
- `tauri/` - Tauri desktop application binary

## Code Style Guidelines

### General Principles

- Write concise, focused functions under 50 lines when possible
- Use early returns to reduce nesting
- Prefer composition over inheritance (Rust idiomatic)
- Avoid unnecessary clones; use references or Arc/Rc where appropriate

### Error Handling

- Define errors as enums with Display and Error traits
- Use `thiserror` for simpler error definitions when appropriate
- Propagate errors with `?` operator
- Include context in error messages (what failed, why, what values)
- Avoid generic error types; use specific error variants

### Async/Await Patterns

- Prefer Result<T, E> over Option for async operations
- Use tokio::spawn for parallel operations when appropriate
- Avoid blocking in async code; use async equivalents
- Handle errors explicitly; don't silently ignore them

## UI Component Library (origa_ui)

### Architecture Philosophy

The UI library provides reusable, styled components for consistent design across the application. **Always prefer library components over custom implementations**.

### Library Structure

Located in `origa_ui/src/ui_components/`:

- **Layout Components**: `PageLayout` (centered/full/compact variants), `CardLayout` (size variants)
- **Form Components**: `Input`, `Button`, `Checkbox`, `Toggle`, `Radio`
- **Typography**: `Heading`, `Text`, `DisplayText` with size/variant enums
- **Container Components**: `Card`, `Divider`, `Avatar`, `Badge`
- **Feedback Components**: `Alert`, `Modal`, `Tooltip`
- **Navigation**: `Navbar`, `Breadcrumbs`, `Pagination`, `Tabs`
- **Data Display**: `Table`, `Stepper`, `Progress`
- **Utility Components**: `Skeleton`, `Search`, `Dropdown`

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
- Pass callbacks directly, not wrapped in `Some()`: `on_click=Callback::new(|_| ...)`

### Export Policy

All components are exported via `pub use` in `ui_components/mod.rs`. Import directly from the library:

```rust
use crate::ui_components::{Button, ButtonVariant, Input, Heading, HeadingLevel};
```

### Usage Guidelines

- **Never** write custom HTML elements when library components exist
- Replace custom `input`/`button`/`label` with library equivalents
- Replace custom `h*`/`p`/`span` with typography components
- Use enum props (e.g., `ButtonVariant::Olive`, `HeadingLevel::H1`) instead of strings
- Maintain consistent styling via component variants
- Leverage shared layout components for page structure

### Design System

- Colors use CSS variables: `--accent-olive`, `--bg-primary`, `--fg-muted`, `--border-color`
- Typography: `font-serif` for headings, `font-mono` for code/data
- Spacing: Tailwind utility classes (e.g., `space-y-5`, `mb-4`, `p-6`)
- Responsive: `sm:`, `md:`, `lg:` prefixes for breakpoints

Full css config placed in `origa_ui/input.css` file.
