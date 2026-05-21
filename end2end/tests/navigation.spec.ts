import { testWithFreshUser } from "../fixtures";
import { skipOnboarding } from "../helpers/navigation";

async function waitForSidebar(page: import("@playwright/test").Page): Promise<void> {
    // Sidebar appears after current_user is loaded (async effect).
    // Retry with increasing timeouts to handle slow backend responses.
    for (let i = 0; i < 6; i++) {
        const sidebar = page.getByTestId("sidebar");
        if (await sidebar.isVisible().catch(() => false)) {
            return;
        }
        // Small wait + reload user data trigger
        await page.waitForTimeout(2000);
        // Navigate to trigger re-evaluation of sidebar_visible
        await page.goto("/home", { waitUntil: "domcontentloaded" });
        await page.getByTestId("home-page").waitFor({ state: "visible", timeout: 15_000 }).catch(() => {});
    }
    // Final attempt with long timeout
    await page.getByTestId("sidebar").waitFor({ state: "visible", timeout: 30_000 });
}

testWithFreshUser.describe("Navigation", () => {
    testWithFreshUser("should navigate between all pages via sidebar", async ({ page }) => {
        await skipOnboarding(page);
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
        await waitForSidebar(page);

        // Home → Words
        await page.getByTestId("sidebar-tab-words").click();
        await page.waitForURL(/\/words/, { timeout: 10_000 });

        // Words → Grammar
        await page.getByTestId("sidebar-tab-grammar").click();
        await page.waitForURL(/\/grammar/, { timeout: 10_000 });

        // Grammar → Kanji
        await page.getByTestId("sidebar-tab-kanji").click();
        await page.waitForURL(/\/kanji/, { timeout: 10_000 });

        // Kanji → Phrases
        await page.getByTestId("sidebar-tab-phrases").click();
        await page.waitForURL(/\/phrases/, { timeout: 10_000 });

        // Phrases → Profile
        await page.getByTestId("sidebar-tab-profile").click();
        await page.waitForURL(/\/profile/, { timeout: 10_000 });

        // Profile → Home
        await page.getByTestId("sidebar-tab-home").click();
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
    });
});
