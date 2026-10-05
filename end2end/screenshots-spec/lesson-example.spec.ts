/**
 * Lesson screenshots for #528 (admin-session variant): mint session is
 * injected into localStorage — the local profile is created by onboarding,
 * the records API is not needed for the local-only flow.
 */
import { test, expect } from "@playwright/test";
import { execSync } from "child_process";
import { HomePage } from "../pages/home.page";
import { LoginPage } from "../pages/login.page";
import { LessonPage } from "../pages/lesson.page";
import { completeOnboardingToScoring } from "../helpers/onboarding";

test("lesson example screenshots", async ({ page }) => {
    test.setTimeout(600_000);

    // Fixture-depot admin: reset to the known password before login (the
    // depot ships with an unknown random one).
    try {
        execSync(
            "trail --depot ./trailbase-fixture/traildepot user change-password admin@localhost secret",
            { cwd: "..", stdio: "pipe", timeout: 10_000 },
        );
    } catch {
        // already set from a previous run
    }

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
    const home = new HomePage(page);
    if (page.url().includes("onboarding")) {
        await completeOnboardingToScoring(page, { level: "N4" });
        // Interactive assessment starts right after scoring-ready: SKIP it
        // so every card stays new — the fresh profile gets due cards and
        // the lesson button appears on home.
        const skipButton = page.getByTestId("onboarding-skip");
        await skipButton.waitFor({ state: "visible", timeout: 60_000 }).catch(() => undefined);
        if (await skipButton.isVisible().catch(() => false)) {
            await skipButton.click();
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
    // Diagnostic dump: what does home render?
    await page.waitForTimeout(4000);
    const mainHtml = await page
        .locator("main")
        .innerHTML()
        .catch(() => "<no main>");
    console.log("HOME-MAIN:", mainHtml.slice(0, 1500));
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
