# E2E Testing - Origa

End-to-end testing suite for Origa application using Playwright.

## Overview

This directory contains end-to-end tests that verify the complete application flow, including:

- User authentication
- Navigation and routing
- Core user workflows
- Frontend-backend integration

## Installation

```bash
cd end2end
npm install
npx playwright install
```

## Environment Variables

Copy `.env.example` to `.env` and configure:

| Variable | Description | Default |
|----------|-------------|---------|
| `TRAILBASE_URL` | TrailBase API URL | `https://origa.uwuwu.net` |
| `ORIGA_ADMIN_EMAIL` | Admin email for user management | `admin@localhost` |
| `ORIGA_ADMIN_PASSWORD` | Admin password (required for user creation) | _(required)_ |
| `TEST_USER_EMAIL` | Test user email | `e2e-test@origa.local` |
| `TEST_USER_PASSWORD` | Test user password | `e2e-test-password-123` |

### Test User Configuration

Test users are configured with hardcoded defaults:

- **Email:** `e2e-test@origa.local`
- **Password:** `e2e-test-password-123`

Override via environment variables if needed.

## Running Tests Locally

```bash
# Run all tests (headless)
npm test

# Run with Playwright UI
npm run test:ui

# Run in headed mode (visible browser)
npm run test:headed

# Debug mode (step through tests)
npm run test:debug

# View test report
npm run report
```

### For CSR-Only Projects

This is a CSR (Client-Side Rendering) Leptos project. Run tests manually:

```bash
# Terminal 1: Start the dev server
cd origa_ui && trunk serve

# Terminal 2: Run Playwright tests
cd end2end && npm test
```

The `webServer` configuration in `playwright.config.ts` will automatically start `trunk serve` if not running.

## Project Structure

```
end2end/
├── global-setup.ts      # Authentication and test setup
├── playwright.config.ts # Playwright configuration
├── package.json         # NPM dependencies and scripts
├── tsconfig.json        # TypeScript configuration
├── .env.example         # Environment variables template
├── .gitignore           # Git ignore rules
└── tests/               # Test files
    ├── auth.spec.ts     # Authentication tests
    └── tauri/           # Desktop-specific tests
        ├── desktop.spec.ts
        └── setup.ts
```

### Key Files

- **`playwright.config.ts`** - Playwright configuration including:
  - Test directory: `./tests`
  - Base URL: `http://localhost:1420`
  - Browser projects: Chromium, Firefox
  - Auto web server startup via `trunk serve`

- **`global-setup.ts`** - Runs before all tests:
  - Validates environment variables
  - Configures test user credentials
  - Logs setup configuration

## CI/CD Integration

The test suite is configured for CI environments:

- **CI detection:** Uses `process.env.CI` to detect CI mode
- **Retries:** 2 retries in CI, 0 locally
- **Workers:** 1 worker in CI (for stability)
- **Forbid only:** `test.only` is forbidden in CI
- **Artifacts:**
  - Traces on first retry
  - Screenshots on failure
  - Videos retained on failure

### GitHub Actions Example

```yaml
- name: Install Playwright
  run: cd end2end && npm ci && npx playwright install --with-deps

- name: Run E2E tests
  run: cd end2end && npm test
  env:
    CI: true
    ORIGA_ADMIN_PASSWORD: ${{ secrets.ORIGA_ADMIN_PASSWORD }}
```

## TrailBase API Requirements

For authentication fixtures to work correctly, the TrailBase API must:

1. Be accessible at `TRAILBASE_URL` (default: `https://origa.uwuwu.net`)
2. Have admin endpoints available for user management
3. Accept admin credentials (`ORIGA_ADMIN_EMAIL`/`ORIGA_ADMIN_PASSWORD`)
4. Support test user creation with the configured email

## Adding New Tests

1. Create a new test file in `tests/` directory:

```typescript
// tests/example.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Example feature', () => {
  test('should work correctly', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('h1')).toBeVisible();
  });
});
```

1. Use test fixtures for authenticated pages:

```typescript
import { test, expect } from '@playwright/test';
import { testUser } from '../playwright.config';

test.use({ storageState: { cookies: [], origins: [] } }); // For unauthenticated tests

test('login flow', async ({ page }) => {
  await page.goto('/login');
  await page.fill('input[name="email"]', testUser.email);
  await page.fill('input[name="password"]', testUser.password);
  await page.click('button[type="submit"]');
});
```

1. Run tests:

```bash
npm test                    # All tests
npm test example.spec.ts    # Specific file
npx playwright test -g "test name"  # By test name
```

## Debugging

### Playwright Inspector

```bash
npm run test:debug
```

Opens Playwright Inspector for step-by-step debugging.

### Trace Viewer

After test failure, view trace:

```bash
npx playwright show-trace trace.zip
```

### Headed Mode

Run tests in visible browser to see actions:

```bash
npm run test:headed
```

## Browser Projects

Tests run on three browsers by default:

- **Chromium** - Desktop Chrome
- **Firefox** - Desktop Firefox

Run specific browser:

```bash
npx playwright test --project=chromium
npx playwright test --project=firefox
```

## Troubleshooting

### Tests timeout

Increase timeout in `playwright.config.ts` or per test:

```typescript
test('slow test', async ({ page }) => {
  test.setTimeout(60000);
  // ...
});
```

### Web server not starting

Ensure `trunk` is installed and port 1420 is available:

```bash
cargo install trunk
```

### Authentication failures

Verify:

1. `TRAILBASE_URL` is accessible
2. `ORIGA_ADMIN_PASSWORD` is set correctly
3. Test user exists or can be created by admin

### CI-specific issues

- Check artifacts (traces, screenshots, videos) in test report
- Ensure `playwright install --with-deps` runs before tests
- Verify environment variables are set in CI configuration

### Report viewing issues

If you see an error like "Unsafe attempt to load URL <http://localhost:10485/> from frame with URL chrome-error://chromewebdata/. Domains, protocols and ports must match", this is a browser security issue with Playwright HTML reports loading in iframes.

**Solutions:**

1. **Use the updated report command** (recommended):

   ```bash
   npm run report
   ```

   This now uses `--host 0.0.0.0 --port 9323` to avoid iframe security issues.

2. **Open report as file directly**:

   ```bash
   npm run report:file
   ```

   Opens the HTML report directly in your default browser.

3. **Manual fix**:
   If issues persist, try clearing browser cache or using a different browser.
