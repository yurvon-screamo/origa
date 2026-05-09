import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { skipOnboarding } from "../helpers/navigation";
import { HomePage, WordsPage, GrammarPage, KanjiPage } from "../pages";

async function setupHomePage(page: Page): Promise<HomePage> {
  await skipOnboarding(page);
  
  const homePage = new HomePage(page);
  await expect(homePage.homePage).toBeVisible({ timeout: 15000 });
  return homePage;
}

testWithFreshUser.describe("Home Page", () => {
  testWithFreshUser("should display home page", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await expect(homePage.homePage).toBeVisible();
    await expect(homePage.homeContent).toBeVisible();
  });

  testWithFreshUser("should navigate to Words", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await homePage.navigateToWords();
    const wordsPage = new WordsPage(page);
    await wordsPage.expectWordsVisible();
  });

  testWithFreshUser("should navigate to Grammar", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await homePage.navigateToGrammar();
    const grammarPage = new GrammarPage(page);
    await grammarPage.expectGrammarVisible();
  });

  testWithFreshUser("should navigate to Kanji", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await homePage.navigateToKanji();
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.expectKanjiVisible();
  });

  testWithFreshUser(
    "should display lesson button in welcome card",
    async ({ page }) => {
      const homePage = await setupHomePage(page);

      await expect(homePage.welcomeCard).toBeVisible({ timeout: 10_000 });

      const lessonButton = page.locator("a").filter({ hasText: "УРОК" });
      await expect(lessonButton).toBeVisible();

      const jlptCard = page.locator(".card").filter({ hasText: /JLPT/ });
      const lessonBtnInCard = jlptCard.locator("a").filter({ hasText: "УРОК" });
      await expect(lessonBtnInCard).not.toBeVisible();
    },
  );

  testWithFreshUser(
    "should display welcome card with username",
    async ({ page }) => {
      const homePage = await setupHomePage(page);
      await expect(homePage.welcomeCard).toBeVisible({ timeout: 10_000 });
      const greetingText = homePage.welcomeCard.locator(".font-serif");
      await expect(greetingText).toBeVisible();
    },
  );

  testWithFreshUser(
    "should navigate to lesson from home page",
    async ({ page }) => {
      const homePage = await setupHomePage(page);
      await expect(homePage.lessonButton).toBeVisible({ timeout: 10_000 });
      await homePage.startLesson();
      await page.waitForURL(/\/lesson$/, { timeout: 15_000 });
    },
  );
});

testWithFreshUser.describe("Home Page - Stats & JLPT Progress", () => {
  async function setupHomeWithCards(page: Page): Promise<HomePage> {
    await skipOnboarding(page);

    const wordsPage = new WordsPage(page);
    await wordsPage.goto();
    await wordsPage.expectWordsVisible();
    await wordsPage.openAddModal();
    await wordsPage.enterText("私は本を読みます");
    await wordsPage.analyzeText();
    await wordsPage.selectFirstWord();
    await wordsPage.addSelectedWords();

    const homePage = new HomePage(page);
    await homePage.goto();
    await expect(homePage.homePage).toBeVisible({ timeout: 15_000 });
    return homePage;
  }

  testWithFreshUser("should display stats grid after adding cards", async ({ page }) => {
    test.setTimeout(60_000);
    const homePage = await setupHomeWithCards(page);

    await expect(homePage.statsGrid).toBeVisible({ timeout: 10_000 });
  });

  testWithFreshUser("should display stat cards in stats grid", async ({ page }) => {
    test.setTimeout(60_000);
    const homePage = await setupHomeWithCards(page);

    await expect(homePage.statLearned).toBeVisible({ timeout: 10_000 });
    await expect(homePage.statNew).toBeVisible();
  });

  testWithFreshUser("should display JLPT progress card", async ({ page }) => {
    test.setTimeout(60_000);
    const homePage = await setupHomeWithCards(page);

    await expect(homePage.jlptProgress).toBeVisible({ timeout: 10_000 });
  });

  testWithFreshUser("should toggle detailed stats view", async ({ page }) => {
    test.setTimeout(60_000);
    const homePage = await setupHomeWithCards(page);

    await expect(homePage.toggleDetails).toBeVisible({ timeout: 10_000 });
    await homePage.toggleDetailedStats();

    await expect(homePage.statInProgress).toBeVisible({ timeout: 5_000 });
    await expect(homePage.statHighDifficulty).toBeVisible();
    await expect(homePage.statPositive).toBeVisible();
    await expect(homePage.statNegative).toBeVisible();
    await expect(homePage.statTotalRatings).toBeVisible();
  });
});
