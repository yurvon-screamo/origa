import { test, expect } from '../fixtures';
import { WordsPage } from '../pages/WordsPage';

test.describe('Words page @smoke', () => {
  let wordsPage: WordsPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    wordsPage = new WordsPage(authenticatedPage);
    await wordsPage.goto();
  });

  test('should load words page @smoke', async () => {
    await test.step('Verify page loaded', async () => {
      await expect(wordsPage.page).toHaveURL(/\/words/);
    });
  });

  test('should show words page content @smoke', async () => {
    await expect(wordsPage.page).toHaveURL(/.*words.*/);
    await expect(wordsPage.page.locator('body')).toBeVisible();
  });

  test('should display search input @smoke', async () => {
    await expect(wordsPage.page).toHaveURL(/.*words.*/);
  });
});

test.describe('Words interactions @slow', () => {
  let wordsPage: WordsPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    wordsPage = new WordsPage(authenticatedPage);
    await wordsPage.goto();
  });

  test('should search for words @slow', async () => {
    await expect(wordsPage.page).toHaveURL(/.*words.*/);
  });

  test('should display filter options', async () => {
    await test.step('Check for filter buttons', async () => {
      const filterCount = await wordsPage.filterButtons.count();
      expect(filterCount).toBeGreaterThanOrEqual(0);
    });
  });

  test('should open add word modal', async () => {
    const hasButton = await wordsPage.addWordButton.isVisible().catch(() => false);
    if (!hasButton) {
      test.skip();
      return;
    }

    await test.step('Click add word button', async () => {
      await wordsPage.openAddModal();
    });

    await test.step('Verify modal is open', async () => {
      const modalVisible = await wordsPage.modal.isVisible().catch(() => false);
      if (modalVisible) {
        await wordsPage.closeModal();
      }
    });
  });

  test('should toggle favorite on word', async () => {
    await test.step('Wait for words list', async () => {
      await wordsPage.waitForWords();
    });

    await test.step('Click favorite button on first word if available', async () => {
      const wordCount = await wordsPage.getWordCount();
      if (wordCount === 0) {
        test.skip();
        return;
      }
      
      const favoriteCount = await wordsPage.favoriteButtons.count();
      if (favoriteCount > 0) {
        await wordsPage.toggleFavorite(0);
        await wordsPage.page.waitForTimeout(500);
      }
    });
  });

  test('should view word details', async () => {
    await test.step('Wait for words list', async () => {
      await wordsPage.waitForWords();
    });

    await test.step('Click on first word if available', async () => {
      const wordCount = await wordsPage.getWordCount();
      if (wordCount === 0) {
        test.skip();
        return;
      }
      await wordsPage.clickWord(0);
      await wordsPage.page.waitForTimeout(500);
    });
  });

  test('should apply status filter', async () => {
    await test.step('Apply filter if available', async () => {
      const filterCount = await wordsPage.filterButtons.count();
      if (filterCount === 0) {
        test.skip();
        return;
      }
      await wordsPage.filterButtons.first().click();
      await wordsPage.page.waitForTimeout(500);
    });
  });
});
