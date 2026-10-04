/**
 * Screenshots for issue #528 (word examples): create a test user, log in,
 * open the word detail page for a translated word, capture hero + examples.
 */
import { test, expect } from "@playwright/test";
import { LoginPage } from "../pages/login.page";
import { getAdminToken, createTestUser } from "../fixtures/admin";
import { generateUniqueEmail, DEFAULT_TEST_PASSWORD } from "../helpers/auth";

const WORD = process.env.SHOT_WORD || "あさ";

test("word detail screenshots", async ({ page }) => {
    test.setTimeout(180_000);

    const { token, csrfToken } = await getAdminToken();
    const email = generateUniqueEmail();
    await createTestUser(token, csrfToken, email, DEFAULT_TEST_PASSWORD);

    const login = new LoginPage(page);
    await page.goto("/");
    await login.expandPasswordForm();
    await login.fillEmail(email);
    await login.fillPassword(DEFAULT_TEST_PASSWORD);
    await login.submit();
    // CSR login stays on "/" and renders the app shell — wait for the
    // sidebar navigation instead of a URL change.
    const wordsNav = page.getByRole("link", { name: "Words" });
    await wordsNav.waitFor({ state: "visible", timeout: 120_000 });

    // Consent gate: set the persisted approval flag and reload so the
    // auto-start effect launches the resource fetch itself (a programmatic
    // DOM-click on the consent button is not delivered to the Leptos
    // handler). Resources already sit in the browser Cache API from earlier
    // runs, so the fetch is near-instant.
    await page.evaluate(() =>
        localStorage.setItem("origa_resource_download_consented", "true"),
    );
    await page.reload();
    await wordsNav.waitFor({ state: "visible", timeout: 600_000 });
    await wordsNav.waitFor({ state: "visible", timeout: 120_000 });
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
    console.log("FAILED-REQUESTS:", JSON.stringify(failedRequests.slice(0, 15), null, 1));
    const mainHtml = await page
        .locator("main")
        .innerHTML()
        .catch(() => "<no main>");
    console.log("MAIN-HTML:", mainHtml.slice(0, 600));
    await page.screenshot({ path: "screenshots/word-detail-diagnostic.png" });
    await expect(page.getByTestId("word-detail-word")).toBeVisible({ timeout: 60_000 });
    await page.waitForTimeout(3000); // examples chunk fetch
    await page.screenshot({ path: "screenshots/word-detail-full.png", fullPage: true });
    await page.screenshot({ path: "screenshots/word-detail-viewport.png" });
});
