/**
 * PR screenshots for #735 (lesson UI unification).
 *
 * Drives a fresh user through the lesson and captures the unified card
 * surfaces at both review viewports (desktop 1280x800, mobile 390x844):
 * acquaintance slides/training, then the review body cards (question +
 * answer per card, type-detected by DOM markers).
 *
 * Usage: node scripts/pr-screenshots-lesson-unification.mjs
 * (expects the e2e web servers: app on :1420, CDN on :8080, TrailBase
 * on :4000 — same recipe as playwright.screenshots.config.ts)
 *
 * The script writes to /tmp/opencode/pr-screens-735; the committed
 * artifacts under .pr-screenshots/issue-735/ are a hand-picked copy of
 * that output (cp after review).
 *
 * The admin credentials below are the local e2e fixture defaults
 * (end2end/README.md: ORIGA_ADMIN_PASSWORD=secret) — no real secrets.
 */
import { chromium } from "@playwright/test";
import { mkdirSync } from "fs";
import { join } from "path";

const OUT = "/tmp/opencode/pr-screens-735";
mkdirSync(OUT, { recursive: true });

const BASE = "http://localhost:1420";
const TRAILBASE = "http://127.0.0.1:4000";
const VIEWPORTS = [
    { tag: "desktop", width: 1280, height: 800 },
    { tag: "mobile", width: 390, height: 844 },
];

const runId = `shot-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;

async function adminToken() {
    const res = await globalThis.fetch(`${TRAILBASE}/api/auth/v1/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: "admin@localhost", password: "secret" }),
    });
    if (!res.ok) throw new Error(`admin login failed: ${res.status}`);
    const data = await res.json();
    return { token: data.auth_token, csrfToken: data.csrf_token };
}

async function createUser(email, password) {
    const { token, csrfToken } = await adminToken();
    const res = await globalThis.fetch(`${TRAILBASE}/api/_admin/user`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
            "csrf-token": csrfToken,
        },
        body: JSON.stringify({ email, password, verified: true, admin: false }),
    });
    if (!res.ok) throw new Error(`create user failed: ${res.status} ${await res.text()}`);
    console.log(`[admin] Created ${email}`);
}

const DEFAULT_PASSWORD = "Passw0rd!train";

async function loginAndReachLesson(page, email) {
    // Cold-start flake retries, same as the proven e2e specs: the WASM
    // payload is heavy and the first load can stall.
    let loggedIn = false;
    for (let attempt = 1; attempt <= 3 && !loggedIn; attempt++) {
        await page.goto(`${BASE}/`, { waitUntil: "domcontentloaded" });
        await page.evaluate(() => {
            localStorage.clear();
            // Re-seed AFTER the clear (the clear would otherwise wipe the
            // initScript-seeded flag): gloo reads it during WASM boot,
            // which starts after this evaluate returns.
            localStorage.setItem("origa_resource_download_consented", "true");
        });
        try {
            const toggle = page.getByTestId("login-password-toggle");
            await toggle.waitFor({ state: "visible", timeout: 120_000 });
            await toggle.click();
            await page.getByTestId("email-input").fill(email);
            await page.getByTestId("password-input").fill(DEFAULT_PASSWORD);
            await page
                .getByTestId("login-form")
                .locator("button[type=submit]")
                .click();
            await page.waitForURL(/\/(home|onboarding)$/, { timeout: 180_000 });
            loggedIn = true;
        } catch {
            loggedIn = false;
        }
    }
    if (!loggedIn) throw new Error("login must succeed within 3 attempts");

    await page.waitForURL(/\/(home|onboarding)$/, { timeout: 180_000 });

    // Onboarding: import the corpus (skipApps — nothing is excluded), then
    // SKIP the scoring so every card stays new. The fresh deck gets due
    // cards and the lesson opens with the acquaintance hand (mirrors the
    // proven lesson-example screenshot flow).
    const onboardingUrl = page.url();
    if (onboardingUrl.includes("/onboarding")) {
        await page
            .getByTestId("onboarding-spinner")
            .waitFor({ state: "hidden", timeout: 30_000 })
            .catch(() => {});
        await page.getByTestId("onboarding-next").click();
        await page
            .getByTestId("onboarding-load-step")
            .waitFor({ state: "visible", timeout: 10_000 });
        await page.getByTestId("onboarding-next").click();
        await page
            .getByTestId("onboarding-jlpt-step")
            .waitFor({ state: "visible", timeout: 10_000 });
        await page.getByTestId("jlpt-option-n5").click();
        await page
            .getByTestId("jlpt-option-n5")
            .waitFor({ state: "visible", timeout: 5_000 });
        await page.getByTestId("onboarding-next").click();
        await page
            .getByTestId("onboarding-apps-step")
            .waitFor({ state: "visible", timeout: 10_000 });
        // No app selection: app progress excludes studied sets from the
        // import, and the full corpus is what seeds the fresh deck.
        await page.getByTestId("onboarding-next").click();
        await page
            .getByTestId("onboarding-progress-step")
            .waitFor({ state: "visible", timeout: 10_000 });
        await page.getByTestId("onboarding-next").click();
        await page
            .getByTestId("onboarding-summary-step")
            .waitFor({ state: "visible", timeout: 10_000 });
        await page.getByTestId("onboarding-import").click();
        await page
            .getByTestId("onboarding-scoring-step")
            .waitFor({ state: "visible", timeout: 300_000 });
        const skipScoring = page.getByTestId("onboarding-skip-scoring");
        await skipScoring
            .waitFor({ state: "visible", timeout: 90_000 })
            .catch(() => {});
        if (await skipScoring.isVisible().catch(() => false)) {
            await skipScoring.click();
            const confirmBtn = page.getByTestId("onboarding-confirm-ok");
            await confirmBtn
                .waitFor({ state: "visible", timeout: 10_000 })
                .catch(() => {});
            await confirmBtn.click({ timeout: 10_000 }).catch(() => {});
        }
        const finish = page.getByTestId("onboarding-finish");
        if (await finish.isVisible().catch(() => false)) {
            await finish.click();
        }
    }
    await page.waitForURL(/\/home$/, { timeout: 300_000 });

    const lessonButton = page.getByTestId("home-welcome-lesson");
    await lessonButton.waitFor({ state: "visible", timeout: 600_000 });
    await lessonButton.click();
    await page.getByTestId("lesson-page").waitFor({ timeout: 60_000 });
}

async function clickFirstVisible(page, testIds, timeout = 1_500) {
    for (const id of testIds) {
        const locator = page.getByTestId(id);
        // isVisible() returns immediately (ignores timeouts) — a mounting
        // card would read as absent. waitFor actually waits.
        const visible = await locator
            .waitFor({ state: "visible", timeout })
            .then(() => true)
            .catch(() => false);
        if (visible) {
            await locator.click({ timeout: 3_000 });
            return id;
        }
    }
    return null;
}

async function shot(page, name) {
    await page.waitForTimeout(700);
    await page.screenshot({ path: join(OUT, `${name}.png`) });
    console.log(`  captured ${name}`);
}

async function captureAcquaintance(page, tag) {
    // The hand appears after the lesson data prep — allow a long window.
    // `acquaintance-view` wraps the whole hand (the first slide may be a
    // word, kanji, grammar or counter slide).
    const hand = page.getByTestId("acquaintance-view");
    const handVisible = await hand
        .waitFor({ state: "visible", timeout: 90_000 })
        .then(() => true)
        .catch(() => false);
    if (!handVisible) {
        return false;
    }
    await shot(page, `${tag}-01-acquaintance-presentation`);

    // Presentation → Next through the slides until the training takes over.
    for (let i = 0; i < 12; i++) {
        const training = await page
            .getByTestId("acquaintance-training")
            .isVisible({ timeout: 1_000 })
            .catch(() => false);
        if (training) break;
        const next = page.getByTestId("acquaintance-next-btn");
        if (!(await next.isVisible().catch(() => false))) break;
        await next.click({ timeout: 3_000 });
        await page.waitForTimeout(700);
    }

    // Training: reveal the answer (RevealAnswerButton) and self-assess.
    // Two rounds are shot, then the hand is driven to completion.
    let rounds = 0;
    for (let i = 0; i < 60; i++) {
        const training = await page
            .getByTestId("acquaintance-training")
            .isVisible()
            .catch(() => false);
        if (!training) break;
        const revealed = await clickFirstVisible(page, ["acquaintance-reveal-btn"], 2_000);
        if (revealed) {
            rounds += 1;
            if (rounds <= 2) {
                await shot(page, `${tag}-0${rounds + 1}-acquaintance-training-answer`);
            }
        }
        const rated = await clickFirstVisible(
            page,
            ["acquaintance-rating-remember", "acquaintance-reveal-btn", "acquaintance-next-btn"],
            2_000,
        );
        if (!rated) await page.waitForTimeout(500);
    }

    // The hand closes into the lesson complete screen (fresh user: no due
    // review cards yet) or the review body — shoot whichever appears.
    const complete = page.getByTestId("lesson-complete-screen");
    const completeShown = await complete
        .waitFor({ state: "visible", timeout: 20_000 })
        .then(() => true)
        .catch(() => false);
    if (completeShown) {
        await shot(page, `${tag}-04-lesson-complete`);
    }
    return true;
}

const QUESTION_MARKERS = [
    ["quiz", 'button[data-testid^="quiz-option-"]'],
    ["yesno", '[data-testid="yesno-yes-btn"]'],
    ["audio-recall", '[data-testid="audio-recall-play-btn"]'],
    ["counter-bindings", '[data-testid="counter-binding-front"]'],
    ["normal", '[data-testid="lesson-show-answer-btn"]'],
];

const REVEALS = [
    "lesson-show-answer-btn",
    "quiz-option-0",
    "yesno-no-btn",
    "audio-recall-reveal-btn",
    "lesson-example-reveal",
    "counter-binding-show-answer-btn",
];

const ADVANCES = [
    "lesson-card-next-btn",
    "quiz-submit-btn",
    "lesson-rating-btn-again",
    "audio-recall-know-btn",
    "lesson-example-understood",
    "lesson-example-next",
];

async function captureReviewBody(page, tag) {
    // A fresh user has no due review cards — the review body stays empty
    // and the walk is a no-op. It still covers seeded profiles (imported
    // history) when run against them.
    for (let card = 1; card <= 8; card++) {
        const completeShown = await page
            .getByTestId("lesson-complete-screen")
            .waitFor({ state: "visible", timeout: 2_000 })
            .then(() => true)
            .catch(() => false);
        if (completeShown) break;

        let kind = "unknown";
        for (const [name, selector] of QUESTION_MARKERS) {
            const found = await page
                .locator(selector)
                .first()
                .waitFor({ state: "visible", timeout: 800 })
                .then(() => true)
                .catch(() => false);
            if (found) {
                kind = name;
                break;
            }
        }
        await shot(page, `${tag}-card-${String(card).padStart(2, "0")}-${kind}-question`);

        const revealed = await clickFirstVisible(page, REVEALS, 1_800);
        if (!revealed) continue;
        await shot(page, `${tag}-card-${String(card).padStart(2, "0")}-${kind}-answer`);

        const advanced = await clickFirstVisible(page, ADVANCES, 2_500);
        if (!advanced) break;
        await page.waitForTimeout(1_200);
        // Submit (quiz) lands on the result + «Далее» step — advance past it.
        await clickFirstVisible(page, ["lesson-card-next-btn"], 1_500);
        await page.waitForTimeout(1_200);
    }
}

const browser = await chromium.launch({
    args: ["--disable-dev-shm-usage", "--disable-gpu"],
});

for (const viewport of VIEWPORTS) {
    const context = await browser.newContext({
        viewport: { width: viewport.width, height: viewport.height },
        locale: "ru-RU",
    });
    await context.addInitScript(() => {
        const g = globalThis;
        if (g.location?.origin === "http://localhost:1420") {
            g.localStorage?.setItem("origa_resource_download_consented", "true");
        }
    });
    const page = await context.newPage();
    const email = `${runId}-${viewport.tag}@example.com`;
    await createUser(email, DEFAULT_PASSWORD);
    await loginAndReachLesson(page, email);
    console.log(`[${viewport.tag}] lesson reached`);
    const hadAcquaintance = await captureAcquaintance(page, viewport.tag);
    if (!hadAcquaintance) {
        console.log(`[${viewport.tag}] no acquaintance hand — review body only`);
    }
    await captureReviewBody(page, viewport.tag);
    await context.close();
}

await browser.close();
console.log("screenshots in", OUT);
