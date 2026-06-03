import { testWithFreshUser } from "../fixtures";
import { skipOnboarding } from "../helpers/navigation";

testWithFreshUser.describe("Navigation", () => {
    testWithFreshUser("sidebar should appear after onboarding without page reload", async ({ page }) => {
        await skipOnboarding(page);
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
        await page.getByTestId("sidebar").waitFor({ state: "visible", timeout: 15_000 });
    });

    testWithFreshUser("should navigate between all pages via sidebar", async ({ page }) => {
        await skipOnboarding(page);
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
        await page.getByTestId("sidebar").waitFor({ state: "visible", timeout: 15_000 });

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
