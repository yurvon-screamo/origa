# AGENTS.md - Origa Frontend (`origa_ui` crate)

## Описание

Leptos/WASM фронтенд. Сборка через `trunk`. Для разработки Tauri desktop используйте `cargo tauri dev`.

## Структура проекта

```text
origa_ui/src/
├── core/               # Ядро: маршрутизация, контекст приложения
├── ui_components/     # Базовые UI компоненты (кнопки, карточки и т.д.)
├── pages/             # Страницы (home, lesson, sets, profile, login, etc.)
├── repository/        # TrailBase/IndexedDB клиент (async data)
├── store/             # Leptos reactive state (Romer)
└── loaders/           # Async data loaders
```

## UI Components

### test_id Паттерн (обязательно для всех интерактивных компонентов)

Все интерактивные компоненты **должны** получать `test_id: Signal<String>` prop.

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

## Ключевые соглашения

### Пропсы

- Все опциональные пропсы с `#[prop(optional, into)]`
- Signal для реактивных пропсов: `test_id: Signal<String>`
- Никогда не используйте `web_sys` напрямую для DOM

### Async данные

- Используйте `create_resource` для серверных данных
- Repository паттерн через `Repository` trait
- Loader функции в `loaders/`

### Логирование

```rust
// ✅ Хорошо: tracing для WASM
tracing::info!("Card loaded: {id}");

// ❌ Плохо: console.log
web_sys::console::log_1(&"message".into());
```

## Разработка

```bash
# Frontend only (WASM)
cd origa_ui && trunk serve

# Tauri desktop (full app)
cd tauri && cargo tauri dev

# Production build
cd origa_ui && trunk build --release
```

## Тестирование

```bash
cargo test -p origa_ui
```
