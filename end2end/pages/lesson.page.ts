import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

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

    // Sync indicator
    readonly syncIndicator: Locator;
    readonly syncSpinner: Locator;

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

        // Sync indicator
        this.syncIndicator = page.getByTestId("lesson-sync-indicator");
        this.syncSpinner = page.getByTestId("lesson-sync-spinner");
    }

    async goto(): Promise<void> {
        await this.navigate("/lesson");
    }

    async gotoFixation(): Promise<void> {
        await this.navigate("/lesson?mode=fixation");
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
}