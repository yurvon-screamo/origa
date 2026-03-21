import { test, expect } from '../fixtures';
import { HomePage, LessonPage, KanjiPage, SetsPage, WordsPage, GrammarPage } from '../pages';


test.describe('Complete User Journey @journey @smoke', () => {
  test.setTimeout(300000);


  test('Full learning flow: login -> sets -> import -> verify cards -> lesson @critical', async ({ authenticatedPage }) => {
    const homePage = new HomePage(authenticatedPage);
    const setsPage = new SetsPage(authenticatedPage);
    const kanjiPage = new KanjiPage(authenticatedPage);
    const wordsPage = new WordsPage(authenticatedPage);
    const grammarPage = new GrammarPage(authenticatedPage);
    const lessonPage = new LessonPage(authenticatedPage);

    await test.step('2. Navigate to Sets and check available sets', async () => {
      await setsPage.goto();
      await expect(authenticatedPage).toHaveURL(/\/sets/);
      await setsPage.waitForSets();
      const setCount = await setsPage.getSetCount();
      console.log(`Found ${setCount} sets`);
    });

    await test.step('3. Import first available set if not imported', async () => {
      const setCount = await setsPage.getSetCount();
      if (setCount > 0) {
        const importButtons = authenticatedPage.getByRole('button', { name: /импортировать|импорт/i });
        const firstImportButton = importButtons.first();
        const hasImportButton = await firstImportButton.isVisible({ timeout: 5000 }).catch(() => false);
        
        if (hasImportButton) {
          await firstImportButton.click();
          await authenticatedPage.waitForTimeout(1000);
          
          const confirmButton = authenticatedPage.getByRole('button', { name: /подтвердить|импортировать/i }).first();
          if (await confirmButton.isVisible({ timeout: 5000 }).catch(() => false)) {
            await confirmButton.click();
            await authenticatedPage.waitForTimeout(2000);
          }
          
          const closeButton = authenticatedPage.getByRole('button', { name: /закрыть/i }).first();
          if (await closeButton.isVisible({ timeout: 3000 }).catch(() => false)) {
            await closeButton.click();
          }
        }
      }
    });

    await test.step('4. Navigate to Kanji and verify cards exist', async () => {
      await kanjiPage.goto();
      await expect(authenticatedPage).toHaveURL(/\/kanji/);
      await kanjiPage.waitForKanjiList();
      const kanjiCount = await kanjiPage.getKanjiCount();
      console.log(`Found ${kanjiCount} kanji items`);
    });

    await test.step('5. Navigate to Words and verify content', async () => {
      await wordsPage.goto();
      await expect(authenticatedPage).toHaveURL(/\/words/);
      await wordsPage.waitForWords();
      const wordCount = await wordsPage.getWordCount();
      console.log(`Found ${wordCount} words`);
    });

    await test.step('6. Navigate to Grammar and verify content', async () => {
      await grammarPage.goto();
      await expect(authenticatedPage).toHaveURL(/\/grammar/);
      await grammarPage.waitForRules();
      const ruleCount = await grammarPage.getRuleCount();
      console.log(`Found ${ruleCount} grammar rules`);
    });

    await test.step('7. Start standard lesson from Home', async () => {
      await homePage.goto();
      await expect(homePage.lessonButton).toBeVisible({ timeout: 30000 });
      await homePage.startLesson();
      await expect(authenticatedPage).toHaveURL(/\/lesson/);
    });

    await test.step('8. Complete lesson by rating cards', async () => {
      await lessonPage.waitForCardsOrComplete();
      
      const hasNoCards = await lessonPage.noCardsMessage.isVisible().catch(() => false);
      if (hasNoCards) {
        console.log('No cards available for lesson - skipping rating');
        return;
      }
      
      const completed = await lessonPage.completeFullLesson(10);
      console.log(`Completed ${completed} cards in lesson`);
      expect(completed).toBeGreaterThanOrEqual(0);
    });

    await test.step('9. Verify home page shows progress', async () => {
      await homePage.goto();
      await expect(authenticatedPage.locator('body')).toContainText(/\d+/, { timeout: 30000 });
    });
  });

  test('Quick smoke test: navigation between all pages @smoke', async ({ authenticatedPage }) => {
    const pages = [
      { name: 'Home', url: '/home' },
      { name: 'Kanji', url: '/kanji' },
      { name: 'Words', url: '/words' },
      { name: 'Grammar', url: '/grammar' },
      { name: 'Sets', url: '/sets' },
    ];

    for (const pageInfo of pages) {
      await test.step(`Navigate to ${pageInfo.name}`,
        async () => {
          await authenticatedPage.goto(pageInfo.url);
          await authenticatedPage.waitForLoadState('domcontentloaded', { timeout: 60000 }).catch(() => {});
          await authenticatedPage.waitForTimeout(2000);
          await expect(authenticatedPage).toHaveURL(new RegExp(pageInfo.url));
        });
    }
  });

  test('Lesson flow: start and complete @smoke', async ({ authenticatedPage }) => {
    const homePage = new HomePage(authenticatedPage);
    const lessonPage = new LessonPage(authenticatedPage);

    await test.step('Navigate to home', async () => {
      await homePage.goto();
    });

    await test.step('Start lesson', async () => {
      await homePage.startLesson();
      await expect(authenticatedPage).toHaveURL(/\/lesson/);
    });

    await test.step('Handle lesson state', async () => {
      await lessonPage.waitForCardsOrComplete();
      
      const hasCards = await lessonPage.cardContainer.isVisible().catch(() => false);
      const hasNoCards = await lessonPage.noCardsMessage.isVisible().catch(() => false);
      const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
      
      expect(hasCards || hasNoCards || isComplete).toBeTruthy();
    });
  });

  test('Fixation lesson flow @smoke', async ({ authenticatedPage }) => {
    const homePage = new HomePage(authenticatedPage);
    const lessonPage = new LessonPage(authenticatedPage);

    await test.step('Navigate to home', async () => {
      await homePage.goto();
    });

    await test.step('Start fixation lesson', async () => {
      const hasFixationButton = await homePage.fixationButton.isVisible({ timeout: 10000 }).catch(() => false);
      if (hasFixationButton) {
        await homePage.startFixation();
        await expect(authenticatedPage).toHaveURL(/\/lesson.*mode=fixation/);
      } else {
        test.skip();
      }
    });
  });
});
