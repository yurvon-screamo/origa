import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { FeedbackPage, LessonPage } from "../../pages";

// ═══════════════════════════════════════════════════════════════════════
// Lesson header report button (ADR-055)
// ═══════════════════════════════════════════════════════════════════════

// The alert-triangle button is disabled while the card shows its question
// side: the report subject is built from the revealed answer, and an early
// report would spoil the answer (see ADR-055).
Then("кнопка проблемы на уроке неактивна", async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.reportButton).toBeVisible();
    await expect(lessonPage.reportButton).toBeDisabled();
});

When("пользователь нажимает кнопку проблемы на уроке", async ({ page }) => {
    const lessonPage = new LessonPage(page);
        await expect(lessonPage.reportButton).toBeEnabled();
    await lessonPage.reportButton.click();
});

// Opened modal must carry the card as the subject: the Japanese surface of
// the card being studied is visible in the subject block.
Then("открылась форма обратной связи с предметом карточки", async ({ page }) => {
    const feedbackPage = new FeedbackPage(page);
    await feedbackPage.expectOpen();
    await expect(feedbackPage.subject).toBeVisible();
    const subjectText = (await feedbackPage.subject.textContent()) ?? "";
    expect(subjectText.trim().length, "subject must carry the card surface").toBeGreaterThan(0);
});

// ═══════════════════════════════════════════════════════════════════════
// Modal form
// ═══════════════════════════════════════════════════════════════════════

When("пользователь заполняет текст проблемы {string}", async ({ page }, text: string) => {
    const feedbackPage = new FeedbackPage(page);
    await feedbackPage.fillMessage(text);
});

When("пользователь отправляет форму обратной связи", async ({ page }) => {
    const feedbackPage = new FeedbackPage(page);
    await feedbackPage.submit();
});

// The e2e build compiles without SENTRY_DSN (see ci.yml e2e-build), so a
// submission surfaces the informational "release builds only" state — not
// an error (ADR-055: empty compile-time DSN is the single legit trigger).
Then(
    "отображается подсказка о доступности в релизных сборках",
    async ({ page }) => {
        const feedbackPage = new FeedbackPage(page);
        await expect(feedbackPage.unavailableInfo).toBeVisible();
        await expect(feedbackPage.errorAlert).not.toBeVisible();
    },
);

Then("кнопка отправки формы обратной связи неактивна", async ({ page }) => {
    const feedbackPage = new FeedbackPage(page);
    await expect(feedbackPage.submitButton).toBeVisible();
    await expect(feedbackPage.submitButton).toBeDisabled();
});
