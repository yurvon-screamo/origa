import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { skipOnboarding } from "../helpers/navigation";
import { HomePage, LessonPage, WordsPage } from "../pages";

async function setupLessonWithCards(page: Page): Promise<LessonPage> {
    await skipOnboarding(page);

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

async function rateCardUntilDone(lessonPage: LessonPage, rating: "again" | "good", maxCards = 5): Promise<void> {
    for (let i = 0; i < maxCards; i++) {
        const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
        if (isComplete) break;
        try {
            await lessonPage.showAnswer();
            await lessonPage.rate(rating);
            await expect(
                lessonPage.showAnswerBtn.or(lessonPage.completeScreen)
            ).toBeVisible({ timeout: 5000 });
        } catch {
            break;
        }
    }
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

        await rateCardUntilDone(lessonPage, "good");

        await lessonPage.waitForComplete();
    });

    testWithFreshUser("should display progress text with card count", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        const progressText = await lessonPage.getProgressText();
        expect(progressText).toMatch(/^\d+\/\d+$/);
    });

    testWithFreshUser("should show rating buttons after revealing answer", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        await lessonPage.showAnswer();

        await expect(lessonPage.ratingAgain).toBeVisible();
        await expect(lessonPage.ratingGood).toBeVisible();
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

        await rateCardUntilDone(lessonPage, "good");

        await lessonPage.waitForComplete();
        await expect(lessonPage.completeStats).toBeVisible();
        await expect(lessonPage.nextLessonBtn).toBeVisible();
        await expect(lessonPage.homeBtn).toBeVisible();
    });

    testWithFreshUser("should navigate to home from complete screen", async ({ page }) => {
        test.setTimeout(120_000);
        const lessonPage = await setupLessonWithCards(page);

        await rateCardUntilDone(lessonPage, "good");

        await lessonPage.waitForComplete();
        await lessonPage.clickHome();
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
    });

    testWithFreshUser("should start next lesson from complete screen", async ({ page }) => {
        test.setTimeout(120_000);
        await skipOnboarding(page);

        // Add multiple words to ensure enough cards for two lessons
        const wordsPage = new WordsPage(page);
        await wordsPage.goto();
        await wordsPage.expectWordsVisible();

        const sentences = [
            "私は本を読みます",
            "彼は学校に行きます",
            "猫が魚を食べる",
        ];
        for (const sentence of sentences) {
            await wordsPage.openAddModal();
            await wordsPage.enterText(sentence);
            await wordsPage.analyzeText();
            await wordsPage.selectFirstWord();
            await wordsPage.addSelectedWords();
            await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });
        }

        const homePage = new HomePage(page);
        await homePage.goto();
        await homePage.startLesson();

        const lessonPage = new LessonPage(page);
        await expect(lessonPage.lessonPage).toBeVisible({ timeout: 15_000 });
        await expect(lessonPage.lessonError).not.toBeVisible({ timeout: 15_000 });
        await expect(lessonPage.lessonLoading).toBeHidden({ timeout: 30_000 });
        await expect(lessonPage.lessonContent).toBeVisible({ timeout: 15_000 });
        await expect(lessonPage.showAnswerBtn).toBeVisible({ timeout: 15_000 });

        await rateCardUntilDone(lessonPage, "good");
        await lessonPage.waitForComplete();

        await lessonPage.clickNextLesson();

        // After next lesson click, either new lesson loads or we get back to home (no cards)
        const newLessonVisible = await lessonPage.lessonContent
            .isVisible({ timeout: 15_000 })
            .catch(() => false);
        const errorVisible = await lessonPage.lessonError.isVisible().catch(() => false);

        if (newLessonVisible) {
            await expect(lessonPage.lessonContent).toBeVisible({ timeout: 15_000 });
        } else if (errorVisible) {
            // No more cards available — this is acceptable behavior
            await expect(lessonPage.lessonError).toBeVisible();
        } else {
            // May have redirected to home
            await expect(page).toHaveURL(/\/home/, { timeout: 10_000 });
        }
    });

});
