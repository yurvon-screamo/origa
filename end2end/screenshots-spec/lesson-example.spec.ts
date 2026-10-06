/**
 * Lesson screenshots for #528: fresh e2e user (admin-API creation), Skip
 * onboarding assessment (cards stay new -> lesson becomes available), then
 * acquaintance slide + answer-side screenshots with the word example.
 * Mirrors the proven login/onboarding flow of word-examples.spec.
 */
import { test, expect } from "@playwright/test";
import { HomePage } from "../pages/home.page";
import { LoginPage } from "../pages/login.page";
import { LessonPage } from "../pages/lesson.page";
import { getAdminToken, createTestUser } from "../fixtures/admin";
import { completeOnboardingToScoring } from "../helpers/onboarding";
import { generateUniqueEmail, DEFAULT_TEST_PASSWORD } from "../helpers/auth";

test("lesson example screenshots", async ({ page }) => {
    test.setTimeout(600_000);

    const { token, csrfToken } = await getAdminToken();
    const email = generateUniqueEmail();
    await createTestUser(token, csrfToken, email, DEFAULT_TEST_PASSWORD);

    // Pre-approve the one-time resource download before any app script
    // runs — otherwise the consent overlay intercepts every click on a
    // fresh browser context.
    await page.context().addInitScript(() => {
        if (window.location.origin === "http://localhost:1420") {
            window.localStorage.setItem("origa_resource_download_consented", "true");
        }
    });

    // Cold-start flake retries, same as word-examples.spec.
    const login = new LoginPage(page);
    let loggedIn = false;
    for (let attempt = 1; attempt <= 3 && !loggedIn; attempt++) {
        await page.goto("/", { waitUntil: "domcontentloaded" });
        await page.evaluate(() => localStorage.clear());
        try {
            await login.expandPasswordForm();
            await login.fillEmail(email);
            await login.fillPassword(DEFAULT_TEST_PASSWORD);
            await login.submit();
            await page.waitForURL(/\/(home|onboarding)$/, { timeout: 120_000 });
            loggedIn = true;
        } catch {
            loggedIn = false;
        }
    }
    expect(loggedIn, "login must succeed within 3 attempts").toBe(true);

    if (page.url().includes("onboarding")) {
        await completeOnboardingToScoring(page, { level: "N4" });
        // SKIP the interactive assessment: every card stays new — the
        // fresh profile gets due cards and the lesson button appears.
        const skipButton = page.getByTestId("onboarding-skip");
        await skipButton
            .waitFor({ state: "visible", timeout: 60_000 })
            .catch(() => undefined);
        if (await skipButton.isVisible().catch(() => false)) {
            await skipButton.click();
            await page
                .getByTestId("onboarding-confirm-ok")
                .click({ timeout: 10_000 })
                .catch(() => undefined);
        }
        // After skip the flow lands either on the completed scoring step
        // (finish button) or straight on home.
        const { OnboardingPage } = await import("../pages/onboarding.page");
        const onboarding = new OnboardingPage(page);
        const finishVisible = await onboarding.finishButton
            .waitFor({ state: "visible", timeout: 90_000 })
            .then(() => true)
            .catch(() => false);
        if (finishVisible) {
            await onboarding.clickFinish();
        }
        await page.waitForURL(/\/home$/, { timeout: 300_000 });
    }

    const home = new HomePage(page);
    const wordsNav = page.getByRole("link", { name: "Words" });
    await wordsNav.waitFor({ state: "visible", timeout: 600_000 });
    await home.startLesson();

    const lesson = new LessonPage(page);
    await lesson.expectLessonVisible();

    let shot = false;
    try {
        await expect(page.getByTestId("acquaintance-word-slide")).toBeVisible({
            timeout: 30_000,
        });
        await expect(page.getByTestId("lesson-word-example")).toBeVisible({
            timeout: 30_000,
        });
        await page.waitForTimeout(1500);
        await page.screenshot({ path: "screenshots/lesson-acquaintance-example.png" });
        shot = true;
    } catch {
        // Lesson schedule may open with a different view — walk answers.
    }

    for (let i = 0; i < 8 && !shot; i++) {
        try {
            await lesson.showAnswer();
            await expect(page.getByTestId("lesson-word-example")).toBeVisible({
                timeout: 8_000,
            });
            await page.waitForTimeout(800);
            await page.screenshot({ path: "screenshots/lesson-answer-example.png" });
            shot = true;
        } catch {
            await lesson.clickNextCard().catch(() => undefined);
        }
    }
    expect(shot, "at least one lesson example screenshot").toBe(true);
});
