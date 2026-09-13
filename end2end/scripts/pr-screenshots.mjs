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

async function adminToken() {
    const res = await globalThis.fetch(`${TRAILBASE}/api/auth/v1/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: ADMIN.email, password: ADMIN.password }),
    });
    const csrf = res.headers.get("set-cookie")?.match(/csrf_token[^;]*/)?.[0];
    const login = await res.json();
    const token = login.auth_token ?? login.token;
    return { token, csrf };
}

async function createUser(email, password) {
    const { token, csrf } = await adminToken();
    await globalThis.fetch(`${TRAILBASE}/api/auth/v1/admin/users`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
            ...(csrf ? { Cookie: csrf } : {}),
        },
        body: JSON.stringify({ email, password }),
    });
}

const browser = await chromium.launch();
const context = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    locale: "ru-RU",
});
const page = await context.newPage();

const email = `${runId}@example.com`;
const password = "Passw0rd!train";
await createUser(email, password);

// Login through the UI.
await page.goto(`${BASE}/login`, { waitUntil: "domcontentloaded" });
await page.getByTestId("email-input").waitFor({ timeout: 60_000 });
await page.getByTestId("email-input").fill(email);
await page.getByTestId("password-input").fill(password);
await page.getByTestId("login-form").locator("button[type=submit]").click();

// Skip onboarding if it appears.
const skip = page.getByTestId("onboarding-skip");
if (await skip.isVisible({ timeout: 20_000 }).catch(() => false)) {
    await skip.click();
    const confirm = page.getByTestId("onboarding-confirm-ok");
    if (await confirm.isVisible().catch(() => false)) await confirm.click();
}
await page.waitForURL(/\/home$/, { timeout: 60_000 });

// Add a word from text (same flow as BDD).
await page.goto(`${BASE}/words`);
await page.getByTestId("words-page").waitFor({ timeout: 30_000 });
await page.getByTestId("words-add-btn").click();
await page.getByTestId("words-drawer-textarea").fill("私は本を読みます");
await page.getByTestId("words-drawer-analyze-btn").click();
await page.getByText(/Найдено/).waitFor({ timeout: 15_000 });
const items = page.getByTestId("words-drawer-item");
const count = await items.count();
for (let i = 0; i < count; i++) {
    const checked = await items.nth(i).locator("input[type=checkbox]").isChecked().catch(() => true);
    if (checked) await items.nth(i).click().catch(() => null);
}
await items.first().click().catch(() => null);
await page.getByTestId("words-drawer-add-btn").click();
await page.getByTestId("words-grid").waitFor({ timeout: 15_000 });

// Start a lesson.
await page.goto(`${BASE}/lesson`);
await page.getByTestId("lesson-page, [data-testid=lesson-header]").first().waitFor({ timeout: 60_000 });

// 1) Lesson header with the report button (question phase: disabled).
await page.getByTestId("lesson-report-btn").waitFor({ state: "visible", timeout: 60_000 });
await page.screenshot({ path: join(OUT, "01-lesson-header-report-disabled.png") });

// Walk the acquaintance presentation to the training reveal.
const next = page.getByTestId("acquaintance-next-btn");
for (let i = 0; i < 25; i++) {
    if (await page.getByTestId("acquaintance-training").isVisible().catch(() => false)) break;
    await next.click({ timeout: 2_000 }).catch(() => null);
}
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
await page.getByTestId("feedback-auto-toggle").click();
await page.waitForTimeout(300);
await page.screenshot({ path: join(OUT, "03-feedback-modal-filled.png") });

// 3) Token popup: find a translator token on the lesson card and click it.
await page.keyboard.press("Escape");
await page.waitForTimeout(400);
const token = page.locator(".token-word .token-surface").first();
if (await token.isVisible().catch(() => false)) {
    await token.click();
    await page.waitForTimeout(400);
    const reportLine = page.getByTestId("token-popup-report");
    if (await reportLine.isVisible().catch(() => false)) {
        await page.screenshot({ path: join(OUT, "04-token-popup-report.png") });
    } else {
        console.warn("token popup rendered without the report line");
    }
} else {
    console.warn("no token-word found on the current card; skipping popup shot");
}

await browser.close();
console.log("screenshots in", OUT);
