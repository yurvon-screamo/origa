# AGENTS.md - Origa Development Guide

## Project Overview

Origa — приложение для изучения японского языка с интервальными повторениями (FSRS),
OCR и токенизацией.
**Tech stack**: Rust workspace (крейты `origa`, `origa_ui`, `tokenizer`),
Leptos/WASM, Tauri v2.
**Архитектура**: Clean Architecture (Use Cases → Domain → Traits).

### Comments & Documentation

- `// TODO:` — только для незавершённой работы
- Код должен быть самодокументируемым через понятные имена
- `///` (doc comments) — только когда действительно нужны

## Project Structure

```
origa/
├── origa/                  # Бизнес-логика
├── origa_ui/               # Leptos frontend (WASM)
├── tauri/                  # Tauri v2 desktop
├── end2end/                # Playwright E2E тесты
└── utils/                  # Утилиты
```

## Git Workflow

- Default branch: `master`

## Commit

Использовать `@git-commit-push` subagent.

## Critical Boundaries

### ⚠️ ASK FIRST

- Изменения в `Cargo.toml`
- Изменения в CI/CD
- Изменения в domain layer

### 🚫 NEVER DO

- Коммитить без прохождения всех тестов
- Использовать `unwrap()` в production-коде
- Использовать `#[async_trait]` и `#[allow(dead_code)]`
- Коммитить `console.log` / `println!`
- Удалять тесты
