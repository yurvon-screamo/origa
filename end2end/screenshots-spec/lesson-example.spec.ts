/**
 * Lesson screenshots for #528 (admin-session variant): mint session is
 * injected into localStorage — the local profile is created by onboarding,
 * the records API is not needed for the local-only flow.
 */
import { test, expect } from "@playwright/test";
import { HomePage } from "../pages/home.page";
import { LoginPage } from "../pages/login.page";
import { LessonPage } from "../pages/lesson.page";
import { completeOnboardingToScoring } from "../helpers/onboarding";
import { getAdminToken, createTestUser } from "../fixtures/admin";
import { generateUniqueEmail, DEFAULT_TEST_PASSWORD } from "../helpers/auth";

test("lesson example screenshots", async ({ page }) => {
    test.setTimeout(600_000);

    const { token, csrfToken } = await getAdminToken();
    const email = generateUniqueEmail();
    await createTestUser(token, csrfToken, email, DEFAULT_TEST_PASSWORD);
    const userPassword = DEFAULT_TEST_PASSWORD;

    const login = new LoginPage(page);
    // Pre-approve the resource download before any app script runs
    // (addInitScript, same as helpers/auth uiLogin).
    await page.context().addInitScript(() => {
        if (window.location.origin === "http://localhost:1420") {
            window.localStorage.setItem("origa_resource_download_consented", "true");
        }
    });
    // Full-retry login loop (mirrors helpers/auth uiLogin): each attempt
    // reloads the page so the form hydrates from scratch.
    // Session-injection: the first successful run's profile already lives
    // in the fixture depot (records API 405s break the login-time profile
    // sync on a fresh depot, so the form login is bypassed). Tokens minted
    // via `trail user mint`.
    const { token: mintToken } = await getAdminToken();
    const mintRes = await page.request.post(
        "http://127.0.0.1:4000/api/auth/v1/mint",
        { headers: { Authorization: `Bearer ${mintToken}` },
          data: { email: email } },
    ).catch(() => null);
    let sess = null;
    if (mintRes && mintRes.ok()) {
        const mint = await mintRes.json();
        const claims = JSON.parse(
            atob(mint.auth_token.split(".")[1].replace(/-/g, "+").replace(/_/g, "/") +
                "==".slice(0, (4 - (mint.auth_token.split(".")[1].length % 4)) % 4)),
        );
        sess = {
            auth_token: mint.auth_token,
            refresh_token: mint.refresh_token,
            email: email,
            trailbase_id: claims.sub,
            record_id: null,
            expires_at: claims.exp,
        };
    }
    await page.goto("/");
    if (sess) {
        await page.evaluate((s) => {
            localStorage.setItem("trailbase_session", JSON.stringify(s));
            localStorage.setItem("origa_resource_download_consented", "true");
        }, sess);
        await page.reload();
    }

    const home = new HomePage(page);
    if (page.url().includes("onboarding")) {
        await completeOnboardingToScoring(page, { level: "N4" });
        // Interactive assessment starts right after scoring-ready: mark
        // everything known (confirm modal) to reach the finish button.
        const markAll = page.getByTestId("onboarding-mark-all-known");
        await markAll.waitFor({ state: "visible", timeout: 60_000 }).catch(() => undefined);
        if (await markAll.isVisible().catch(() => false)) {
            await markAll.click();
            await page
                .getByTestId("onboarding-confirm-ok")
                .click({ timeout: 10_000 })
                .catch(() => undefined);
        }
        await expect(page.getByTestId("scoring-step-complete")).toBeVisible({
            timeout: 120_000,
        });
        const { OnboardingPage } = await import("../pages/onboarding.page");
        const onboarding = new OnboardingPage(page);
        await onboarding.clickFinish();
        await page.waitForURL(/\/home$/, { timeout: 300_000 });
    }
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
