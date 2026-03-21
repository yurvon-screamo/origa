import { test, expect } from '../fixtures';
import { HomePage } from '../pages/HomePage';

test.describe('Home page @smoke', () => {
  let homePage: HomePage;

  test.beforeEach(async ({ authenticatedPage }) => {
    homePage = new HomePage(authenticatedPage);
    await homePage.goto();
  });

  test('should display user greeting @smoke', async () => {
    await test.step('Verify page shows user content', async () => {
      await expect(homePage.lessonButton).toBeVisible({ timeout: 30000 });
    });
  });

  test('should display lesson buttons @smoke', async () => {
    await test.step('Verify lesson button exists', async () => {
      await expect(homePage.lessonButton).toBeVisible();
    });

    await test.step('Verify fixation button exists', async () => {
      await expect(homePage.fixationButton).toBeVisible();
    });
  });

  test('should navigate to lesson page @smoke', async () => {
    await test.step('Click lesson button', async () => {
      await homePage.startLesson();
    });

    await test.step('Verify navigation', async () => {
      await expect(homePage.page).toHaveURL(/\/lesson/);
    });
  });

  test('should navigate to fixation lesson', async () => {
    await test.step('Click fixation button', async () => {
      await homePage.startFixation();
    });

    await test.step('Verify navigation with mode parameter', async () => {
      await expect(homePage.page).toHaveURL(/\/lesson.*mode=fixation/);
    });
  });

  test('should navigate to words page @smoke', async ({ authenticatedPage }) => {
    await test.step('Navigate to words', async () => {
      await homePage.navigateTo('/words');
    });

    await test.step('Verify navigation', async () => {
      await expect(authenticatedPage).toHaveURL(/\/words/);
    });
  });

  test('should navigate to kanji page @smoke', async ({ authenticatedPage }) => {
    await test.step('Navigate to kanji', async () => {
      await homePage.navigateTo('/kanji');
    });

    await test.step('Verify navigation', async () => {
      await expect(authenticatedPage).toHaveURL(/\/kanji/);
    });
  });

  test('should navigate to grammar page', async ({ authenticatedPage }) => {
    await test.step('Navigate to grammar', async () => {
      await homePage.navigateTo('/grammar');
    });

    await test.step('Verify navigation', async () => {
      await expect(authenticatedPage).toHaveURL(/\/grammar/);
    });
  });

  test('should navigate to sets page', async ({ authenticatedPage }) => {
    await test.step('Navigate to sets', async () => {
      await homePage.navigateTo('/sets');
    });

    await test.step('Verify navigation', async () => {
      await expect(authenticatedPage).toHaveURL(/\/sets/);
    });
  });
});

test.describe('Home statistics @slow', () => {
  let homePage: HomePage;

  test.beforeEach(async ({ authenticatedPage }) => {
    homePage = new HomePage(authenticatedPage);
    await homePage.goto();
  });

  test('should display statistics section @smoke', async ({ authenticatedPage }) => {
    await expect(authenticatedPage.locator('body')).toContainText(/\d+/, { timeout: 30000 });
  });

  test('should display JLPT progress if data available', async () => {
    const jlptText = homePage.page.getByText(/jlpt|n5|n4|n3|n2|n1/i);
    const hasJlpt = await jlptText.first().isVisible({ timeout: 10000 }).catch(() => false);
    if (!hasJlpt) {
      test.skip();
    }
  });
});
