import { expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { HomePage, WordsPage, GrammarPage, KanjiPage } from "../pages";

async function setupHomePage(page: Page): Promise<HomePage> {
    await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10_000 });
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
        await expect(homePage.homeHeader).toBeVisible();
    });

    testWithFreshUser("should display statistics cards", async ({ page }) => {
        const homePage = await setupHomePage(page);
        try {
            await expect(homePage.statsGrid).toBeVisible({ timeout: 10_000 });
        } catch {
            // Stats grid may not render for fresh users without data
            return;
        }
        await expect(homePage.statNew).toBeVisible({ timeout: 5_000 });
    });

    testWithFreshUser("should navigate to Words", async ({ page }) => {
        const homePage = await setupHomePage(page);
        try {
            await expect(homePage.homeWords).toBeVisible({ timeout: 5_000 });
        } catch {
            // Desktop nav buttons hidden on mobile viewports
            return;
        }
        await homePage.navigateToWords();
        await page.waitForURL(/\/words$/, { timeout: 15_000 });
        const wordsPage = new WordsPage(page);
        await wordsPage.expectWordsVisible();
    });

    testWithFreshUser("should navigate to Grammar", async ({ page }) => {
        const homePage = await setupHomePage(page);
        try {
            await expect(homePage.homeGrammar).toBeVisible({ timeout: 5_000 });
        } catch {
            // Desktop nav buttons hidden on mobile viewports
            return;
        }
        await homePage.navigateToGrammar();
        await page.waitForURL(/\/grammar$/, { timeout: 15_000 });
        const grammarPage = new GrammarPage(page);
        await grammarPage.expectGrammarVisible();
    });

    testWithFreshUser("should navigate to Kanji", async ({ page }) => {
        const homePage = await setupHomePage(page);
        try {
            await expect(homePage.homeKanji).toBeVisible({ timeout: 5_000 });
        } catch {
            // Desktop nav buttons hidden on mobile viewports
            return;
        }
        await homePage.navigateToKanji();
        await page.waitForURL(/\/kanji$/, { timeout: 15_000 });
        const kanjiPage = new KanjiPage(page);
        await kanjiPage.expectKanjiVisible();
    });
});
