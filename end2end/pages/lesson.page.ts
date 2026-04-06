import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

type Rating = "again" | "hard" | "good" | "easy";

export class LessonPage extends BasePage {
    // Page structure
    readonly lessonPage: Locator;
    readonly lessonCard: Locator;
    readonly lessonHeader: Locator;

    // Navigation
    readonly backButton: Locator;
    readonly muteButton: Locator;

    // Loading states
    readonly lessonLoading: Locator;
    readonly lessonSpinner: Locator;
    readonly lessonLoadingText: Locator;

    // Error states
    readonly lessonError: Locator;

    // Content
    readonly lessonContent: Locator;

    // Card interaction
    readonly showAnswerBtn: Locator;
    readonly ratingAgain: Locator;
    readonly ratingHard: Locator;
    readonly ratingGood: Locator;
    readonly ratingEasy: Locator;
    readonly progressText: Locator;

    // Quiz
    readonly quizOptions: readonly [Locator, Locator, Locator, Locator];

    // Yes/No
    readonly yesnoYesBtn: Locator;
    readonly yesnoNoBtn: Locator;

    // Lesson completion
    readonly completeScreen: Locator;
    readonly completeBackBtn: Locator;

    // Sync indicator
    readonly syncIndicator: Locator;
    readonly syncSpinner: Locator;

    private readonly ratingMap: Record<Rating, Locator>;

    constructor(page: Page) {
        super(page);

        // Page structure
        this.lessonPage = page.getByTestId("lesson-page");
        this.lessonCard = page.getByTestId("lesson-card");
        this.lessonHeader = page.getByTestId("lesson-header");

        // Navigation
        this.backButton = page.getByTestId("lesson-back-btn");
        this.muteButton = page.getByTestId("lesson-mute-btn");

        // Loading states
        this.lessonLoading = page.getByTestId("lesson-loading");
        this.lessonSpinner = page.getByTestId("lesson-spinner");
        this.lessonLoadingText = page.getByTestId("lesson-loading-text");

        // Error states
        this.lessonError = page.getByTestId("lesson-error");

        // Content
        this.lessonContent = page.getByTestId("lesson-content");

        // Card interaction
        this.showAnswerBtn = page.getByTestId("lesson-show-answer-btn");
        this.ratingAgain = page.getByTestId("lesson-rating-btn-again");
        this.ratingHard = page.getByTestId("lesson-rating-btn-hard");
        this.ratingGood = page.getByTestId("lesson-rating-btn-good");
        this.ratingEasy = page.getByTestId("lesson-rating-btn-easy");
        this.progressText = page.getByTestId("lesson-progress-text");

        // Quiz
        this.quizOptions = [
            page.getByTestId("quiz-option-0"),
            page.getByTestId("quiz-option-1"),
            page.getByTestId("quiz-option-2"),
            page.getByTestId("quiz-option-3"),
        ] as const;

        // Yes/No
        this.yesnoYesBtn = page.getByTestId("yesno-yes-btn");
        this.yesnoNoBtn = page.getByTestId("yesno-no-btn");

        // Lesson completion
        this.completeScreen = page.getByTestId("lesson-complete-screen");
        this.completeBackBtn = page.getByTestId("lesson-complete-back-btn");

        // Sync indicator
        this.syncIndicator = page.getByTestId("lesson-sync-indicator");
        this.syncSpinner = page.getByTestId("lesson-sync-spinner");

        this.ratingMap = {
            again: this.ratingAgain,
            hard: this.ratingHard,
            good: this.ratingGood,
            easy: this.ratingEasy,
        };
    }

    async goto(): Promise<void> {
        await this.navigate("/lesson");
    }

    async expectLessonVisible(): Promise<void> {
        await expect(this.lessonPage).toBeVisible();
        await expect(this.lessonCard).toBeVisible();
        await expect(this.lessonHeader).toBeVisible();
    }

    async waitForLoading(): Promise<void> {
        await expect(this.lessonLoading).toBeVisible({ timeout: 10_000 });
        await expect(this.lessonLoading).toBeHidden({ timeout: 30_000 });
    }

    async clickBack(): Promise<void> {
        await this.backButton.click();
    }

    async toggleMute(): Promise<void> {
        await this.muteButton.click();
    }

    async expectContentVisible(): Promise<void> {
        await expect(this.lessonContent).toBeVisible();
    }

    async expectErrorVisible(): Promise<void> {
        await expect(this.lessonError).toBeVisible();
    }

    async waitForSync(): Promise<void> {
        await expect(this.syncIndicator).toBeVisible({ timeout: 10_000 });
        await expect(this.syncIndicator).toBeHidden({ timeout: 30_000 });
    }

    async showAnswer(): Promise<void> {
        await this.showAnswerBtn.click();
    }

    async rate(rating: Rating): Promise<void> {
        await this.ratingMap[rating].click();
    }

    async getProgressText(): Promise<string> {
        const text = await this.progressText.textContent();
        return text ?? "";
    }

    async waitForComplete(): Promise<void> {
        await expect(this.completeScreen).toBeVisible({ timeout: 30_000 });
    }

    async selectQuizOption(index: number): Promise<void> {
        await this.quizOptions[index].click();
    }
}