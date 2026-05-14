# AGENTS.md - Origa Frontend (`origa_ui` crate)

## Description

Leptos/WASM frontend. Built with `trunk`.

For Tauri desktop development, use `cargo tauri dev`.

## Project Structure

```text
origa_ui/src/
├── core/               # Core: routing, application context
├── ui_components/     # Base UI components (buttons, cards, etc.)
├── pages/             # Pages (home, lesson, sets, profile, login, etc.)
├── repository/        # TrailBase/IndexedDB client (async data)
├── store/             # Leptos reactive state (Romer)
└── loaders/           # Async data loaders
```

## UI Components

### test_id Pattern (required for all interactive components)

All interactive components **must** accept a `test_id: Signal<String>` prop.

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

## Key Conventions

### Props

- All optional props with `#[prop(optional, into)]`
- Signal for reactive props: `test_id: Signal<String>`
- Never use `web_sys` directly for DOM

### Async Data

- Use `create_resource` for server data
- Repository pattern via `Repository` trait
- Loader functions in `loaders/`

### Logging

```rust
// ✅ Good: tracing for WASM
tracing::info!("Card loaded: {id}");

// ❌ Bad: console.log
web_sys::console::log_1(&"message".into());
```

## Development

```bash
# Frontend only (WASM)
cd origa_ui && trunk serve

# Tauri desktop (full app)
cd tauri && cargo tauri dev

# Production build
cd origa_ui && trunk build --release
```

## Testing

```bash
cargo test -p origa_ui
```
