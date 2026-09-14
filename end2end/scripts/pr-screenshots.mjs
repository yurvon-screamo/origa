/**
 * PR screenshots for the feedback form (issue #414).
 *
 * Drives the same fixture flow as the BDD scenarios and captures:
 *   1. lesson header with the alert-triangle report button
 *   2. feedback modal opened with the card subject
 *   3. token popup with the "wrong translation?" action line
 *
 * Usage: node scripts/pr-screenshots.mjs  (expects the e2e web servers:
 * app on :1420, CDN on :8080, TrailBase on :4000 — reuseExistingServer)
 */
import { chromium } from "@playwright/test";
import { mkdirSync } from "fs";
import { join } from "path";

const OUT = "/tmp/opencode/pr-screens";
mkdirSync(OUT, { recursive: true });

const BASE = "http://localhost:1420";
const ADMIN = { email: "admin@localhost", password: "secret" };
const TRAILBASE = "http://127.0.0.1:4000";

const runId = `shot-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;

/// Admin login → { token, csrfToken } (matches fixtures/admin.ts).
async function adminToken() {
    const res = await globalThis.fetch(`${TRAILBASE}/api/auth/v1/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: ADMIN.email, password: ADMIN.password }),
    });
    if (!res.ok) throw new Error(`admin login failed: ${res.status}`);
    const data = await res.json();
    if (!data.csrf_token) throw new Error("admin login missing csrf_token");
    return { token: data.auth_token, csrfToken: data.csrf_token };
}

/// Create a verified test user via the admin API (fixtures/admin.ts flow).
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
    if (!res.ok) {
        throw new Error(`create user failed: ${res.status} ${await res.text()}`);
    }
    const data = await res.json();
    console.log(`[admin] Created ${email} (${data.id})`);
    return data.id;
}

const browser = await chromium.launch();
const context = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    locale: "ru-RU",
});
await context.addInitScript(() => {
    // Runs in the page (browser globals via globalThis — the eslint node
    // profile for scripts/*.mjs has no `window`).
    const g = globalThis;
    if (g.location?.origin === "http://localhost:1420") {
        g.localStorage?.setItem("origa_resource_download_consented", "true");
    }
});
const page = await context.newPage();

const email = `${runId}@example.com`;
const password = "Passw0rd!train";
await createUser(email, password);

// Login through the UI.
await page.goto(`${BASE}/login`, { waitUntil: "domcontentloaded" });
const toggle = page.getByTestId("login-password-toggle");
await toggle.waitFor({ state: "visible", timeout: 60_000 });
await toggle.click();
await page.getByTestId("email-input").waitFor({ timeout: 30_000 });
await page.getByTestId("email-input").fill(email);
await page.getByTestId("password-input").fill(password);
await page.getByTestId("login-form").locator("button[type=submit]").click();
// Terminal route after login: /home (onboarded) or /onboarding (fresh user).
await page.waitForURL(/\/(home|onboarding)$/, { timeout: 90_000 });

const ONBOARDED_MODE = process.env.SHOTS_MODE === "onboarded";

if (ONBOARDED_MODE) {
    // Full onboarding at N5 (skip apps): the phrases catalogue becomes
    // available, which the plain skip flow does not provide. Mirrors
    // completeOnboardingToScoring + mark-all + finish.
    await page.goto(`${BASE}/`);
    await page.waitForURL(/\/onboarding$/, { timeout: 30_000 });
    await page
        .getByTestId("onboarding-spinner")
        .waitFor({ state: "hidden", timeout: 30_000 })
        .catch(() => {});
    await page.getByTestId("onboarding-next").click();
    await page.getByTestId("onboarding-load-step").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByTestId("onboarding-next").click();
    await page.getByTestId("onboarding-jlpt-step").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByTestId("jlpt-option-n5").click();
    await page
        .getByTestId("jlpt-option-n5")
        .waitFor({ state: "visible", timeout: 5_000 });
    await page.getByTestId("onboarding-next").click();
    await page.getByTestId("onboarding-apps-step").waitFor({ state: "visible", timeout: 10_000 });
    await page.getByTestId("onboarding-next").click();
    await page
        .getByTestId("onboarding-progress-step")
        .waitFor({ state: "visible", timeout: 10_000 });
    await page.getByTestId("onboarding-next").click();
    await page
        .getByTestId("onboarding-summary-step")
        .waitFor({ state: "visible", timeout: 10_000 });
    // Summary → Import → Scoring (see completeOnboardingToScoring).
    await page.getByTestId("onboarding-import").click();
    // Import runs, then the scoring step appears (may take a while).
    await page
        .getByTestId("onboarding-scoring-step")
        .waitFor({ state: "visible", timeout: 240_000 });
    await page.getByTestId("scoring-step-hint").waitFor({ state: "visible", timeout: 60_000 });
    await page.getByTestId("onboarding-mark-all-known").click();
    await page.getByTestId("onboarding-confirm-ok").click();
    await page
        .getByTestId("scoring-step-complete")
        .waitFor({ state: "visible", timeout: 240_000 });
    await page.getByTestId("onboarding-finish").click();
    await page.waitForURL(/\/home$/, { timeout: 240_000 });
} else {
    // Skip onboarding if it appears.
    await page
        .getByTestId("onboarding-spinner")
        .waitFor({ state: "hidden", timeout: 30_000 })
        .catch(() => {});
    const skip = page.getByTestId("onboarding-skip");
    if (await skip.isVisible({ timeout: 15_000 }).catch(() => false)) {
        await skip.click();
        const confirm = page.getByTestId("onboarding-confirm-ok");
        await confirm
            .waitFor({ state: "visible", timeout: 10_000 })
            .then(() => confirm.click())
            .catch(() => {});
    }
    await page.waitForURL(/\/home$/, { timeout: 60_000 });
}

if (ONBOARDED_MODE) {
    // 4) Token popup on the phrases catalogue.
    await page.goto(`${BASE}/phrases`);
    await page
        .getByTestId("phrases-card-item")
        .first()
        .waitFor({ state: "visible", timeout: 30_000 });
    await page.waitForTimeout(3000);
    const token = page.locator(".token-word .token-surface").first();
    await token.waitFor({ state: "visible", timeout: 30_000 });
    await token.click();
    await page
        .getByTestId("token-popup-report")
        .waitFor({ state: "visible", timeout: 10_000 });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(OUT, "04-token-popup-report.png") });
    await browser.close();
    console.log("screenshots in", OUT);
    process.exit(0);
}

// Add a word from text (same flow as BDD).
await page.goto(`${BASE}/words`);
await page.getByTestId("words-page").waitFor({ timeout: 30_000 });
await page.getByTestId("words-add-btn").click();
await page.getByTestId("words-drawer-textarea").fill("私は本を読みます");
await page.getByTestId("words-drawer-analyze-btn").click();
await page.getByText(/Найдено/).waitFor({ timeout: 15_000 });
const items = page.getByTestId("words-drawer-item");
await items.first().waitFor({ state: "visible", timeout: 10_000 });
// analyze_text() pre-selects everything; deselect all, then pick the first
// (words.page.ts selectFirstWord flow, incl. disabled items).
const count = await items.count();
for (let i = 0; i < count; i++) {
    const item = items.nth(i);
    const checkbox = item.locator('input[type="checkbox"]');
    if (await checkbox.isChecked().catch(() => false)) {
        const isDisabled = await item
            .evaluate((el) => el.classList.contains("cursor-not-allowed"))
            .catch(() => false);
        if (isDisabled) continue;
        await item.click();
        await page.waitForTimeout(150);
    }
}
await items.first().click();
await page.waitForTimeout(200);
await page.getByTestId("words-drawer-add-btn").click();
await page.getByTestId("words-grid").waitFor({ timeout: 20_000 });

// Start a lesson.
await page.goto(`${BASE}/lesson`);
await page.getByTestId("lesson-header").waitFor({ state: "visible", timeout: 60_000 });

// 1) Lesson header with the report button (question phase: disabled).
await page.getByTestId("lesson-report-btn").waitFor({ state: "visible", timeout: 60_000 });
await page.screenshot({ path: join(OUT, "01-lesson-header-report-disabled.png") });

// Walk the acquaintance presentation to the training reveal
// (runAcquaintancePresentation flow from helpers/lesson.ts).
await page.getByTestId("acquaintance-view").waitFor({ state: "visible", timeout: 30_000 });
const next = page.getByTestId("acquaintance-next-btn");
for (let i = 0; i < 20; i++) {
    await next.click({ timeout: 3_000 }).catch(() => null);
    const training = await page
        .getByTestId("acquaintance-training")
        .waitFor({ state: "visible", timeout: 1_000 })
        .then(() => true)
        .catch(() => false);
    if (training) break;
}
await page
    .getByTestId("acquaintance-training")
    .waitFor({ state: "visible", timeout: 15_000 });
const reveal = page.getByTestId("acquaintance-reveal-btn");
await reveal.waitFor({ state: "visible", timeout: 30_000 });
await reveal.click();
await page.waitForTimeout(500);

// 2) Open the feedback modal from the lesson header (enabled after reveal).
await page.getByTestId("lesson-report-btn").click();
await page.getByTestId("feedback-modal").waitFor({ state: "visible", timeout: 10_000 });
await page.screenshot({ path: join(OUT, "02-feedback-modal-subject.png") });

// Fill + auto-context disclosure open.
await page.getByTestId("feedback-message-input").fill("Чтение не соответствует карточке");
await page.screenshot({ path: join(OUT, "03-feedback-modal-filled.png") });

// 3) Token popup with the "wrong translation?" line: the phrases page
// renders TranslatorText (token popups) on every phrase card.
await page.keyboard.press("Escape");
await page.waitForTimeout(400);
await page.goto(`${BASE}/phrases`);
await page.getByTestId("phrases-card-item").first().waitFor({ state: "visible", timeout: 30_000 });
await page.waitForTimeout(3000);
await page.screenshot({ path: join(OUT, "diag-phrases.png") });
const token = page.locator(".token-word .token-surface").first();
await token.waitFor({ state: "visible", timeout: 30_000 });
await token.click();
await page.waitForTimeout(400);
await page
    .getByTestId("token-popup-report")
    .waitFor({ state: "visible", timeout: 10_000 });
await page.screenshot({ path: join(OUT, "04-token-popup-report.png") });

await browser.close();
console.log("screenshots in", OUT);
