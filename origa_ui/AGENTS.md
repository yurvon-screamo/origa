# AGENTS.md - Origa Frontend (`origa_ui` crate)

## UI Components

### test_id Паттерн (обязательно для всех интерактивных компонентов)

Все интерактивные компоненты **должны** получать `test_id: Signal<String>`
prop.

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

## Project Structure

```
origa_ui/
└── src/
    ├── components/     # Переиспользуемые UI компоненты
    ├── pages/          # Страницы приложения
    └── repository/     # TrailBase client
```
