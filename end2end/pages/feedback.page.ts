import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

/**
 * Feedback modal (issue #414, ADR-053). Opened contextually from the
 * translator token popup or the lesson header button — never navigated to.
 */
export class FeedbackPage extends BasePage {
    readonly modal: Locator;
    readonly subject: Locator;
    readonly messageInput: Locator;
    readonly submitButton: Locator;
    readonly cancelButton: Locator;
    readonly retryButton: Locator;
    readonly unavailableInfo: Locator;
    readonly errorAlert: Locator;
    readonly successBlock: Locator;

    constructor(page: Page) {
        super(page);
        this.modal = page.getByTestId("feedback-modal");
        this.subject = page.getByTestId("feedback-subject");
        this.messageInput = page.getByTestId("feedback-message-input");
        this.submitButton = page.getByTestId("feedback-submit-btn");
        this.cancelButton = page.getByTestId("feedback-cancel-btn");
        this.retryButton = page.getByTestId("feedback-retry-btn");
        this.unavailableInfo = page.getByTestId("feedback-unavailable");
        this.errorAlert = page.getByTestId("feedback-error");
        this.successBlock = page.getByTestId("feedback-success");
    }

    async expectOpen(): Promise<void> {
        await expect(this.modal).toBeVisible();
        await expect(this.messageInput).toBeVisible();
    }

    async expectClosed(): Promise<void> {
        await expect(this.modal).not.toBeVisible();
    }

    async fillMessage(text: string): Promise<void> {
        await this.messageInput.fill(text);
    }

    async submit(): Promise<void> {
        await this.submitButton.click();
    }
}
