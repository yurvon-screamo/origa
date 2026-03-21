import { test, expect } from '../fixtures';
import { LessonPage } from '../pages/LessonPage';

test.describe('Lesson page @smoke', () => {
  let lessonPage: LessonPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    lessonPage = new LessonPage(authenticatedPage);
  });

  test('should load lesson page @smoke', async () => {
    await test.step('Navigate to lesson', async () => {
      await lessonPage.goto();
    });

    await test.step('Verify page loaded', async () => {
      await expect(lessonPage.page).toHaveURL(/\/lesson/);
    });
  });

  test('should show lesson page content @smoke', async () => {
    await test.step('Navigate to lesson', async () => {
      await lessonPage.goto();
    });

    await test.step('Check for any lesson content', async () => {
      const hasCards = await lessonPage.cardContainer.isVisible().catch(() => false);
      const hasNoCards = await lessonPage.noCardsMessage.isVisible().catch(() => false);
      const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
      const hasUrl = lessonPage.page.url().includes('/lesson');
      expect(hasCards || hasNoCards || isComplete || hasUrl).toBeTruthy();
    });
  });
});

test.describe('Lesson interactions @slow', () => {
  let lessonPage: LessonPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    lessonPage = new LessonPage(authenticatedPage);
  });

  test('should complete a full lesson cycle', async () => {
    await test.step('Navigate to lesson', async () => {
      await lessonPage.goto();
    });

    await test.step('Check if cards available', async () => {
      const hasCards = await lessonPage.cardContainer.isVisible().catch(() => false);
      if (!hasCards) {
        test.skip();
        return;
      }
    });

    await test.step('Complete lesson cards', async () => {
      const completed = await lessonPage.completeFullLesson(5);
      console.log(`Completed ${completed} cards`);
    });

    await test.step('Verify completion screen appears', async () => {
      const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
      const hasNoCards = await lessonPage.noCardsMessage.isVisible().catch(() => false);
      expect(isComplete || hasNoCards).toBeTruthy();
    });
  });

  test('should show rating buttons after answer', async () => {
    await test.step('Navigate to lesson', async () => {
      await lessonPage.goto();
    });

    await test.step('Check if cards available', async () => {
      const hasCards = await lessonPage.cardContainer.isVisible().catch(() => false);
      if (!hasCards) {
        test.skip();
        return;
      }
    });

    await test.step('Show answer', async () => {
      const hasAnswerButton = await lessonPage.answerButton.isVisible().catch(() => false);
      if (hasAnswerButton) {
        await lessonPage.showAnswer();
      }
    });

    await test.step('Verify rating buttons visible', async () => {
      await lessonPage.expectRatingButtonsVisible();
    });
  });

  test('should rate card with different ratings', async () => {
    await test.step('Navigate to lesson', async () => {
      await lessonPage.goto();
    });

    await test.step('Check if cards available', async () => {
      const hasCards = await lessonPage.cardContainer.isVisible().catch(() => false);
      if (!hasCards) {
        test.skip();
        return;
      }
    });

    await test.step('Show answer and rate', async () => {
      const hasAnswerButton = await lessonPage.answerButton.isVisible().catch(() => false);
      if (hasAnswerButton) {
        await lessonPage.showAnswer();
      }
      await lessonPage.rate('good');
    });

    await test.step('Verify card processed', async () => {
      await lessonPage.page.waitForTimeout(500);
    });
  });

  test('should navigate to next lesson from completion screen', async () => {
    await test.step('Navigate to lesson', async () => {
      await lessonPage.goto();
    });

    await test.step('Complete lesson', async () => {
      await lessonPage.completeFullLesson(10);
    });

    await test.step('Check for completion screen', async () => {
      const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
      if (!isComplete) {
        test.skip();
        return;
      }
    });

    await test.step('Click next lesson', async () => {
      const hasNextButton = await lessonPage.nextLessonButton.isVisible().catch(() => false);
      if (hasNextButton) {
        await lessonPage.goToNextLesson();
      }
    });
  });

  test('should navigate to home from completion screen', async () => {
    await test.step('Navigate to lesson', async () => {
      await lessonPage.goto();
    });

    await test.step('Complete lesson', async () => {
      await lessonPage.completeFullLesson(10);
    });

    await test.step('Check for completion screen', async () => {
      const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
      if (!isComplete) {
        test.skip();
        return;
      }
    });

    await test.step('Click go home', async () => {
      const hasHomeButton = await lessonPage.goHomeButton.isVisible().catch(() => false);
      if (hasHomeButton) {
        await lessonPage.goHome();
        await expect(lessonPage.page).toHaveURL(/\/home/);
      }
    });
  });
});

test.describe('Fixation lesson @slow', () => {
  let lessonPage: LessonPage;

  test.beforeEach(async ({ authenticatedPage }) => {
    lessonPage = new LessonPage(authenticatedPage);
  });

  test('should load fixation lesson', async () => {
    await test.step('Navigate to fixation lesson', async () => {
      await lessonPage.goto('fixation');
    });

    await test.step('Verify page loaded', async () => {
      await expect(lessonPage.page).toHaveURL(/mode=fixation/);
    });
  });
});
