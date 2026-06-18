# AGENTS.md — Origa

**Origa** — приложение для изучения японского языка (FSRS, OCR, STT, токенизация).
Репозиторий: <https://github.com/yurvon-screamo/origa>

## Стек

| Слой           | Технология                                                                |
|----------------|---------------------------------------------------------------------------|
| Workspace      | Rust 2024 edition, id `net.uwuwu.origa`                                   |
| Бизнес-логика  | `origa/` — Clean Architecture (Use Cases → Domain → Traits)               |
| Frontend       | `origa_ui/` — Leptos 0.8, CSR/WASM, trunk                                 |
| Landing        | `origa_landing/` — Leptos 0.8, SSR/Axum, i18n (EN+RU)                    |
| Desktop        | `tauri/` — Tauri v2 (Windows, Linux, macOS)                               |
| E2E            | `end2end/` — Playwright (TypeScript)                                      |
| Утилиты        | `utils/`, `scripts/` (Python)                                             |

## Структура проекта

```text
origa/          — domain, use_cases, traits, ocr, stt, dictionary
origa_ui/       — Leptos 0.8 frontend (WASM)
origa_landing/  — SSR landing site (Leptos 0.8 + Axum)
tauri/          — Tauri v2 desktop app
end2end/     — Playwright E2E тесты
utils/       — CLI утилиты
cdn/         — статический контент (dictionaries, grammar, kanji_animations, ndlocr, phrases, pitch, well_known_set)
scripts/     — Python скрипты обработки данных
docs/        — документация (decisions/)
models/      — ML модели
```

## Среда разработки

```powershell
$env:ORIGA_CDN_BASE_URL = "https://s3-proxy-production-52e3.up.railway.app"  # ОБЯЗАТЕЛЬНО
cd tauri && cargo tauri dev
```

### Переменные окружения (compile-time, `build.rs`)

Обязательные: `ORIGA_CDN_BASE_URL`.
Опциональные: `ORIGA_CDN_REGION`, `ORIGA_VERSION`, `ORIGA_COMMIT`, `ORIGA_BUILD_DATE`, `TRAILBASE_URL`, `ORIGA_LANDING_BASE_URL`.

**DNS naming scheme** (CI/CD production):

- `ORIGA_BASE_URI` — base domain (e.g. `origa.uwuwu.net`)
- `ORIGA_CDN_URI_PREFIX` — CDN subdomain prefix (e.g. `s3` → `s3.origa.uwuwu.net`)
- `ORIGA_APP_URI_PREFIX` — app subdomain prefix (e.g. `app` → `app.origa.uwuwu.net`)
- Landing = base domain (no prefix)

**Local dev:** `$env:ORIGA_CDN_BASE_URL = "https://s3-proxy-production-52e3.up.railway.app"`
**Landing dev:** `$env:ORIGA_LANDING_BASE_URL = "https://origa.app"`

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

T3 Storage (`s3://adaptable-foodbox-ucep7wx`), CDN URL вшивается через `build.rs`.
Трейт: `origa/src/traits/cdn_provider.rs`, реализация: `origa_ui/src/repository/cdn_provider.rs`.

Все объекты — статические и immutable, поэтому `Cache-Control: public, max-age=31536000, immutable`.

```powershell
python scripts/deploy_cdn.py            # генерация манифеста + инкрементальный деплой
python scripts/deploy_cdn.py --dry-run  # показать что будет залито
```

Манифест (`manifest.json`) содержит SHA256 хеши 32 версионных файлов и позволяет клиенту обнаруживать обновления. Деплоится с `Cache-Control: no-cache` (единственное исключение из immutable-политики).

Обновить Cache-Control на существующих объектах (one-time):

```powershell
aws s3 cp s3://adaptable-foodbox-ucep7wx/ s3://adaptable-foodbox-ucep7wx/ --profile origa --endpoint-url https://t3.storageapi.dev --recursive --metadata-directive REPLACE --cache-control "public, max-age=31536000, immutable"
```

## CI/CD

Workflows: `ci.yml`, `docker.yml`, `tauri.yml`, `cleanup-cache.yml`.
CI: lint + test + e2e + docker build (2 images: landing + ui).
CD: 2 Docker images (GHCR) + Railway deploy (2 services).
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
