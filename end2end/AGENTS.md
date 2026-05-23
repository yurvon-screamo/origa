# AGENTS.md — Origa E2E Tests (`end2end/`)

Playwright E2E tests for Origa. TypeScript + `@playwright/test`.

## npm Scripts

```bash
npm test                     # All tests (headless)
npm run test:ui              # Playwright UI mode
npm run test:headed          # Visible browser
npm run test:debug           # Debug mode
npm run test:chrome          # Chromium only
npm run test:chrome:headed   # Chromium, visible browser
npm run test:chrome:ui       # Chromium, Playwright UI
npm run report               # HTML report (0.0.0.0:9323)
```

## Structure

```text
end2end/
├── config.ts                              # TEST_USERS
├── global-setup.ts / global-teardown.ts   # Auto server lifecycle
├── playwright.config.ts                   # Playwright config
├── pages/ {base,login,...}.page.ts → index.ts
├── tests/                                 # *.spec.ts
├── fixtures/                              # Auth, onboarding, test data
└── helpers/                               # Navigation, auth, HTTP, cleanup
```

## Page Object Pattern

All tests **must** use Page Objects from `pages/`. Select via `[data-testid="..."]` only — never CSS classes or text.

```typescript
import { Page } from '@playwright/test';
import { TestUser } from '../config';
export class LoginPage {
  constructor(private page: Page) {}
  async login(user: TestUser) {
    await this.page.fill('[data-testid="username"]', user.username);
    await this.page.fill('[data-testid="password"]', user.password);
    await this.page.click('[data-testid="login-button"]');
  }
}
```

## Test Users (`config.ts`)

```typescript
export const TEST_USERS = { admin: { username: 'admin', password: '...' } };
```

## Rules

- Headless by default; use `:headed` / `:ui` scripts for debugging
- `async/await` everywhere — no raw Promises or `.then()`
- Web-first assertions: `await expect(locator).toBeVisible()`
- Tags: `test('name @smoke @critical', ...)`
