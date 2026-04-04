# AGENTS.md - Origa E2E Testing (`end2end`)

## Описание

Playwright E2E тесты. TypeScript + @playwright/test.

## Быстрые команды

```bash
# Все тесты
npm test

# Headed режим (видимый браузер)
npm test:headed

# Только Chrome
npm test:chrome

# Показать отчёт
npm report
```

## Важно

- Сервер (Tauri/TrailBase) запускается **автоматически** перед тестами через `global-setup.ts`
- Вручную запускать сервер **не нужно**
- Тесты работают в headless режиме по умолчанию

## Структура проекта

```text
end2end/
├── config.ts           # Конфигурация тестовых пользователей
├── global-setup.ts     # Запуск сервера перед тестами
├── playwright.config.ts # Playwright конфигурация
├── pages/              # Page objects
├── tests/               # Playwright тесты
└── .env                 # Локальные переменные (gitignored)
```

## Page Objects

Используйте паттерн Page Object из `pages/`:

```typescript
import { Page } from '@playwright/test';

export class LoginPage {
  constructor(private page: Page) {}

  async login(user: TestUser) {
    await this.page.fill('[data-testid="username"]', user.username);
    await this.page.fill('[data-testid="password"]', user.password);
    await this.page.click('[data-testid="login-button"]');
  }
}
```

## Конфигурация

Тестовые пользователи в `config.ts`:

```typescript
export const TEST_USERS = {
  admin: { username: 'admin', password: '...' },
  regular: { username: 'user', password: '...' },
};
```
