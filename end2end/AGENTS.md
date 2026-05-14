# AGENTS.md - Origa E2E Testing (`end2end`)

## Description

Playwright E2E tests. TypeScript + @playwright/test.

## Quick Commands

```bash
# All tests
npm test

# Headed mode (visible browser)
npm test:headed

# Chrome only
npm test:chrome

# Show report
npm report
```

## Important

- The server (Tauri/TrailBase) is started **automatically** before tests via `global-setup.ts`
- You do **not** need to start the server manually
- Tests run in headless mode by default

## Project Structure

```text
end2end/
├── config.ts           # Test user configuration
├── global-setup.ts     # Server startup before tests
├── playwright.config.ts # Playwright configuration
├── pages/              # Page objects
├── tests/               # Playwright tests
└── .env                 # Local variables (gitignored)
```

## Page Objects

Use the Page Object pattern from `pages/`:

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

## Configuration

Test users in `config.ts`:

```typescript
export const TEST_USERS = {
  admin: { username: 'admin', password: '...' },
  regular: { username: 'user', password: '...' },
};
```
