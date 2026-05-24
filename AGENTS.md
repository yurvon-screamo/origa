# AGENTS.md — Origa

**Origa** — приложение для изучения японского языка (FSRS, OCR, STT, токенизация).
Репозиторий: <https://github.com/yurvon-screamo/origa>

## Стек

| Слой           | Технология                                                                |
|----------------|---------------------------------------------------------------------------|
| Workspace      | Rust 2024 edition, id `net.uwwu.origa`                                    |
| Бизнес-логика  | `origa/` — Clean Architecture (Use Cases → Domain → Traits)               |
| Frontend       | `origa_ui/` — Leptos 0.8, CSR/WASM, trunk                                 |
| Desktop        | `tauri/` — Tauri v2 (Windows, Linux, macOS)                               |
| E2E            | `end2end/` — Playwright (TypeScript)                                      |
| Утилиты        | `utils/`, `scripts/` (Python)                                             |

## Структура проекта

```text
origa/       — domain, use_cases, traits, ocr, stt, dictionary
origa_ui/    — Leptos 0.8 frontend (WASM)
tauri/       — Tauri v2 desktop app
end2end/     — Playwright E2E тесты
utils/       — CLI утилиты
cdn/         — статический контент (dictionaries, grammar, kanji_animations, ndlocr, phrases, pitch, well_known_set)
scripts/     — Python скрипты обработки данных
docs/        — документация (decisions/)
models/      — ML модели
```

## Среда разработки

```powershell
$env:ORIGA_CDN_BASE_URL = "https://storage.yandexcloud.net/origa-data"  # ОБЯЗАТЕЛЬНО
cd tauri && cargo tauri dev
```

### Переменные окружения (compile-time, `build.rs`)

Обязательная: `ORIGA_CDN_BASE_URL`. Опциональные: `ORIGA_CDN_REGION`, `ORIGA_VERSION`,
`ORIGA_COMMIT`, `ORIGA_BUILD_DATE`, `ORIGA_PUBLIC_BASE_URL`, `TRAILBASE_URL`.

## Команды

```powershell
cargo test --workspace                              # все тесты
cargo test -p origa -- --nocapture                  # с выводом
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check && cargo fmt
```

Тесты: `rstest` (параметризованные). Конфиги: `.rustfmt.toml` (max_width=100), `clippy.toml` (complexity=25).

## Ключевые зависимости

`rs-fsrs` (FSRS), `ort` + NDLOCR-Lite (OCR), `ort` + `rustfft` (Whisper STT),
`lindera` + UniDic (токенизация), `serde`/`bincode`/`rkyv` (сериализация),
`rusqlite` (БД), Leptos 0.8 + `leptos_router`/`leptos-use`/`leptos_i18n` (frontend),
`sha2`/`hmac` (TrailBase auth), `tracing`/`tracing-wasm` (логирование).
Плагины: opener, tts, deep-link (`origa://`), single-instance, updater, process.

## CDN / S3

Yandex Cloud Storage (`s3://origa-data`), CDN URL вшивается через `build.rs`.
Трейт: `origa/src/traits/cdn_provider.rs`, реализация: `origa_ui/src/repository/cdn_provider.rs`.

Все объекты — статические и immutable, поэтому `Cache-Control: public, max-age=31536000, immutable`.

```powershell
aws s3 sync cdn/ s3://origa-data --profile yandex --endpoint-url https://storage.yandexcloud.net --cache-control "public, max-age=31536000, immutable"
```

Обновить Cache-Control на существующих объектах (one-time):

```powershell
aws s3 cp s3://origa-data/ s3://origa-data/ --profile yandex --endpoint-url https://storage.yandexcloud.net --recursive --metadata-directive REPLACE --cache-control "public, max-age=31536000, immutable"
```

## CI/CD

Workflows: `tauri.yml`, `version.yml`, `reusable-release.yml`, `mobile.yml`, `docker.yml`, `cleanup-cache.yml`.
Targets: Windows x86_64, Linux x86_64, macOS aarch64. Релиз при push `master` + tag `v*.*.*`.

## Границы

### ✅ ВСЕГДА

- `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt` + `cargo test --workspace` перед коммитом
- `ORIGA_CDN_BASE_URL` установлена перед сборкой

### ⚠️ СПРОСИТЕ СНАЧАЛА

- Изменения в `Cargo.toml` (workspace deps), `.github/workflows/`, `origa/src/domain/`, линтер-конфигах

### 🚫 НИКОГДА

- Коммит без тестов / `unwrap()` в production / `#[async_trait]` / `#[allow(dead_code)]`
- `println!` / `console.log` в production / удаление тестов
- Sans-serif шрифты (только Cormorant Garamond + DM Mono)
- `border-radius` на основных UI / `box-shadow` с blur (только жёсткие offset-тени)

## Git

Коммиты на английском. Ветка: `master`. Теги: `v*.*.*` для релизов.
