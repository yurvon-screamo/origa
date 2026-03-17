import { test as base, Page, BrowserContext } from '@playwright/test';
import path from 'node:path';

const authFile = path.join(__dirname, '..', 'playwright', '.auth', 'user.json');

type AuthFixtures = {
  authenticatedPage: Page;
  authenticatedContext: BrowserContext;
};

export const test = base.extend<AuthFixtures>({
  authenticatedContext: async ({ browser }, use) => {
    const context = await browser.newContext({
      storageState: authFile,
    });
    await use(context);
    await context.close();
  },
  authenticatedPage: async ({ authenticatedContext }, use) => {
    const page = await authenticatedContext.newPage();
    page.setDefaultTimeout(60000);
    await use(page);
  },
});

export { expect } from '@playwright/test';
export { TEST_KANJI, TEST_VOCABULARY, TEST_GRAMMAR_RULES, TEST_USER } from './test-data';
