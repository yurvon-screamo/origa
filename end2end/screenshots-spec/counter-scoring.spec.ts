/**
 * Screenshots for #808: onboarding-scoring counter-suffix card.
 *
 * The scoring queue is ordered by CardType::sort_order, counters are the
 * LAST section (sort_order 4), so the walk dismisses every preceding card
 * with «Не знаю» (per-click skip records are persisted, so the run is
 * resumable in principle). The served build decides what the counter card
 * looks like: master (before — dictionary furigana 時→とき + audio button)
 * versus the fix (after — plain text, no audio button). Which build is
 * under port 1420 is decided OUTSIDE this spec; SHOT_LABEL names the
 * output files.
 *
 * Run: bunx playwright test screenshots-spec/counter-scoring.spec.ts -c playwright.screenshots.config.ts
 * Env: SHOT_LABEL=after|before, SHOT_OUT_DIR=/tmp/opencode/screens-808
 */
import { test, expect } from "@playwright/test";
import { LoginPage } from "../pages/login.page";
import { getAdminToken, createTestUser } from "../fixtures/admin";
import { completeOnboardingToScoring } from "../helpers/onboarding";
import { generateUniqueEmail, DEFAULT_TEST_PASSWORD } from "../helpers/auth";

const COUNTER_TAG = "Счётный суффикс";
const LABEL = process.env.SHOT_LABEL ?? "after";
const OUT_DIR = process.env.SHOT_OUT_DIR ?? "/tmp/opencode/screens-808";
// The «before» build (master) is served on 1421; the fix runs on 1420.
const IS_BEFORE = LABEL === "before";
test.use({ baseURL: IS_BEFORE ? "http://localhost:1421" : "http://localhost:1420" });

test("counter scoring card screenshots", async ({ page }) => {
    test.setTimeout(1_200_000);

    const { token, csrfToken } = await getAdminToken();
    const email = generateUniqueEmail();
    await createTestUser(token, csrfToken, email, DEFAULT_TEST_PASSWORD);

    // Pre-approve the one-time resource download before any app script
    // runs — otherwise the consent overlay intercepts every click on a
    // fresh browser context. Unconditional origin: the same spec serves
    // both builds (1420 fix / 1421 master).
    await page.context().addInitScript(() => {
        window.localStorage.setItem("origa_resource_download_consented", "true");
    });

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

    const reached = await completeOnboardingToScoring(page, { level: "N5" });
    expect(reached, "scoring step must be reachable for a fresh N5 user").toBe(true);

    // Walk the queue: counters are the last section (sort_order 4 — after
    // Grammar/Kanji/Vocabulary/Phrase). «Не знаю» advances and persists the
    // skip; the button re-enables when the per-click save settles.
    const step = page.getByTestId("onboarding-scoring-step");
    const tag = step.locator("span.tag").first();
    const dontKnow = page.getByTestId("scoring-step-dont-know");
    let reached_counter = false;
    for (let i = 0; i < 1500; i++) {
        await tag.waitFor({ state: "visible", timeout: 30_000 });
        const label = ((await tag.textContent()) ?? "").trim();
        if (label === COUNTER_TAG) {
            reached_counter = true;
            break;
        }
        await dontKnow.click();
        await expect(dontKnow).toBeEnabled({ timeout: 15_000 });
    }
    expect(reached_counter, "counter card must appear in the N5 scoring queue").toBe(true);

    // The fix's DOM contract: counter question is plain text (no
    // dictionary furigana) and the card offers no audio button. On master
    // assert the inverse — furigana question + audio button present.
    if (IS_BEFORE) {
        await expect(page.getByTestId("scoring-step-question")).toBeVisible();
        await expect(page.getByTestId("scoring-step-audio")).toBeVisible();
    } else {
        await expect(page.getByTestId("scoring-step-question-plain")).toBeVisible();
        await expect(page.getByTestId("scoring-step-audio")).toHaveCount(0);
    }

    await page.setViewportSize({ width: 1280, height: 800 });
    await page.waitForTimeout(500);
    await page.screenshot({ path: `${OUT_DIR}/scoring-counter-${LABEL}-desktop.png` });

    await page.setViewportSize({ width: 390, height: 844 });
    await page.waitForTimeout(500);
    await page.screenshot({ path: `${OUT_DIR}/scoring-counter-${LABEL}-mobile.png` });
});
