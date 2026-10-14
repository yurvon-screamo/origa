/**
 * PR screenshots for the JFT-Basic Can-do tracking feature (issue #760).
 *
 * Drives a fresh user through onboarding with ONLY the Irodori app selected
 * (Nyuumon lessons 1-9), which makes the home page render the JFT-Basic
 * readiness card, then captures:
 *   1. home with the readiness card (three Can-do bands)
 *   2. sets with the JFT-Basic badge on Irodori cards
 * in desktop (1280x800) and mobile (390x844) viewports.
 *
 * Usage: node scripts/pr-screenshots-jft.mjs
 * (expects the e2e web servers: app on :1420 serving a build pointed at
 * localhost, CDN on :8080 with well_known_set/irodori_cando.json,
 * TrailBase on :4000 — same stack as scripts/pr-screenshots.mjs)
 */
import { chromium } from "@playwright/test";
import { mkdirSync } from "fs";
import { join } from "path";

const OUT = "/tmp/opencode/pr-screens-jft";
mkdirSync(OUT, { recursive: true });

const BASE = "http://localhost:1420";
const ADMIN = { email: "admin@localhost", password: "secret" };
const TRAILBASE = "http://127.0.0.1:4000";

const runId = `jft-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;

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
    console.log(`[admin] Created ${email}`);
}

const browser = await chromium.launch();
const context = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    locale: "ru-RU",
});
await context.addInitScript(() => {
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
await page.waitForURL(/\/onboarding$/, { timeout: 90_000 });

// Onboarding: intro → load → JLPT (none) → apps (Irodori only) → progress
// (Nyuumon lesson 9) → summary → import → scoring → mark known → finish.
await page.getByTestId("onboarding-spinner").waitFor({ state: "hidden", timeout: 60_000 });
await page.getByTestId("onboarding-next").click();
await page.getByTestId("onboarding-load-step").waitFor({ state: "visible", timeout: 10_000 });
await page.getByTestId("onboarding-next").click();
await page.getByTestId("onboarding-jlpt-step").waitFor({ state: "visible", timeout: 10_000 });
// No JLPT level selected: the import consists of the Irodori sets only.
await page.getByTestId("onboarding-next").click();
await page.getByTestId("onboarding-apps-step").waitFor({ state: "visible", timeout: 10_000 });
await page.getByTestId("apps-step-app-Irodori-checkbox").click();
await page.getByTestId("onboarding-next").click();
await page.getByTestId("onboarding-progress-step").waitFor({ state: "visible", timeout: 10_000 });
await page.getByTestId("irodori-book-dropdown").click();
await page.getByTestId("irodori-book-dropdown-option-nyuumon").click();
await page.getByTestId("irodori-lesson-dropdown").click();
await page.getByTestId("irodori-lesson-dropdown-option-lesson_9").click();
await page.getByTestId("onboarding-next").click();
await page.getByTestId("onboarding-summary-step").waitFor({ state: "visible", timeout: 10_000 });
await page.getByTestId("onboarding-import").click();
await page.getByTestId("onboarding-scoring-step").waitFor({ state: "visible", timeout: 240_000 });
await page.getByTestId("scoring-step-hint").waitFor({ state: "visible", timeout: 60_000 });
await page.getByTestId("onboarding-mark-all-known").click();
await page.getByTestId("onboarding-confirm-ok").click();
await page.getByTestId("scoring-step-complete").waitFor({ state: "visible", timeout: 240_000 });
await page.getByTestId("onboarding-finish").click();
await page.waitForURL(/\/home$/, { timeout: 240_000 });

// The readiness card needs the CDN catalog fetch; give it a generous window.
await page
    .getByTestId("home-jft-readiness")
    .waitFor({ state: "visible", timeout: 60_000 });
await page
    .getByTestId("home-jft-band-A2")
    .waitFor({ state: "visible", timeout: 10_000 });

console.log("[shots] Desktop home…");
await page.screenshot({ path: join(OUT, "home-jft-desktop.png"), fullPage: false });

// Sets page: filter to Irodori so the badge cards fill the viewport.
await page.goto(`${BASE}/sets`, { waitUntil: "domcontentloaded" });
await page.getByTestId("sets-search-input").waitFor({ state: "visible", timeout: 60_000 });
await page.getByTestId("sets-search-input").fill("Irodori");
await page.getByTestId("sets-loading").waitFor({ state: "hidden", timeout: 60_000 });
await page.getByTestId("sets-card-jft-badge").first().waitFor({ state: "visible", timeout: 60_000 });
console.log("[shots] Desktop sets…");
await page.screenshot({ path: join(OUT, "sets-jft-badge-desktop.png"), fullPage: false });

// Mobile pass: same two surfaces at 390x844.
await page.setViewportSize({ width: 390, height: 844 });
await page.goto(`${BASE}/`, { waitUntil: "domcontentloaded" });
await page.getByTestId("home-jft-readiness").waitFor({ state: "visible", timeout: 60_000 });
console.log("[shots] Mobile home…");
await page.screenshot({ path: join(OUT, "home-jft-mobile.png"), fullPage: false });

await page.goto(`${BASE}/sets`, { waitUntil: "domcontentloaded" });
await page.getByTestId("sets-search-input").waitFor({ state: "visible", timeout: 60_000 });
await page.getByTestId("sets-search-input").fill("Irodori");
await page.getByTestId("sets-card-jft-badge").first().waitFor({ state: "visible", timeout: 60_000 });
console.log("[shots] Mobile sets…");
await page.screenshot({ path: join(OUT, "sets-jft-badge-mobile.png"), fullPage: false });

await browser.close();
console.log(`[shots] Done → ${OUT}`);
