/**
 * Screenshots for issue #749 (home insight panels): create a test user, log
 * in, skip onboarding, capture the redesigned home — desktop and mobile
 * viewports. Output: .pr-screenshots/ at the repo root.
 */
import { test, expect } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { LoginPage } from "../pages/login.page";
import { getAdminToken, createTestUser } from "../fixtures/admin";
import { skipOnboarding } from "../helpers/navigation";
import { generateUniqueEmail, DEFAULT_TEST_PASSWORD } from "../helpers/auth";

const OUTPUT_DIR = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "../../.pr-screenshots",
);

async function loginFreshUser(page: import("@playwright/test").Page): Promise<void> {
    const { token, csrfToken } = await getAdminToken();
    const email = generateUniqueEmail();
    await createTestUser(token, csrfToken, email, DEFAULT_TEST_PASSWORD);

    // Pre-approve the one-time resource download before any app script
    // runs — otherwise the consent overlay intercepts every click.
    await page.context().addInitScript(() => {
        if (window.location.origin === "http://localhost:1420") {
            window.localStorage.setItem("origa_resource_download_consented", "true");
        }
    });

    // Cold-start flake: retry the whole login like helpers/auth uiLogin.
    // NOTE: no localStorage.clear() here — it would race the init script
    // that pre-approves the resource-download consent (the word-examples
    // spec hit this exact ordering). The Playwright context is fresh per
    // test anyway.
    const login = new LoginPage(page);
    let loggedIn = false;
    for (let attempt = 1; attempt <= 3 && !loggedIn; attempt++) {
        await page.goto("/", { waitUntil: "domcontentloaded" });
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

    // Belt and suspenders: if the consent gate still appeared (flag lost),
    // click through it — the ~230 MB payload comes from the local CDN mirror.
    const downloadButton = page.getByRole("button", { name: "Download" });
    if (await downloadButton.isVisible().catch(() => false)) {
        await downloadButton.click();
        await downloadButton.waitFor({ state: "hidden", timeout: 300_000 });
    }

    if (page.url().includes("onboarding")) {
        await skipOnboarding(page);
    }
    await page.waitForURL(/\/home$/, { timeout: 60_000 });
}

async function captureHome(page: import("@playwright/test").Page, name: string): Promise<void> {
    // The insights section renders behind the loading gate — wait for the
    // week-verdict panel before shooting.
    await expect(
        page.getByTestId("home-insights-week-verdict"),
    ).toBeVisible({ timeout: 120_000 });
    // Give fonts/images a beat to settle, then capture the viewport.
    await page.waitForTimeout(1_500);
    await page.screenshot({ path: path.join(OUTPUT_DIR, name) });
}

test("home insights screenshots", async ({ page }) => {
    test.setTimeout(600_000);

    // Desktop 1280x800
    await page.setViewportSize({ width: 1280, height: 800 });
    await loginFreshUser(page);
    await captureHome(page, "home-insights-desktop.png");

    // Mobile 390x844 — fresh user per viewport keeps the states identical.
    await page.setViewportSize({ width: 390, height: 844 });
    await page.reload({ waitUntil: "domcontentloaded" });
    await captureHome(page, "home-insights-mobile.png");
});
