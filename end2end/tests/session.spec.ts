import { testWithFreshUser } from "../fixtures";
import { skipOnboarding } from "../helpers/navigation";

testWithFreshUser.describe("Session Persistence", () => {
    testWithFreshUser("should persist session after page reload", async ({ page }) => {
        await skipOnboarding(page);
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
        await page.getByTestId("home-page").waitFor({ state: "visible", timeout: 15_000 });

        // Reload page
        await page.reload();
        await page.waitForURL(/\/home$/, { timeout: 10_000 });

        // Should still be authenticated
        await page.getByTestId("home-page").waitFor({ state: "visible", timeout: 30_000 });
    });

    testWithFreshUser("should persist session after navigating away and back", async ({ page }) => {
        await skipOnboarding(page);
        await page.waitForURL(/\/home$/, { timeout: 10_000 });

        // Wait for sidebar (means current_user is loaded)
        for (let i = 0; i < 6; i++) {
            const sidebar = page.getByTestId("sidebar");
            if (await sidebar.isVisible().catch(() => false)) {
                break;
            }
            await page.waitForTimeout(2000);
            await page.goto("/home", { waitUntil: "domcontentloaded" });
            await page.getByTestId("home-page").waitFor({ state: "visible", timeout: 15_000 }).catch(() => {});
        }
        await page.getByTestId("sidebar").waitFor({ state: "visible", timeout: 30_000 });

        // Navigate to words via sidebar
        await page.getByTestId("sidebar-tab-words").click();
        await page.waitForURL(/\/words/, { timeout: 10_000 });

        // Navigate back to home
        await page.getByTestId("sidebar-tab-home").click();
        await page.waitForURL(/\/home$/, { timeout: 10_000 });

        // Should still be authenticated
        await page.getByTestId("home-page").waitFor({ state: "visible", timeout: 10_000 });
    });
});
