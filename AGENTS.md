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

# Run single tests by name
cargo test create_card_use_case_should_create_card_and_save_to_database
cargo test rate_card_use_case_should_add_review_and_update_schedule
cargo test -p origa create_card_use_case_should_create_card_and_save_to_database

# Run integration tests
cargo test --test create_card
cargo test --test rate_card
cargo test --test delete_card
cargo test --test edit_card
cargo test --test start_study_session

# Run tests with filters
cargo test --workspace card
cargo test --workspace use_case
cargo test --workspace should_

# Lint and format
cargo clippy --workspace -- -D warnings
cargo fmt --check --all
cargo fmt --all

# Documentation
cargo doc --workspace --no-deps --open
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

#### Basic Patterns

- Use `signal()`, `create_memo`, `create_resource` for state management (note: Leptos 0.7 uses `signal()` not `create_signal`)
- Components are functions with `#[component]` attribute receiving props as arguments
- Use `view!` macro for UI rendering
- Access signal values with `.get()` and set with `.set()`
- Props must implement Clone + IntoView
- **IMPORTANT**: `children()` can only be called **once** per component. See "Working with `children()` in Components" section below for correct patterns.

#### Avoiding `FnOnce` vs `Fn` Closure Errors

Closures in `view!` macros must implement `Fn` (callable multiple times), not just `FnOnce`. Follow these patterns:

**✅ CORRECT: Extract Option values BEFORE view! macro**

```rust
#[component]
pub fn MyComponent(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] error: Option<String>,
) -> impl IntoView {
    // Extract values before view!
    let label_text = label.unwrap_or_default();
    
    view! {
        <div>
            <label>{label_text}</label>
        </div>
    }
}
```

**✅ CORRECT: Use .map() and .then() for conditional rendering**

```rust
view! {
    <div>
        // For Option<T> - use .map()
        {label.map(|lbl| view! {
            <label>{lbl}</label>
        })}
        
        // For bool - use .then()
        {show_error.then(|| view! {
            <div class="error">Error!</div>
        })}
        
        // For Option with nested logic
        {error.map(|err| view! {
            <div class="error">{err}</div>
        })}
    </div>
}
```

**✅ CORRECT: Clone values used in multiple closures**

```rust
// If a value is used in Signal::derive AND in view!, clone it
let error_clone = error.clone();
let has_error = Signal::derive(move || error_clone.is_some());

view! {
    <div>
        {error.map(|err| view! { <div>{err}</div> })}
    </div>
}
```

**❌ WRONG: Using Show with clones inside closures**

```rust
// Don't do this - causes FnOnce errors
view! {
    <Show when=move || label.is_some()>
        <label>{move || label.clone().unwrap_or_default()}</label>
    </Show>
}
```

#### Component Calling Patterns

**✅ CORRECT: Call components inside view! macro**

```rust
view! {
    <MyComponent
        label="Hello"
        value=some_signal
        on_click=callback
    />
}
```

**❌ WRONG: Cannot call components as functions with multiple arguments**

```rust
// This DOES NOT WORK in Leptos 0.7
MyComponent(
    label,
    value,
    on_click,
)
```

#### Passing Optional Props Between Components

When creating wrapper components that forward optional props, either:

1. **Extract and pass non-optional values:**

```rust
#[component]
pub fn Wrapper(
    #[prop(into, optional)] label: Option<String>,
) -> impl IntoView {
    let label_text = label.unwrap_or_default();
    
    view! {
        <ChildComponent label=label_text />
    }
}
```

2. **Duplicate the component logic** (preferred for simple cases):

```rust
#[component]
pub fn Textarea(
    #[prop(into, optional)] label: Option<String>,
) -> impl IntoView {
    // Implement textarea-specific logic here
    // rather than trying to wrap Input component
    view! { <textarea /> }
}
```

#### Working with `children()` in Components

**CRITICAL**: `children()` can only be called **once** per component. Calling it inside reactive closures (like `move || ... .then()`) causes `FnOnce` errors because the closure can be called multiple times.

**✅ CORRECT: Call `children()` ONCE outside reactive closures (PREFERRED)**

This is the recommended pattern for modals, bottom sheets, and other conditionally-rendered components:

```rust
#[component]
pub fn BottomSheet(
    show: Signal<bool>,
    #[prop(into, optional)] title: Option<String>,
    children: Children,
) -> impl IntoView {
    // Extract optional values before view!
    let title_text = title;
    
    // CRITICAL: Call children() ONCE outside any reactive closure
    let children_view = children();
    
    // Create event handlers outside view! macro
    let handle_close = move |_| {
        // Handle close logic
    };
    
    view! {
        // Use CSS classes for visibility instead of conditional rendering
        <div
            class=move || if show.get() { "modal-overlay modal-visible" } else { "modal-overlay modal-hidden" }
            on:click=handle_close
        >
            <div class="modal-content">
                {title_text.map(|t| view! {
                    <h1>{t}</h1>
                })}
                <div class="modal-body">
                    {children_view}  // ✅ Use the stored view, not children()
                </div>
            </div>
        </div>
    }
}
```

**Why this works**: 
- `children()` is called once when the component initializes
- The result is stored in a variable and reused
- CSS classes control visibility instead of conditional DOM rendering
- No `FnOnce` errors because `children()` is never called inside a closure

**CSS for visibility control**:

```scss
.modal-overlay {
  transition: opacity var(--duration-fast), visibility var(--duration-fast);
}

.modal-overlay.modal-visible {
  opacity: 1;
  visibility: visible;
  pointer-events: auto;
}

.modal-overlay.modal-hidden {
  opacity: 0;
  visibility: hidden;
  pointer-events: none;
}
```

**✅ ALTERNATIVE: Use `.then()` pattern with cloning (for simple cases)**

Only use this if you cannot use CSS-visibility pattern:

```rust
view! {
    {move || show.get().then(|| {
        // IMPORTANT: Clone ALL values that will be used in nested closures
        let on_close_local = on_close;
        let title_local = title.clone();
        let subtitle_local = subtitle.clone();
        
        // CRITICAL: Call children() INSIDE .then() block, not outside
        let children_view = children();
        
        view! {
            <div on:click=move |_| {
                if let Some(handler) = on_close_local {
                    handler.run(());
                }
            }>
                <h1>{title_local.clone()}</h1>
                {subtitle_local.map(|sub| view! { <p>{sub}</p> })}
                {children_view}  // ✅ Use stored view
            </div>
        }
    })}
}
```

**Why this works**: When you clone values INSIDE the `.then()` block (before they're captured by nested closures), the outer `move ||` closure becomes `FnMut` instead of `FnOnce`. The `children()` call happens inside `.then()`, so it's only executed when the condition is true.

**❌ WRONG: Calling `children()` inside reactive closure**

```rust
// This causes FnOnce errors!
view! {
    {move || show.get().then(|| view! {
        <div>
            {children()}  // ❌ FnOnce error! children() can only be called once
        </div>
    })}
}
```

**❌ WRONG: Calling `children()` multiple times**

```rust
// This causes FnOnce errors!
let children1 = children();  // First call
view! {
    {move || show.get().then(|| {
        let children2 = children();  // ❌ Second call - ERROR!
        view! { {children1} }
    })}
}
```

**❌ WRONG: Using Show component with children()**

```rust
// This causes FnOnce errors
view! {
    <Show when=move || show.get()>
        <div>
            {children()}  // ❌ FnOnce error!
        </div>
    </Show>
}
```

**❌ WRONG: Cloning values OUTSIDE .then()**

```rust
// This still causes FnOnce errors!
let title_local = title.clone();
let children_view = children();  // Called outside
view! {
    {move || show.get().then(|| view! {
        <h1>{title_local}</h1>  // ❌ Captured from outside!
        {children_view}  // ❌ Captured from outside!
    })}
}
```

#### Callback and Signal Types are Copy

**Important**: In Leptos 0.7, `Callback<T>`, `Signal<T>`, `ReadSignal<T>`, and `WriteSignal<T>` implement `Copy`.

**✅ CORRECT: No need to clone**

```rust
let handle_close = Callback::new(move |_| {
    if let Some(handler) = on_close {  // on_close is Copy
        handler.run(());
    }
    set_value.set("");  // set_value is Copy
});
```

**❌ WRONG: Don't use .clone() on Copy types**

```rust
// Unnecessary - these types are Copy
let on_close_clone = on_close.clone();  // ❌ Not needed
let set_value_clone = set_value.clone();  // ❌ Not needed
```

#### Using Callback vs Closures in Event Handlers

**Important**: For `on:click` and other event handlers, use closures directly, not `Callback::new()`.

**✅ CORRECT: Use closures for `on:click` handlers**

```rust
// Define handler as closure
let handle_back = move |_| {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/");
    }
};

view! {
    <button on:click=handle_back>
        Back
    </button>
}
```

**✅ CORRECT: Use inline closures**

```rust
view! {
    <button on:click=move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/");
        }
    }>
        Back
    </button>
}
```

**✅ CORRECT: Use Callback with `.run()` when passing to components**

```rust
// For component props that expect Callback<T>
let handle_next = Callback::new(move |_| {
    set_current_index.set(current_index.get() + 1);
});

view! {
    <NextButton on_click=handle_next />
}

// Inside component, use .run() to invoke
view! {
    <button on:click=move |_| handle_next.run(())>
        Next
    </button>
}
```

**❌ WRONG: Using Callback directly in `on:click`**

```rust
let handle_back = Callback::new(move |_| { /* ... */ });

view! {
    <button on:click=handle_back>  // ❌ Error: expected FnMut(MouseEvent), found Callback
        Back
    </button>
}
```

**Rule of thumb**: 
- Use closures (`move |_| { ... }`) for HTML element event handlers (`on:click`, `on:input`, etc.)
- Use `Callback<T>` for component props that expect callbacks
- When using `Callback` in component props, invoke with `.run()` inside event handlers

#### Passing Props to Components

**Important**: Don't wrap simple expressions in parentheses when passing as props. Leptos can infer types correctly.

**✅ CORRECT: Direct values without parentheses**

```rust
<StepIndicator
    current=Signal::derive(move || Some(current_index.get()))
    total=total_cards as u32
    active=Signal::derive(move || !is_completed.get())
/>

<StudyNavigation
    show_next=!is_completed.get() && !show_answer.get()
    next_disabled=show_answer.get() || is_completed.get()
    audio_enabled=audio_enabled.get()
/>
```

**❌ WRONG: Unnecessary parentheses**

```rust
// ❌ Don't wrap simple expressions
<StepIndicator
    total=(total_cards as u32)  // ❌ Unnecessary parentheses
    show_next=(!is_completed.get() && !show_answer.get())  // ❌ Unnecessary
/>
```

**When to use parentheses**: Only when needed for operator precedence or complex expressions:
```rust
total=(total_cards as u32 + offset)  // ✅ Needed for precedence
```

#### Component Props: Simple Types vs `impl IntoView`

**Important**: Prefer concrete types (`Signal<T>`, `u32`, `bool`, `String`) over `impl IntoView` for component props when possible. This provides better type safety and clearer APIs.

**✅ CORRECT: Use concrete types**

```rust
#[component]
pub fn StepIndicator(
    current: Signal<Option<usize>>,  // ✅ Concrete type
    total: u32,                      // ✅ Concrete type
    active: Signal<bool>,            // ✅ Concrete type
) -> impl IntoView {
    // Implementation
}
```

**❌ WRONG: Overly generic `impl IntoView`**

```rust
#[component]
pub fn StepIndicator(
    #[prop(into, optional)] current: Option<impl IntoView + Clone + 'static>,  // ❌ Too generic
    #[prop(into)] total: impl IntoView + Clone + 'static,                       // ❌ Too generic
) -> impl IntoView {
    // Hard to extract actual values, causes type inference issues
}
```

**When to use `impl IntoView`**: Only when you need to accept multiple different view types (e.g., text, numbers, components) in the same prop.

#### Using `#[derive(Default)]` for Enums

**Important**: Use `#[derive(Default)]` with `#[default]` attribute instead of manual `impl Default`.

**✅ CORRECT: Use derive macro**

```rust
#[derive(Clone, Copy, PartialEq, Default)]
pub enum CircularSize {
    Small,
    #[default]
    Medium,
    Large,
}
```

**❌ WRONG: Manual implementation**

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum CircularSize {
    Small,
    Medium,
    Large,
}

impl Default for CircularSize {
    fn default() -> Self {
        CircularSize::Medium  // ❌ Prefer derive macro
    }
}
```

#### Pattern Matching in Signal::derive

**Important**: Use simple tuple destructuring in pattern matching, not nested parentheses.

**✅ CORRECT: Simple tuple destructuring**

```rust
let progress_percent = Signal::derive(move || {
    if let (Some(c), Some(t)) = (current, total) {  // ✅ Simple tuple
        // ...
    }
});
```

**❌ WRONG: Nested parentheses**

```rust
if let ((Some(c), Some(t))) = (current, total) {  // ❌ Unnecessary nested parentheses
    // ...
}
```

#### Reactive Attributes vs Static Values

**Important**: Attributes like `disabled`, `class`, etc. can be either static or reactive.

**✅ CORRECT: Use closure for reactive values**

```rust
<button
    disabled=move || is_loading.get()  // Reactive
    class=move || if active.get() { "active" } else { "" }  // Reactive
>
```

**❌ WRONG: Calling .get() once makes it static**

```rust
<button
    disabled=is_loading.get()  // ❌ Evaluated once, not reactive!
>
```

#### Numeric Type Suffixes in Props

**Important**: When passing numeric literals to props, use explicit type suffixes to avoid type inference errors.

**✅ CORRECT: Use type suffixes**

```rust
<Input
    maxlength=50u32   // Explicit u32
    rows=3u32         // Explicit u32
/>
```

**❌ WRONG: Implicit i32 literals**

```rust
<Input
    maxlength=50   // ❌ Error: cannot convert i32 to u32
    rows=3         // ❌ Error: cannot convert i32 to u32
/>
```

**Common type suffixes**:
- `u32` - unsigned 32-bit integer
- `i32` - signed 32-bit integer  
- `u64`, `i64` - 64-bit integers
- `f32`, `f64` - floating point

#### Moving Values in For Loops

When using `For` component, clone ALL fields you'll need BEFORE using them in closures:

**✅ CORRECT:**

```rust
<For
    each=move || items.get()
    key=|item| item.id.clone()
    children=move |item| {
        // Clone everything first
        let id = item.id.clone();
        let id_for_signal = item.id.clone();
        let name = item.name.clone();
        let value = item.value;
        
        let is_active = Signal::derive(move || selected.get() == id_for_signal);
        
        view! {
            <button on:click=move |_| handle_click(id.clone())>
                {name}
            </button>
        }
    }
/>
```

#### Reference Examples

See these files for correct patterns:
- `origa_ui/src/components/forms/bottom_sheet.rs` - **correct `children()` usage with CSS-visibility pattern** (PREFERRED)
- `origa_ui/src/components/layout/app_layout.rs` - using .map() and .then()
- `origa_ui/src/components/layout/tab_bar.rs` - component calling in view!
- `origa_ui/src/components/forms/input.rs` - handling multiple optional props
- `origa_ui/src/components/forms/chip_group.rs` - cloning values in For loops
- `origa_ui/src/pages/study.rs` - **using closures in `on:click`, passing props without parentheses**
- `origa_ui/src/components/interactive/progress_bar.rs` - **concrete types in props (`Signal<T>`, `u32`), `#[derive(Default)]`**

### Testing

- Use `rstest` for parameterized tests when needed
- Use `#[tokio::test]` for async tests
- Keep tests isolated; use temporary directories for database tests
- Use descriptive test names: `should_create_card_and_persist_to_database`
- Group test utilities in `tests/mod.rs`
- Test utilities available: `create_test_repository()`, `create_test_user()`

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
- `lindera = { version = "2.0", features = ["embed-unidic"] }` - Japanese tokenizer
- `async-openai-wasm` - OpenAI API client (WASM compatible)
- `dioxus-heroicons` - Icon components
- `rstest` - Testing framework

## Development Tips

1. **Debug logging**: Use `tracing::info!`, `tracing::debug!` macros
2. **Database inspection**: SQLite files are in user config directory; use `sqlite3` CLI to inspect
3. **Web development**: Run `cargo trunk watch` in `origa_ui/` directory for hot reload
4. **Desktop**: Build with Tauri for native window, tray icon, menu bar integration
5. **State management**: Use Leptos signals for local state, context provider for global state

## Common Operations

- Add new dependencies to root Cargo.toml [workspace.dependencies]
- Create use cases in `origa/src/application/use_cases/` and add to mod.rs
- Add UI views in `origa_ui/src/pages/` and routes in main.rs
- Add UI components in `origa_ui/src/components/`

## Leptos Resources

- Official docs: <https://book.leptos.dev/>
- Router: <https://book.leptos.dev/view/09_router.html>
- State management: <https://book.leptos.dev/view/02_reactivity.html>
