/**
 * Screenshots for issue #528 (word examples): create a test user, log in,
 * open the word detail page for a translated word, capture hero + examples.
 */
import { test, expect } from "@playwright/test";
import { LoginPage } from "../pages/login.page";
import { getAdminToken, createTestUser } from "../fixtures/admin";
import { completeOnboardingToScoring } from "../helpers/onboarding";
import { generateUniqueEmail, DEFAULT_TEST_PASSWORD } from "../helpers/auth";

const WORD = process.env.SHOT_WORD || "あさ";

test("word detail screenshots", async ({ page }) => {
    test.setTimeout(180_000);

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

    // Cold-start flake: the resource-loading overlay can intercept the
    // first form interaction while bundles download — full retries, like
    // helpers/auth uiLogin. NOTE: the localStorage.clear() below runs
    // after the app document booted, so the consent flag set by the init
    // script has already been read — the clear only wipes stale sessions
    // from earlier manual debugging.
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
        // Fixture depot runs the interactive assessment: mark everything
        // known (with the confirm modal) to reach the finish button.
        const markAll = page.getByTestId("onboarding-mark-all-known");
        if (await markAll.isVisible({ timeout: 10_000 }).catch(() => false)) {
            await markAll.click();
            const confirmBtn = page.getByTestId("onboarding-confirm-ok");
            await confirmBtn.click({ timeout: 10_000 }).catch(() => undefined);
        }
        await expect(page.getByTestId("scoring-step-complete")).toBeVisible({
            timeout: 120_000,
        });
        const { OnboardingPage } = await import("../pages/onboarding.page");
        const onboarding = new OnboardingPage(page);
        await onboarding.clickFinish();
        await page.waitForURL(/\/home$/, { timeout: 300_000 });
    }

    const wordsNav = page.getByRole("link", { name: "Words" });
    await wordsNav.waitFor({ state: "visible", timeout: 600_000 });
    await expect(wordsNav).toBeVisible();

    // Word detail page: hero + translations section + examples.
    await page.goto(`/words/${encodeURIComponent(WORD)}`);
    await expect(page.getByTestId("word-detail-word")).toBeVisible({ timeout: 60_000 });
    await page.waitForTimeout(3000); // examples chunk fetch
    await page.screenshot({ path: "screenshots/word-detail-full.png", fullPage: true });
    await page.screenshot({ path: "screenshots/word-detail-viewport.png" });
});
