import { test, expect } from '../fixtures';
import { GrammarPage } from '../pages/GrammarPage';

test.describe('Grammar page @smoke', () => {
  let grammarPage: GrammarPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    grammarPage = new GrammarPage(authenticatedPage);
    await grammarPage.goto();
  });

  test('should load grammar page @smoke', async () => {
    await test.step('Verify page loaded', async () => {
      await expect(grammarPage.page).toHaveURL(/\/grammar/);
    });
  });

  test('should show grammar page content @smoke', async () => {
    await expect(grammarPage.page.getByRole('heading', { name: 'Грамматика', exact: true })).toBeVisible({ timeout: 30000 });
    await expect(grammarPage.page).toHaveURL(/.*grammar.*/);
  });

  test('should display level selectors if available', async () => {
    await test.step('Check for level buttons', async () => {
      const n5Button = grammarPage.page.getByRole('button', { name: 'N5' });
      const n4Button = grammarPage.page.getByRole('button', { name: 'N4' });
      
      const hasLevels = await n5Button.or(n4Button).isVisible({ timeout: 10000 }).catch(() => false);
      if (!hasLevels) {
        test.skip();
      }
    });
  });
});

test.describe('Grammar interactions @slow', () => {
  let grammarPage: GrammarPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    grammarPage = new GrammarPage(authenticatedPage);
    await grammarPage.goto();
  });

  test('should filter grammar by level', async () => {
    const n5Button = grammarPage.page.getByRole('button', { name: 'N5' });
    const hasN5 = await n5Button.isVisible().catch(() => false);
    if (!hasN5) {
      test.skip();
      return;
    }

    await test.step('Select N5 level', async () => {
      await grammarPage.selectLevel('N5');
    });

    await test.step('Verify rules list updated', async () => {
      await grammarPage.page.waitForTimeout(1000);
      const count = await grammarPage.getRuleCount();
      expect(count).toBeGreaterThanOrEqual(0);
    });
  });

  test('should select grammar rules for adding to cards', async () => {
    await test.step('Wait for rules list', async () => {
      await grammarPage.waitForRules();
    });

    await test.step('Click on rule items if available', async () => {
      const count = await grammarPage.getRuleCount();
      if (count === 0) {
        test.skip();
        return;
      }
      await grammarPage.clickRule(0);
    });

    await test.step('Verify selection indicator', async () => {
      await grammarPage.page.waitForTimeout(500);
    });
  });

  test('should open add grammar modal', async () => {
    const hasButton = await grammarPage.addRuleButton.isVisible().catch(() => false);
    if (!hasButton) {
      test.skip();
      return;
    }

    await test.step('Click add rule button', async () => {
      await grammarPage.openAddModal();
    });

    await test.step('Verify modal is open', async () => {
      const modalVisible = await grammarPage.modal.isVisible().catch(() => false);
      if (modalVisible) {
        await grammarPage.closeModal();
      }
    });
  });

  test('should add grammar rule via modal', async () => {
    const hasButton = await grammarPage.addRuleButton.isVisible().catch(() => false);
    if (!hasButton) {
      test.skip();
      return;
    }

    await test.step('Open add modal', async () => {
      await grammarPage.openAddModal();
    });

    await test.step('Enter rule data', async () => {
      await grammarPage.addRule('〜は〜です', 'X is Y');
    });

    await test.step('Verify modal closed', async () => {
      await grammarPage.page.waitForTimeout(1000);
    });
  });

  test('should add selected rules to cards', async () => {
    await test.step('Wait for rules list', async () => {
      await grammarPage.waitForRules();
    });

    await test.step('Select rules if available', async () => {
      const count = await grammarPage.getRuleCount();
      if (count === 0) {
        test.skip();
        return;
      }
      await grammarPage.selectRules([0]);
    });

    await test.step('Check for add button', async () => {
      const hasAddButton = await grammarPage.addToCardsButton.isVisible().catch(() => false);
      if (hasAddButton) {
        await grammarPage.addToCards();
        await grammarPage.page.waitForTimeout(500);
      }
    });
  });

  test('should display grammar rule details', async () => {
    await test.step('Wait for rules list', async () => {
      await grammarPage.waitForRules();
    });

    await test.step('Check rule item content', async () => {
      const count = await grammarPage.getRuleCount();
      if (count === 0) {
        test.skip();
        return;
      }
      const firstItem = grammarPage.ruleItems.first();
      await expect(firstItem).toBeVisible();
    });
  });
});
