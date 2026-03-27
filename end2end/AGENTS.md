# AGENTS.md - Origa E2E Testing (`end2end`)

## E2E Testing (Playwright)

Сервер запускается автоматически, вручную, отдельно от тестов, запускать не нужно.

## Project Structure

```
end2end/
├── config.ts        # Конфигурация тестовых пользователей
├── .env             # Локальные переменные окружения (gitignored)
└── tests/           # Playwright тесты
```
