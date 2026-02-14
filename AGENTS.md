# AGENTS.md

This document provides guidelines for agentic coding assistants working in this repository.

## Project Overview

Origa is a Japanese language learning application built with Rust.

Workspace structure:

- `origa/` - Core domain and application logic library
- `origa_ui/` - Leptos UI library with Tauri bindings
- `tokenizer/` - Japanese text tokenization service
- `tauri/` - Tauri desktop application binary
- `telegram/` - Telegram desktop application binary

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
- Use `async-trait` for trait methods that need to be async
- Avoid blocking in async code; use async equivalents
- Handle errors explicitly; don't silently ignore them

## Leptos Architecture

### Reactive System

Leptos uses fine-grained reactivity without virtual DOM. UI nodes are directly subscribed to signals, enabling minimal DOM updates. Core reactive primitives:

- **Signals** (`create_signal`, `RwSignal`): Reactive state sources with getter/setter pattern
- **Effects** (`create_effect`): Reactive code that runs when dependencies change
- **Memos** (`create_memo`): Computed derived values with caching
- **Resources** (`create_resource`): Async data with loading/error states

**Critical Rule**: Never destructure signal values outside tracking scopes (`view!`, `create_effect`). Always use closures `move || signal.get()` for reactivity.

### Component Model

Leptos components are setup functions that execute **once**, not re-render functions. Key principles:

- Annotate with `#[component]`
- Props use attributes: `#[prop(into)]`, `#[prop(optional)]`, `#[prop(default)]`
- `view!` macro returns UI tree
- Dynamic content in `view!` must be closures for reactivity
- Use `Show`, `For`, `Suspense`, `Transition` for reactive control flow

### Isomorphic Application

Code runs on both server (SSR) and client (CSR/Hydrate):

- Compile to WebAssembly for client, native code for server
- Use `cfg!(feature = "hydrate")` or `cfg!(target_family = "wasm")` for platform-specific code
- Effects typically don't run on server during SSR

### State Management Patterns

- **Local**: `create_signal` within component
- **Prop Drilling**: Acceptable for shallow trees
- **Context API**: Preferred for global state (`provide_context`, `use_context`)
- **Global**: Top-level or lazy static initialization (use caution in WebAssembly)

### Navigation

Use `<A>` component instead of `<a>` for client-side navigation. Hooks: `use_params()`, `use_query()`, `use_navigate()`.

### Error Handling

- Server functions return `Result<T, ServerFnError>`
- Wrap error-prone components in `<ErrorBoundary>` with fallback
- Resources provide `.error()` signal for async error handling

### Common Pitfalls

- Almost all closures in `view!`/`create_effect` require `move ||`
- Clone signals before moving into closures
- Pass signals (ReadSignal) to child components, not values, for reactivity
- Use `NodeRef` for imperative DOM access, prefer declarative when possible
- Avoid panic-prone code directly in views
- Use `ErrorBoundary` for error handling

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
