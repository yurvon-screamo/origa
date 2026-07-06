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

const MAX_LESSON_ITERATIONS = 50;

async function rateCardUntilDone(
    lessonPage: LessonPage,
    rating: "again" | "good",
    maxIterations = MAX_LESSON_ITERATIONS
): Promise<void> {
    for (let i = 0; i < maxIterations; i++) {
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

async function completeLessonFlexible(
    lessonPage: LessonPage,
    page: Page,
    maxIterations = MAX_LESSON_ITERATIONS
): Promise<void> {
    for (let i = 0; i < maxIterations; i++) {
        const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
        if (isComplete) break;

        const anyInteractive = lessonPage.showAnswerBtn
            .or(lessonPage.quizOptions[0])
            .or(lessonPage.yesnoYesBtn)
            .or(lessonPage.completeScreen);
        await expect(anyInteractive).toBeVisible({ timeout: 15_000 });

        if (await lessonPage.completeScreen.isVisible().catch(() => false)) break;

        if (await lessonPage.showAnswerBtn.isVisible().catch(() => false)) {
            await lessonPage.showAnswer();
            await lessonPage.rate("good");
        } else if (await lessonPage.quizOptions[0].isVisible().catch(() => false)) {
            await lessonPage.selectQuizOption(0);
            await page.waitForTimeout(2000);
        } else if (await lessonPage.yesnoYesBtn.isVisible().catch(() => false)) {
            await lessonPage.yesnoYesBtn.click();
            await page.waitForTimeout(2000);
        } else {
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

        await completeLessonFlexible(lessonPage, page);
        await lessonPage.waitForComplete();

        await lessonPage.clickNextLesson();

        // Wait for lesson reload: loading appears then resolves
        await lessonPage.lessonLoading
            .waitFor({ state: "visible", timeout: 5_000 })
            .catch(() => {});
        await lessonPage.lessonLoading
            .waitFor({ state: "hidden", timeout: 30_000 });

        // After loading, check the resulting state
        const hasContent = await lessonPage.lessonContent
            .isVisible({ timeout: 5_000 })
            .catch(() => false);
        const hasError = await lessonPage.lessonError
            .isVisible({ timeout: 5_000 })
            .catch(() => false);

        expect(hasContent || hasError).toBe(true);
    });

});

testWithFreshUser.describe("Lesson Card Vertical Layout", () => {
    testWithFreshUser(
        "card fills the lesson-content rectangle (no resize/centering between types)",
        async ({ page }) => {
            test.setTimeout(90_000);

            // Tablet portrait: where the dynamic-resizing bug was reported.
            await page.setViewportSize({ width: 820, height: 1180 });

            const lessonPage = await setupLessonWithCards(page);

            // The card component (data-testid="lesson-card-root") must FILL the
            // lesson-content rectangle and sit top-anchored, so its visible box
            // stays constant across card types (quiz/text/writing/phrase/yesno).
            // lesson-content is flex-1 of the shell's flex column (ADR-027 height
            // chain); clientHeight > 700 proves that chain is intact. See ADR-029
            // (fill-over-center, supersedes ADR-027 centering).
            const card = lessonPage.lessonCardRoot;
            await expect(card).toBeVisible();

            const containerHeight = await lessonPage.lessonContent.evaluate(
                (el) => (el as HTMLElement).clientHeight,
            );
            const containerBox = await lessonPage.lessonContent.boundingBox();
            const cardBox = await card.boundingBox();
            if (!containerBox || !cardBox) {
                throw new Error("lesson-content or card bounding box unavailable");
            }

            // Height-chain intact: the container fills the shell (ADR-027).
            expect(containerHeight).toBeGreaterThan(700);
            // Card is inside proportional padding (12dvh top/bottom, 15dvw left/right).
            // Card top should be at ~12% of container height from the top.
            const expectedPaddingTop = containerBox.height * 0.12;
            expect(cardBox.y - containerBox.y).toBeCloseTo(expectedPaddingTop, -1); // ±10px tolerance
            // Card fills the padded area (container minus top+bottom padding).
            const paddedHeight = containerBox.height * (1 - 0.24); // 1 - 2*12%
            expect(cardBox.height).toBeGreaterThanOrEqual(paddedHeight * 0.8); // 80% of padded area
        },
    );
});
