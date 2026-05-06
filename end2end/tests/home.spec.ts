import { expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { HomePage, WordsPage, GrammarPage, KanjiPage } from "../pages";

async function setupHomePage(page: Page): Promise<HomePage> {
  await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({
    timeout: 10_000,
  });
  await page.getByTestId("onboarding-skip").click();
  await page.waitForURL(/\/home$/, { timeout: 10_000 });

  await page.waitForLoadState("domcontentloaded");
  await page.evaluate(() => {
    const overlays = document.querySelectorAll(".loading-overlay");
    overlays.forEach((el) => {
      (el as HTMLElement).style.display = "none";
    });
  });

  const homePage = new HomePage(page);
  await homePage.expectHomeVisible();
  return homePage;
}

testWithFreshUser.describe("Home Page", () => {
  testWithFreshUser("should display home page", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await expect(homePage.homePage).toBeVisible();
    await expect(homePage.homeContent).toBeVisible();
    await expect(homePage.sidebar).toBeVisible();
  });

  testWithFreshUser("should navigate to Words", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await expect(homePage.sidebarWords).toBeVisible({ timeout: 5_000 });
    await homePage.navigateToWords();
    await page.waitForURL(/\/words$/, { timeout: 15_000 });
    const wordsPage = new WordsPage(page);
    await wordsPage.expectWordsVisible();
  });

  testWithFreshUser("should navigate to Grammar", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await expect(homePage.sidebarGrammar).toBeVisible({ timeout: 5_000 });
    await homePage.navigateToGrammar();
    await page.waitForURL(/\/grammar$/, { timeout: 15_000 });
    const grammarPage = new GrammarPage(page);
    await grammarPage.expectGrammarVisible();
  });

  testWithFreshUser("should navigate to Kanji", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await expect(homePage.sidebarKanji).toBeVisible({ timeout: 5_000 });
    await homePage.navigateToKanji();
    await page.waitForURL(/\/kanji$/, { timeout: 15_000 });
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.expectKanjiVisible();
  });

  testWithFreshUser(
    "should display lesson button in welcome card",
    async ({ page }) => {
      const homePage = await setupHomePage(page);

      await expect(homePage.welcomeCard).toBeVisible({ timeout: 10_000 });

      const lessonButton = homePage.welcomeCard.locator(
        "[data-testid='lesson-buttons-lesson']",
      );
      await expect(lessonButton).toBeVisible();

      const jlptCard = page.locator(".card").filter({ hasText: /JLPT/ });
      const lessonBtnInCard = jlptCard.locator(
        "[data-testid='lesson-buttons-lesson']",
      );
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
