# AGENTS.md — origa_landing

SSR-лендинг (Leptos 0.8 + Axum). Общие правила репо, SSR-тесты и env — в корневом `AGENTS.md`.

## `style/landing.processed.css` — закоммиченный артефакт сборки

`landing.processed.css` — результат сборки tailwind из `style/landing.css`.
Артефакт **коммитится** в репо. Docker-образ и прод отдают именно закоммиченный
файл: `build.rs` пересобирает его только при mtime-дрейфе исходников, а в свежем
clone mtime совпадают — пересборка всегда скипается. CSS отдаётся с
`immutable` + cache-busting тег `?v=` (`LANDING_CSS_VERSION`, FNV-хеш артефакта
в `build.rs`), так что протухший артефакт = браузеры держат старый CSS.

### После правок `style/landing.css`, `style/input.css` или `tailwind.config.js`

Пересобери артефакт и закоммить его **в том же PR**:

```sh
cd origa_landing
npm ci            # или bun install --frozen-lockfile
just css          # рецепт Justfile: npx tailwindcss --input style/landing.css --output style/landing.processed.css --minify
git add style/landing.processed.css
```

Проверка свежести локально — чистый git, node-тулинг не нужен (смысл имеет
сразу после пересборки):

```sh
git diff --exit-code -- style/landing.processed.css   # пусто = артефакт свеж
```

CI-гейт: джоба `Test (landing SSR)` (`ci.yml`) пересобирает CSS командой
`npm ci && npm run build:css` и падает на `git diff --exit-code`, если
закоммиченный артефакт разошёлся с исходником. PR с протухшим артефактом
не мержится.

### Версия tailwindcss

Спека в `package.json` — `^3.4.0`; точную версию пинят **оба** lock-файла,
`package-lock.json` (CI) и `bun.lock` (локальные сборки). Держи их синхронными:
если резолвы разойдутся, локально «свежий» артефакт упадёт в CI. При бампе
версии — пересобери и закоммить артефакт.
