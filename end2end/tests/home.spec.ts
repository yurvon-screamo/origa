import { expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { HomePage, WordsPage, GrammarPage, KanjiPage } from "../pages";

async function setupHomePage(page: Page): Promise<HomePage> {
  await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({
    timeout: 10_000,
  });
  await page.getByTestId("onboarding-skip").click();
  await page.waitForURL(/\/home$/, { timeout: 10_000 });
  
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
