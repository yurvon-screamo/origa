import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { HomePage, LessonPage, WordsPage } from "../pages";

async function setupLessonWithCards(page: Page): Promise<LessonPage> {
    await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10000 });
    await page.getByTestId("onboarding-skip").click();
    await page.waitForURL(/\/home$/, { timeout: 10000 });

    const wordsPage = new WordsPage(page);
    await wordsPage.goto();
    await wordsPage.expectWordsVisible();
    await wordsPage.openAddModal();
    await wordsPage.enterText("私は本を読みます");
    await wordsPage.analyzeText();
    await wordsPage.selectFirstWord();
    await wordsPage.addSelectedWords();
    await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });

    const homePage = new HomePage(page);
    await homePage.goto();
    await homePage.startLesson();

    const lessonPage = new LessonPage(page);

    await expect(lessonPage.lessonPage).toBeVisible({ timeout: 15_000 });
    await expect(lessonPage.lessonError).not.toBeVisible({ timeout: 15_000 });
    await expect(lessonPage.lessonLoading).toBeHidden({ timeout: 30_000 });
    await expect(lessonPage.lessonContent).toBeVisible({ timeout: 15_000 });
    await expect(lessonPage.showAnswerBtn).toBeVisible({ timeout: 15_000 });

    return lessonPage;
}

testWithFreshUser.describe("Lesson Page", () => {
    testWithFreshUser("should display lesson page with card", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        await lessonPage.expectLessonVisible();
    });

    testWithFreshUser("should show answer and rate card", async ({ page }) => {
        test.setTimeout(120_000);
        const lessonPage = await setupLessonWithCards(page);
        await lessonPage.expectLessonVisible();
        await lessonPage.showAnswer();
        await lessonPage.rate("good");
    });

    testWithFreshUser("should navigate back from lesson", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        await lessonPage.clickBack();
        await page.waitForURL(/\/home$/, { timeout: 10000 });
    });

    testWithFreshUser("should complete lesson after rating all cards", async ({ page }) => {
        test.setTimeout(120_000);
        const lessonPage = await setupLessonWithCards(page);

        const alreadyComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
        if (alreadyComplete) {
            test.skip();
            return;
        }

        for (let i = 0; i < 5; i++) {
            const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
            if (isComplete) break;
            try {
                await lessonPage.showAnswer();
                await page.waitForTimeout(300);
                await lessonPage.rate("easy");
                await page.waitForTimeout(500);
            } catch {
                break;
            }
        }

        await lessonPage.waitForComplete();
    });

    testWithFreshUser("should display progress text with card count", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        const progressText = await lessonPage.getProgressText();
        expect(progressText).toMatch(/^\d+\/\d+$/);
    });

    testWithFreshUser("should show all rating buttons after revealing answer", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        await lessonPage.showAnswer();

        await expect(lessonPage.ratingAgain).toBeVisible();
        await expect(lessonPage.ratingHard).toBeVisible();
        await expect(lessonPage.ratingGood).toBeVisible();
        await expect(lessonPage.ratingEasy).toBeVisible();
    });

    testWithFreshUser("should toggle mute button and change state", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        await expect(lessonPage.muteButton).toBeVisible();
        await expect(lessonPage.muteButton).toHaveAttribute("data-muted", "false");

        await lessonPage.toggleMute();
        await expect(lessonPage.muteButton).toHaveAttribute("data-muted", "true");

        await lessonPage.toggleMute();
        await expect(lessonPage.muteButton).toHaveAttribute("data-muted", "false");
    });

    testWithFreshUser("should show complete screen with stats and navigation buttons", async ({ page }) => {
        test.setTimeout(120_000);
        const lessonPage = await setupLessonWithCards(page);

        const alreadyComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
        if (alreadyComplete) {
            test.skip();
            return;
        }

        for (let i = 0; i < 5; i++) {
            const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
            if (isComplete) break;
            try {
                await lessonPage.showAnswer();
                await page.waitForTimeout(300);
                await lessonPage.rate("easy");
                await page.waitForTimeout(500);
            } catch {
                break;
            }
        }

        await lessonPage.waitForComplete();
        await expect(lessonPage.completeStats).toBeVisible();
        await expect(lessonPage.nextLessonBtn).toBeVisible();
        await expect(lessonPage.homeBtn).toBeVisible();
    });

    testWithFreshUser("should navigate to home from complete screen", async ({ page }) => {
        test.setTimeout(120_000);
        const lessonPage = await setupLessonWithCards(page);

        const alreadyComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
        if (alreadyComplete) {
            test.skip();
            return;
        }

        for (let i = 0; i < 5; i++) {
            const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
            if (isComplete) break;
            try {
                await lessonPage.showAnswer();
                await page.waitForTimeout(300);
                await lessonPage.rate("easy");
                await page.waitForTimeout(500);
            } catch {
                break;
            }
        }

        await lessonPage.waitForComplete();
        await lessonPage.clickHome();
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
    });

});
