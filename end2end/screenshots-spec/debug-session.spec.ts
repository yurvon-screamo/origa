import { test, expect } from "@playwright/test";
import { getAdminToken, createTestUser } from "../fixtures/admin";
import { generateUniqueEmail, DEFAULT_TEST_PASSWORD } from "../helpers/auth";
import { completeOnboardingToScoring } from "../helpers/onboarding";
import { LoginPage } from "../pages/login.page";

test("debug full flow", async ({ page }) => {
    test.setTimeout(600_000);
    const { token, csrfToken } = await getAdminToken();
    const email = generateUniqueEmail();
    await createTestUser(token, csrfToken, email, DEFAULT_TEST_PASSWORD);
    await page.context().addInitScript(() => {
        if (window.location.origin === "http://localhost:1420") {
            window.localStorage.setItem("origa_resource_download_consented", "true");
        }
    });
    const login = new LoginPage(page);
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await page.evaluate(() => localStorage.clear());
    await login.expandPasswordForm();
    await login.fillEmail(email);
    await login.fillPassword(DEFAULT_TEST_PASSWORD);
    await login.submit();
    await page.waitForURL(/\/(home|onboarding)$/, { timeout: 180_000 });
    if (page.url().includes("onboarding")) {
        await completeOnboardingToScoring(page, { level: "N4" });
        const markAll = page.getByTestId("onboarding-mark-all-known");
        if (await markAll.isVisible({ timeout: 10_000 }).catch(() => false)) {
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
    console.log("FLOW: home reached");

    const consoleAll: string[] = [];
    page.on("console", (msg) => {
        consoleAll.push(msg.text().replace(/\x1b\[[0-9;]*m/g, "").slice(0, 180));
    });

    // SPA transition instead of full reload: pushState + popstate keeps
    // the authenticated session alive (full bootstrap hangs in merge).
    // ASCII word first: isolates encoded-segment routing from auth gating.
    await page.evaluate(() => {
        window.history.pushState({}, "", "/words/abc");
        window.dispatchEvent(new PopStateEvent("popstate"));
    });
    await page.waitForTimeout(8_000);
    console.log(
        "S0-ASCII:",
        JSON.stringify(
            await page.evaluate(() => ({
                url: location.href,
                mainHead: (document.querySelector("main")?.innerHTML || "NO MAIN").slice(0, 120),
            })),
        ),
    );
    // Then the kana word with a LONG poll: if it eventually renders, it is
    // slow session work; if login forever — route fallback.
    await page.evaluate(() => {
        window.history.pushState({}, "", "/words/%E3%81%82%E3%81%95");
        window.dispatchEvent(new PopStateEvent("popstate"));
    });
    for (let i = 1; i <= 12; i++) {
        await page.waitForTimeout(5_000);
        const head = await page.evaluate(
            () => (document.querySelector("main")?.innerHTML || "NO MAIN").slice(0, 90),
        );
        console.log(`POLL ${i * 5}s:`, JSON.stringify(head));
        if (head.includes("word-detail") || head.includes("login-page") === false) break;
    }
    console.log(
        "S1:",
        JSON.stringify(
            await page.evaluate(() => ({
                url: location.href,
                mainHead: (document.querySelector("main")?.innerHTML || "NO MAIN").slice(0, 120),
            })),
        ),
    );
    console.log(
        "S2:",
        JSON.stringify(
            await page.evaluate(() => ({
                url: location.href,
                mainHead: (document.querySelector("main")?.innerHTML || "NO MAIN").slice(0, 120),
            })),
        ),
    );
    console.log("CONSOLE-TAIL:", JSON.stringify(consoleAll.slice(-14), null, 1));
});
