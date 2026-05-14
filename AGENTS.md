# AGENTS.md - Origa Development Guide

## Project Overview

Origa — приложение для изучения японского языка с интервальными повторениями (FSRS),
OCR и токенизацией.
**Tech stack**: Rust workspace (крейты `origa`, `origa_ui`, `tokenizer`),
Leptos/WASM, Tauri v2.
**Архитектура**: Clean Architecture (Use Cases → Domain → Traits).

## Структура проекта

- `origa/` — бизнес-логика (domain, use_cases, traits, ocr, dictionary)
- `origa_ui/` — Leptos frontend (WASM)
- `tauri/` — Tauri v2 desktop app
- `end2end/` — Playwright E2E тесты
- `utils/` — CLI утилиты

## Команды

### Разработка

```bash
cd tauri && cargo tauri dev
```

### Тестирование

```bash
# Все тесты
cargo test --workspace

# Тесты конкретного крейта
cargo test -p origa
cargo test -p origa_ui
```

## Стиль кода и соглашения

### Обработка ошибок

- Используйте `thiserror` для ошибок домена
- Никогда не используйте `unwrap()` в production-коде
- Никогда не используйте `#[async_trait]` — используйте async fn напрямую

### Комментарии и документация

- `///` (doc comments) — только когда действительно нужны
- Код должен быть самодокументируемым через понятные имена
- Никогда не используйте `#[allow(dead_code)]`

### Логирование

- Используйте `tracing` для логирования
- Никогда не оставляйте `println!` или `console.log` в коде

## Критические границы

### ✅ ВСЕГДА делайте

- Запускайте `cargo clippy --workspace --all-targets -- -D warnings` перед коммитом
- Форматируйте код через `cargo fmt` перед коммитом
- Проходите все тесты (`cargo test --workspace`) перед коммитом

### ⚠️ СПРОСИТЕ СНАЧАЛА

- Изменения в `Cargo.toml` (workspace dependencies)
- Изменения в CI/CD (`.github/workflows/`)
- Изменения в domain layer (`origa/src/domain/`)

### 🚫 НИКОГДА не делайте

- Не коммитьте без прохождения всех тестов
- Не используйте `unwrap()` в production-коде
- Не используйте `#[async_trait]` и `#[allow(dead_code)]`
- Не коммитьте `println!` / `console.log`
- Не удаляйте тесты

## CDN / S3 Storage

Статический контент (словари, вокабуляр, грамматика, кандзи, OCR модели, SVG, аудио фраз)
хранится на Yandex Cloud Storage (S3-совместимый). Bucket: `s3://origa-data`.

CDN URL бейкается в WASM на этапе компиляции через `build.rs`.
Для запуска dev: `$env:ORIGA_CDN_BASE_URL = "https://storage.yandexcloud.net/origa-data"`.

Управление CDN через `aws s3 sync` с профилем `yandex`, endpoint `https://storage.yandexcloud.net`.

CDN трейт: `origa/src/traits/cdn_provider.rs`, реализация: `origa_ui/src/repository/cdn_provider.rs`.
