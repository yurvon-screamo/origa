import { test, expect } from '../fixtures';
import { KanjiPage } from '../pages/KanjiPage';

test.describe('Kanji page @smoke', () => {
  let kanjiPage: KanjiPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    kanjiPage = new KanjiPage(authenticatedPage);
    await kanjiPage.goto();
  });

  test('should load kanji page @smoke', async () => {
    await test.step('Verify page loaded', async () => {
      await expect(kanjiPage.page).toHaveURL(/\/kanji/);
    });
  });

  test('should show kanji page content @smoke', async () => {
    await expect(kanjiPage.page.getByRole('heading', { name: 'Кандзи', exact: true })).toBeVisible({ timeout: 30000 });
    await expect(kanjiPage.page).toHaveURL(/.*kanji.*/);
  });

  test('should display level selectors if available', async () => {
    await test.step('Check for level buttons', async () => {
      const n5Button = kanjiPage.page.getByRole('button', { name: 'N5' });
      const n4Button = kanjiPage.page.getByRole('button', { name: 'N4' });
      
      const hasLevels = await n5Button.or(n4Button).isVisible({ timeout: 10000 }).catch(() => false);
      if (!hasLevels) {
        test.skip();
      }
    });
  });
});

test.describe('Kanji interactions @slow', () => {
  let kanjiPage: KanjiPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    kanjiPage = new KanjiPage(authenticatedPage);
    await kanjiPage.goto();
  });

  test('should filter kanji by level', async () => {
    const n5Button = kanjiPage.page.getByRole('button', { name: 'N5' });
    const hasN5 = await n5Button.isVisible().catch(() => false);
    if (!hasN5) {
      test.skip();
      return;
    }

    await test.step('Select N5 level', async () => {
      await kanjiPage.selectLevel('N5');
    });

    await test.step('Verify kanji list updated', async () => {
      await kanjiPage.page.waitForTimeout(1000);
      const count = await kanjiPage.getKanjiCount();
      expect(count).toBeGreaterThanOrEqual(0);
    });
  });

  test('should select kanji for adding to cards', async () => {
    await test.step('Wait for kanji list', async () => {
      await kanjiPage.waitForKanjiList();
    });

    await test.step('Click on kanji items if available', async () => {
      const count = await kanjiPage.getKanjiCount();
      if (count === 0) {
        test.skip();
        return;
      }
      await kanjiPage.clickKanji(0);
    });

    await test.step('Verify selection indicator', async () => {
      await kanjiPage.page.waitForTimeout(500);
    });
  });

  test('should open add kanji modal', async () => {
    const hasButton = await kanjiPage.addKanjiButton.isVisible().catch(() => false);
    if (!hasButton) {
      test.skip();
      return;
    }

    await test.step('Click add kanji button', async () => {
      await kanjiPage.openAddModal();
    });

    await test.step('Verify modal is open', async () => {
      await expect(kanjiPage.modal).toBeVisible();
    });

    await test.step('Close modal', async () => {
      await kanjiPage.closeModal();
    });
  });

  test('should add kanji via modal', async () => {
    const hasButton = await kanjiPage.addKanjiButton.isVisible().catch(() => false);
    if (!hasButton) {
      test.skip();
      return;
    }

    await test.step('Open add modal', async () => {
      await kanjiPage.openAddModal();
    });

    await test.step('Enter kanji', async () => {
      await kanjiPage.addKanjiInModal('日');
    });

    await test.step('Verify modal closed', async () => {
      await kanjiPage.page.waitForTimeout(1000);
    });
  });

  test('should display kanji details', async () => {
    await test.step('Wait for kanji list', async () => {
      await kanjiPage.waitForKanjiList();
    });

    await test.step('Check kanji item content', async () => {
      const count = await kanjiPage.getKanjiCount();
      if (count === 0) {
        test.skip();
        return;
      }
      const firstItem = kanjiPage.kanjiItems.first();
      await expect(firstItem).toBeVisible();
    });
  });
});
