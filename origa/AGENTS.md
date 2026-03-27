# AGENTS.md - Origa Development Guide

Origa — приложение для изучения японского языка с интервальными повторениями (FSRS),
OCR и токенизацией.
**Tech stack**: Rust workspace (`origa`, `origa_ui`, `tokenizer`), Leptos/WASM, Tauri v2.
**Архитектура**: Clean Architecture (Use Cases → Domain → Traits).

## Code Style

- Rustfmt по умолчанию (`cargo make fmt-check` / `cargo make fmt`)
- `// TODO:` — только для незавершённой работы
- `///` doc comments — только когда действительно нужны

## Project Structure

```
origa/
├── origa/       # Бизнес-логика (domain, use_cases, traits, ocr)
├── origa_ui/    # Leptos frontend (components, pages, repository)
├── tauri/       # Tauri v2 desktop
├── end2end/     # Playwright E2E
└── utils/       # Утилиты
```
