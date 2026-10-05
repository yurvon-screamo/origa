/**
 * Screenshots for issue #528 (word examples): create a test user, log in,
 * open the word detail page for a translated word, capture hero + examples.
 */
import { test, expect } from "@playwright/test";
import { LoginPage } from "../pages/login.page";
import { HomePage } from "../pages/home.page";
import { LessonPage } from "../pages/lesson.page";
import { getAdminToken, createTestUser } from "../fixtures/admin";
import { completeOnboardingToScoring } from "../helpers/onboarding";
import { generateUniqueEmail, DEFAULT_TEST_PASSWORD } from "../helpers/auth";

const WORD = process.env.SHOT_WORD || "あさ";

test("word detail screenshots", async ({ page }) => {
    test.setTimeout(180_000);

    const { token, csrfToken } = await getAdminToken();
    const email = generateUniqueEmail();
    await createTestUser(token, csrfToken, email, DEFAULT_TEST_PASSWORD);

    const login = new LoginPage(page);
    const appConsole: string[] = [];
    page.on("console", (msg) => {
        const t = msg.text().replace(/\x1b\[[0-9;]*m/g, "");
        if (/ERROR|WARN|Login|login|profile|merge|records/.test(t)) {
            appConsole.push(`${msg.type()}: ${t.slice(0, 220)}`);
        }
    });
    // Pre-approve the one-time resource download before any app script
    // runs (same trick as helpers/auth.ts uiLogin).
    await page.context().addInitScript(() => {
        if (window.location.origin === "http://localhost:1420") {
            window.localStorage.setItem("origa_resource_download_consented", "true");
        }
    });
    await page.goto("/");
    await login.expandPasswordForm();
    await login.fillEmail(email);
    await login.fillPassword(DEFAULT_TEST_PASSWORD);
    await login.submit();
    await page.waitForURL(/\/(home|onboarding)$/, { timeout: 120_000 });
    if (page.url().includes("onboarding")) {
        await completeOnboardingToScoring(page, { level: "N4" });
        // Fixture depot runs the interactive assessment: mark everything
        // known (with the confirm modal) to reach the finish button.
        const markAll = page.getByTestId("onboarding-mark-all-known");
        if (await markAll.isVisible({ timeout: 10_000 }).catch(() => false)) {
            await markAll.click();
            const confirmBtn = page.getByTestId("onboarding-confirm-ok");
            await confirmBtn
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

    const wordsNav = page.getByRole("link", { name: "Words" });
    await wordsNav.waitFor({ state: "visible", timeout: 600_000 });
    await expect(wordsNav).toBeVisible();

    // Word detail page
    const failedRequests: string[] = [];
    page.on("requestfailed", (req) =>
        failedRequests.push(`FAIL ${req.url().slice(0, 120)}: ${req.failure()?.errorText}`),
    );
    page.on("response", (res) => {
        if (res.status() >= 400) failedRequests.push(`${res.status()} ${res.url().slice(0, 120)}`);
    });
    await page.goto(`/words/${encodeURIComponent(WORD)}`);
    await page.waitForTimeout(8000);
    console.log("APP-CONSOLE:", JSON.stringify(appConsole.slice(-14), null, 1));
    console.log("FAILED-REQUESTS:", JSON.stringify(failedRequests.slice(0, 15), null, 1));
    const mainHtml = await page
        .locator("main")
        .innerHTML()
        .catch(() => "<no main>");
    console.log("MAIN-HTML:", mainHtml.slice(0, 600));
    await page.screenshot({ path: "screenshots/word-detail-diagnostic.png" });
    await expect(page.getByTestId("word-detail-word")).toBeVisible({ timeout: 60_000 });
    await page.waitForTimeout(3000); // examples chunk fetch
    // Layout diagnostics: computed styles of the ruby annotation.
    const rubyStyles = await page.evaluate(() => {
        const dump = (el) => {
            if (!el) return null;
            const cs = getComputedStyle(el);
            return {
                fontFamily: cs.fontFamily.slice(0, 70),
                fontSize: cs.fontSize,
                letterSpacing: cs.letterSpacing,
                lineHeight: cs.lineHeight,
                rubyAlign: cs.rubyAlign,
                rubyPosition: cs.rubyPosition,
                display: cs.display,
            };
        };
        return {
            heroWord: dump(document.querySelector('[data-testid="word-detail-word"]')),
            heroRt: dump(document.querySelector('[data-testid="word-detail-word"] .furigana-rt')),
            heroRuby: dump(document.querySelector('[data-testid="word-detail-word"] .furigana-ruby')),
            exampleJa: dump(document.querySelector('.word-detail-example-ja')),
            exampleRt: dump(document.querySelector('.word-detail-example-ja .furigana-rt')),
        };
    });
    console.log("RUBY-STYLES:", JSON.stringify(rubyStyles, null, 1));
    await page.screenshot({ path: "screenshots/word-detail-full.png", fullPage: true });
    await page.screenshot({ path: "screenshots/word-detail-viewport.png" });

    // ── Lesson: acquaintance slide + answer side with the example ──
    const home = new HomePage(page);
    const lesson = new LessonPage(page);
    await page.goto("/home");
    await expect(page.getByTestId("home-content")).toBeVisible({ timeout: 120_000 });
    await home.startLesson();
    await lesson.expectLessonVisible();

    let lessonShot = false;
    try {
        await expect(page.getByTestId("acquaintance-word-slide")).toBeVisible({
            timeout: 60_000,
        });
        await expect(page.getByTestId("lesson-word-example")).toBeVisible({
            timeout: 30_000,
        });
        await page.waitForTimeout(1200);
        await page.screenshot({ path: "screenshots/lesson-acquaintance-example.png" });
        lessonShot = true;
    } catch {
        // Lesson schedule may start differently — walk answers below.
    }

    for (let i = 0; i < 10 && !lessonShot; i++) {
        try {
            await lesson.showAnswer();
            await page.waitForTimeout(900);
            if (await page.getByTestId("lesson-word-example").isVisible({ timeout: 4_000 }).catch(() => false)) {
                await page.screenshot({ path: "screenshots/lesson-answer-example.png" });
                lessonShot = true;
            }
        } catch {
            // not a word card
        }
        if (lessonShot) break;
        try {
            await lesson.clickNextCard();
        } catch {
            try {
                await lesson.rate("good");
            } catch {
                break;
            }
        }
    }
    expect(lessonShot, "lesson example screenshot must be captured").toBe(true);
});
