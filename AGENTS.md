# AGENTS.md - Origa Development Guide

## Project Overview

Origa — приложение для изучения японского языка с интервальными повторениями (FSRS),
OCR и токенизацией.
**Tech stack**: Rust workspace (крейты `origa`, `origa_ui`, `tokenizer`),
Leptos/WASM, Tauri v2.
**Архитектура**: Clean Architecture (Use Cases → Domain → Traits).

## Быстрые команды для начала работы

### Настройка

```bash
cargo build --release
```

### Разработка (Tauri desktop)

```bash
cd tauri && cargo tauri dev
```

### Сборка

```bash
cd tauri && cargo tauri build
```

### Тестирование

```bash
# Все тесты
cargo test --workspace

# Тесты конкретного крейта
cargo test -p origa
cargo test -p origa_ui

# Один тестовый файл
cargo test -p origa --test test_name
```

### Линтинг и форматирование

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Стиль кода и соглашения

### Форматирование

Rustfmt использует настройки по умолчанию. Нет кастомного `rustfmt.toml`.

### Именование

- Крейты: kebab-case (`origa_ui`, `rs_fsrs`)
- Типы/модули: snake_case
- Функции/переменные: snake_case
- Enum variants: PascalCase

### Обработка ошибок

- Используйте `thiserror` для ошибок домена
- Никогда не используйте `unwrap()` в production-коде
- Никогда не используйте `#[async_trait]` — используйте async fn напрямую

### Комментарии и документация

- `// TODO:` — только для незавершённой работы
- `///` (doc comments) — только когда действительно нужны
- Код должен быть самодокументируемым через понятные имена
- Никогда не используйте `#[allow(dead_code)]`

### Логирование

- Используйте `tracing` для логирования
- Никогда не оставляйте `println!` или `console.log` в коде

## Структура проекта

```
origa/
├── origa/                  # Бизнес-логика (domain, use_cases, traits, ocr, dictionary)
├── origa_ui/              # Leptos frontend (WASM)
├── tauri/                  # Tauri v2 desktop app
├── end2end/                # Playwright E2E тесты
└── utils/                  # CLI утилиты (api, commands)
```

## Git Workflow

- Default branch: `master`
- Commit: использовать `@git-commit-push` subagent

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

Статический контент (словари, вокабуляр, грамматика, кандзи,
OCR модели, SVG, аудио фраз) хранится на
Railway Storage (S3-совместимый).

### Доступ через AWS CLI

Для работы с CDN используется `aws` CLI:

```bash
# Env vars для доступа
export AWS_ACCESS_KEY_ID="tid_..."
export AWS_SECRET_ACCESS_KEY="tsec_..."
export AWS_DEFAULT_REGION="auto"
export AWS_ENDPOINT_URL="https://t3.storageapi.dev"

BUCKET="s3://stored-breadbox-fk2diaux"

# Загрузить данные
aws s3 sync origa_ui/public/dictionary $BUCKET/dictionary
aws s3 sync origa_ui/public/dictionaries $BUCKET/dictionaries
aws s3 sync origa_ui/public/grammar $BUCKET/grammar
aws s3 sync origa_ui/public/domain $BUCKET/domain
aws s3 sync origa_ui/public/ndlocr $BUCKET/ndlocr
aws s3 sync origa_ui/public/kanji_animations $BUCKET/kanji_animations
aws s3 sync origa_ui/public/kanji_frames $BUCKET/kanji_frames

# Просмотр
aws s3 ls $BUCKET/ --recursive

# Удалить
aws s3 rm $BUCKET/path/to/file
```

### Что на CDN

| Путь | Описание |
| --- | --- |
| `dictionary/` | Кандзи, радикалы, вокабуляр (JSON chunks) |
| `dictionaries/` | UniDic токенизатор (бинарные файлы) |
| `grammar/` | Грамматические правила |
| `domain/` | Well-known sets (JLPT, Duolingo, Migii, etc.) |
| `ndlocr/` | OCR модели (ONNX) |
| `kanji_animations/` | SVG анимации порядка начертания |
| `kanji_frames/` | SVG кадры порядка начертания |
| `phrases/` | Индекс фраз + чанки + аудио |

### Что остаётся локально (`origa_ui/public/`)

- `auth/` — OAuth callback
- `logo*.png` — логотипы приложения
- `external_icons/` — иконки брендов
- `ort/` — ONNX Runtime WASM (SharedArrayBuffer + COOP/COEP)

### Доступ из приложения

Все CDN-запросы идут через SigV4 presigned URLs (приватный бакет).
См. `origa/src/traits/cdn_provider.rs` (трейт)
и `origa_ui/src/repository/cdn_provider.rs`
(реализация: CacheApi → CDN fallback).

## Развёртывание

CI/CD автоматический через GitHub Actions:

- Push в `master` или тег `v*.*.*` запускает сборку
- Сборки: Windows (NSIS), Linux (AppImage, DEB), macOS
- Артефакты публикуются в GitHub Releases
