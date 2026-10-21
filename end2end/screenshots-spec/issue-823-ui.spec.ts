/**
 * Screenshots for issue #823 (phrases/examples out of nav, words header
 * shortcuts, examples page): create a test user, log in, capture the words
 * page header (both new buttons) and the examples page at desktop and
 * mobile viewports.
 */
import { test, expect } from "@playwright/test";
import { LoginPage } from "../pages/login.page";
import { getAdminToken, createTestUser } from "../fixtures/admin";
import { completeOnboardingToScoring, isOnboardingShown } from "../helpers/onboarding";
import {
    generateUniqueEmail,
    DEFAULT_TEST_PASSWORD,
    waitForLoginComplete,
    restoreDownloadConsent,
} from "../helpers/auth";

const DESKTOP = { width: 1280, height: 800 };
const MOBILE = { width: 390, height: 844 };

test("words header + examples page screenshots", async ({ page }) => {
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

    // Cold-start flake: the resource-loading overlay can intercept the
    // first form interaction while bundles download — full retries, like
    // helpers/auth uiLogin.
    const login = new LoginPage(page);
    let loggedIn = false;
    for (let attempt = 1; attempt <= 3 && !loggedIn; attempt++) {
        await page.setViewportSize(DESKTOP);
        await page.goto("/", { waitUntil: "domcontentloaded" });
        await page.evaluate(() => localStorage.clear());
        await restoreDownloadConsent(page);
        try {
            await login.expandPasswordForm();
            await login.fillEmail(email);
            await login.fillPassword(DEFAULT_TEST_PASSWORD);
            await login.submit();
            await waitForLoginComplete(page);
            loggedIn = true;
        } catch {
            loggedIn = false;
        }
    }
    expect(loggedIn, "login must succeed within 3 attempts").toBe(true);
    if (await isOnboardingShown(page)) {
        await completeOnboardingToScoring(page, { level: "N4" });
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

    // Desktop: words page with the phrases/examples header buttons.
    await page.setViewportSize(DESKTOP);
    await page.goto("/words");
    await expect(page.getByTestId("words-header")).toBeVisible({ timeout: 120_000 });
    await expect(page.getByTestId("words-phrases-btn")).toBeVisible();
    await expect(page.getByTestId("words-examples-btn")).toBeVisible();
    await page.waitForTimeout(2000); // let the vocabulary grid settle
    await page.screenshot({ path: "screenshots/issue-823-desktop-words.png" });

    // Desktop: examples page (a fresh learner has no example cards —
    // the empty state with the header + info tooltip is the honest view).
    await page.getByTestId("words-examples-btn").click();
    await page.waitForURL(/\/examples$/, { timeout: 30_000 });
    await expect(page.getByTestId("examples-page")).toBeVisible({ timeout: 60_000 });
    await expect(page.getByTestId("examples-title")).toBeVisible();
    await page.waitForTimeout(1500);
    await page.screenshot({ path: "screenshots/issue-823-desktop-examples.png" });

    // Mobile: the words header must fit 390px (the #823 overflow check).
    await page.setViewportSize(MOBILE);
    await page.goto("/words");
    await expect(page.getByTestId("words-header")).toBeVisible({ timeout: 60_000 });
    await expect(page.getByTestId("words-phrases-btn")).toBeVisible();
    await expect(page.getByTestId("words-examples-btn")).toBeVisible();
    await page.waitForTimeout(1500);
    await page.screenshot({ path: "screenshots/issue-823-mobile-words.png" });

    await page.goto("/examples");
    await expect(page.getByTestId("examples-page")).toBeVisible({ timeout: 60_000 });
    await page.waitForTimeout(1500);
    await page.screenshot({ path: "screenshots/issue-823-mobile-examples.png" });
});
