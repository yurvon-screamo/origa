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

  testWithFreshUser("should display JLPT progress card", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await expect(homePage.jlptProgress).toBeVisible({ timeout: 10_000 });
  });

  testWithFreshUser(
    "should expand JLPT details to show categories",
    async ({ page }) => {
      const homePage = await setupHomePage(page);
      await expect(homePage.jlptProgress).toBeVisible({ timeout: 10_000 });
      const toggle = page.getByTestId("home-jlpt-progress-toggle");
      await expect(toggle).toBeVisible();
      await toggle.click();
      const expandedCategories = page.getByTestId("home-jlpt-progress-expanded");
      await expect(expandedCategories).toBeVisible({ timeout: 5_000 });
    },
  );

  testWithFreshUser("should display today overview", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await expect(homePage.todayOverview).toBeVisible({ timeout: 10_000 });
  });

  testWithFreshUser("should display activity chart", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await expect(homePage.activityChart).toBeVisible({ timeout: 10_000 });
  });

  testWithFreshUser("should display recent study section", async ({ page }) => {
    const homePage = await setupHomePage(page);
    await expect(homePage.recentStudy).toBeVisible({ timeout: 10_000 });
  });

  testWithFreshUser(
    "should update JLPT progress after marking kanji as known",
    async ({ page }) => {
      test.setTimeout(120_000);
      await skipOnboarding(page);

      const homePage = new HomePage(page);
      await expect(homePage.jlptProgress).toBeVisible({ timeout: 15_000 });

      const jlptPct = page.getByTestId("home-jlpt-progress-pct");
      await expect(jlptPct).toHaveText("0%", { timeout: 10_000 });

      // Add N5 kanji and mark as known (N5 has ~80 kanji, so 5 gives ~6%)
      const kanjiPage = new KanjiPage(page);
      await kanjiPage.goto();
      await kanjiPage.expectKanjiVisible();
      await kanjiPage.openAddModal();
      await kanjiPage.selectLevel("N5");
      await kanjiPage.selectAllKanji();
      await kanjiPage.addSelectedKanji();

      const count = Math.min(5, await kanjiPage.getCardCount());
      for (let i = 0; i < count; i++) {
        await kanjiPage.markCardAsKnownByIndex(i);
        await page.waitForTimeout(300);
      }

      // Navigate to home and verify JLPT progress updated
      await homePage.goto();
      await expect(homePage.jlptProgress).toBeVisible({ timeout: 15_000 });
      await expect(jlptPct).not.toHaveText("0%", { timeout: 10_000 });
    },
  );
});

testWithFreshUser.describe("Home Page - Mobile Bottom Tab Navigation", () => {
    testWithFreshUser("should display bottom tab bar on mobile viewport", async ({ page }) => {
        await page.setViewportSize({ width: 375, height: 667 });
        await skipOnboarding(page);
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
        await page.getByTestId("home-page").waitFor({ state: "visible", timeout: 15_000 });

        const bottomTabBar = page.locator(".bottom-tab-bar");
        await expect(bottomTabBar).toBeVisible({ timeout: 10_000 });
    });

    testWithFreshUser("should navigate between pages via bottom tabs on mobile", async ({ page }) => {
        await page.setViewportSize({ width: 375, height: 667 });
        await skipOnboarding(page);
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
        await page.getByTestId("home-page").waitFor({ state: "visible", timeout: 15_000 });

        // Home → Words
        await page.getByTestId("bottom-tab-tab-words").click();
        await page.waitForURL(/\/words/, { timeout: 10_000 });

        // Words → Grammar
        await page.getByTestId("bottom-tab-tab-grammar").click();
        await page.waitForURL(/\/grammar/, { timeout: 10_000 });

        // Grammar → Profile
        await page.getByTestId("bottom-tab-tab-profile").click();
        await page.waitForURL(/\/profile/, { timeout: 10_000 });

        // Profile → Home
        await page.getByTestId("bottom-tab-tab-home").click();
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
    });
});
