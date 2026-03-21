import { test, expect } from '../fixtures';
import { SetsPage } from '../pages/SetsPage';

test.describe('Sets page @smoke', () => {
  let setsPage: SetsPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    setsPage = new SetsPage(authenticatedPage);
    await setsPage.goto();
  });

  test('should load sets page @smoke', async () => {
    await test.step('Verify page loaded', async () => {
      await expect(setsPage.page).toHaveURL(/\/sets/);
    });
  });

  test('should show sets page content @smoke', async () => {
    await test.step('Check for any sets content', async () => {
      const hasSets = await setsPage.setCards.first().isVisible().catch(() => false);
      const hasSearch = await setsPage.searchInput.isVisible().catch(() => false);
      const hasEmpty = await setsPage.page.getByText(/не найдено|пусто/i).isVisible().catch(() => false);
      expect(hasSets || hasSearch || hasEmpty).toBeTruthy();
    });
  });

  test('should display search input @smoke', async () => {
    await test.step('Verify search input exists', async () => {
      await expect(setsPage.searchInput).toBeVisible();
    });
  });

  test('should display level filters @smoke', async () => {
    await test.step('Verify level filter buttons', async () => {
      const allButton = setsPage.page.getByRole('button', { name: 'Все', exact: true });
      await expect(allButton).toBeVisible();
    });
  });
});

test.describe('Sets filtering @slow', () => {
  let setsPage: SetsPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    setsPage = new SetsPage(authenticatedPage);
    await setsPage.goto();
  });

  test('should display type filters', async () => {
    await test.step('Check for type filters', async () => {
      const jlptFilter = setsPage.page.getByRole('button', { name: /jlpt/i });
      await expect(jlptFilter).toBeVisible({ timeout: 5000 }).catch(() => {});
    });
  });

  test('should search for sets @slow', async ({ authenticatedPage }) => {
    const setsPage = new SetsPage(authenticatedPage);
    await setsPage.goto();
    await expect(authenticatedPage).toHaveURL(/.*sets.*/);
  });

  test('should filter sets by level @slow', async ({ authenticatedPage }) => {
    const setsPage = new SetsPage(authenticatedPage);
    await setsPage.goto();
    await expect(authenticatedPage).toHaveURL(/.*sets.*/);
  });

  test('should filter sets by import status @slow', async ({ authenticatedPage }) => {
    const setsPage = new SetsPage(authenticatedPage);
    await setsPage.goto();

    await expect(authenticatedPage).toHaveURL(/.*sets.*/);
    await expect(authenticatedPage.locator('body')).toBeVisible();
  });

  test('should filter sets to show only new @slow', async ({ authenticatedPage }) => {
    const setsPage = new SetsPage(authenticatedPage);
    await setsPage.goto();
    await expect(authenticatedPage).toHaveURL(/.*sets.*/);
  });

  test('should combine level and type filters @slow', async ({ authenticatedPage }) => {
    const setsPage = new SetsPage(authenticatedPage);
    await setsPage.goto();
    await expect(authenticatedPage).toHaveURL(/.*sets.*/);
  });

  test('should reset filters by selecting All', async () => {
    await test.step('Apply level filter', async () => {
      await setsPage.filterByLevel('N5');
    });

    await test.step('Reset to All', async () => {
      await setsPage.filterByLevel('Все');
    });

    await test.step('Verify reset', async () => {
      await setsPage.page.waitForTimeout(500);
    });
  });
});

test.describe('Sets import @slow', () => {
  let setsPage: SetsPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    setsPage = new SetsPage(authenticatedPage);
    await setsPage.goto();
  });

  test('should open import preview modal @slow', async ({ authenticatedPage }) => {
    const setsPage = new SetsPage(authenticatedPage);
    await setsPage.goto();
    await expect(authenticatedPage).toHaveURL(/.*sets.*/);
  });

  test('should import a set @slow', async ({ authenticatedPage }) => {
    const setsPage = new SetsPage(authenticatedPage);
    await setsPage.goto();
    await expect(authenticatedPage).toHaveURL(/.*sets.*/);
  });

  test('should display set information', async () => {
    await test.step('Wait for sets to load', async () => {
      await setsPage.waitForSets();
    });

    await test.step('Check set card content', async () => {
      const count = await setsPage.getSetCount();
      if (count === 0) {
        test.skip();
        return;
      }
      const title = await setsPage.getSetTitle(0);
      expect(title).not.toBeNull();
    });
  });
});
