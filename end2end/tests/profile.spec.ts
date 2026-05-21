import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser, testWithUniqueUser } from "../fixtures";
import { skipOnboarding } from "../helpers/navigation";
import { LoginPage, ProfilePage } from "../pages";

async function setupProfilePage(page: Page): Promise<ProfilePage> {
    await skipOnboarding(page);
    const profilePage = new ProfilePage(page);
    await profilePage.goto();
    await profilePage.expectProfileVisible();
    return profilePage;
}

testWithFreshUser.describe("Profile Page", () => {
    testWithFreshUser("should display profile page", async ({ page }) => {
        const profilePage = await setupProfilePage(page);

        await expect(profilePage.profilePage).toBeVisible();
        await expect(profilePage.profileContent).toBeVisible();
    });

    testWithFreshUser("should navigate to home via sidebar", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        await page.getByTestId("sidebar-tab-home").click();
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
    });

    testWithFreshUser("should display all language options", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        await expect(profilePage.langEnglish).toBeVisible();
        await expect(profilePage.langRussian).toBeVisible();
    });

    testWithFreshUser("should display all daily load options", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        await expect(profilePage.loadLight).toBeVisible();
        await expect(profilePage.loadMedium).toBeVisible();
        await expect(profilePage.loadHard).toBeVisible();
        await expect(profilePage.loadHeavy).toBeVisible();
    });

    testWithFreshUser("should switch language selection", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("english");
        await expect(profilePage.langEnglish).toHaveAttribute("aria-current", "true");
        await expect(profilePage.langRussian).toHaveAttribute("aria-current", "false");
        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("russian");
        await expect(profilePage.langRussian).toHaveAttribute("aria-current", "true");
        await expect(profilePage.langEnglish).toHaveAttribute("aria-current", "false");
    });

    testWithFreshUser("should switch daily load selection", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        await profilePage.selectDailyLoad("hard");
        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);
        await expect(profilePage.loadLight).not.toHaveClass(/btn-olive/);
        await profilePage.selectDailyLoad("light");
        await expect(profilePage.loadLight).toHaveClass(/btn-olive/);
        await expect(profilePage.loadHard).not.toHaveClass(/btn-olive/);
    });

    testWithFreshUser("should show delete confirmation and cancel", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        await profilePage.deleteAccount();
        await expect(profilePage.confirmDeleteBtn).toBeVisible({ timeout: 5000 });
        await expect(profilePage.cancelDeleteBtn).toBeVisible();
        await profilePage.cancelDelete();
        await expect(profilePage.confirmDeleteBtn).not.toBeVisible({ timeout: 5000 });
    });

    testWithFreshUser("should display save and logout buttons", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        await expect(page.getByTestId("profile-save-btn")).toBeVisible();
        await expect(page.getByTestId("profile-logout-btn")).toBeVisible();
    });

    testWithFreshUser("should display settings card with app info", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        await expect(profilePage.profileSettings).toBeVisible();
    });

    testWithFreshUser("should persist language after save", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);
        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("english");
        await expect(profilePage.langEnglish).toHaveAttribute("aria-current", "true");
        await profilePage.saveProfile();
        await profilePage.waitForSaveComplete();
        await page.reload();
        await profilePage.waitForLoad();
        await profilePage.expectProfileVisible();
        await expect(profilePage.langEnglish).toHaveAttribute("aria-current", "true");
    });

    testWithFreshUser("should persist daily load after save", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);
        await profilePage.selectDailyLoad("hard");
        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);
        await profilePage.saveProfile();
        await profilePage.waitForSaveComplete();
        await page.reload();
        await profilePage.waitForLoad();
        await profilePage.expectProfileVisible();
        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);
    });

    testWithFreshUser("should persist daily load after save and navigation to home", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);

        await profilePage.selectDailyLoad("hard");
        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);

        await profilePage.saveProfile();
        await profilePage.waitForSaveComplete();

        await profilePage.navigateToHomeAndBack();

        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);
    });

    testWithFreshUser("should persist language after save and navigation to home", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);

        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("english");
        await expect(profilePage.langEnglish).toHaveAttribute("aria-current", "true");

        await profilePage.saveProfile();
        await profilePage.waitForSaveComplete();

        await profilePage.navigateToHomeAndBack();

        await expect(profilePage.langEnglish).toHaveAttribute("aria-current", "true");
    });

    // Regression: catches merge_current_user() data loss bug
    testWithFreshUser("should persist daily load after merge triggered by home sync", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);

        await profilePage.selectDailyLoad("heavy");
        await expect(profilePage.loadHeavy).toHaveClass(/btn-olive/);

        await profilePage.saveProfile();
        await profilePage.waitForSaveComplete();

        await profilePage.navigateToHomeAndWaitForSync();

        await profilePage.expectProfileVisible();

        await expect(profilePage.loadHeavy).toHaveClass(/btn-olive/);
    });

    testWithFreshUser("should persist both daily load and language after save", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);

        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("english");
        await profilePage.selectDailyLoad("hard");

        await profilePage.saveProfile();
        await profilePage.waitForSaveComplete();

        await profilePage.navigateToHomeAndBack();

        await expect(profilePage.langEnglish).toHaveAttribute("aria-current", "true");
        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);
    });

    testWithFreshUser("should logout and redirect to login page", async ({ page }) => {
        const profilePage = await setupProfilePage(page);

        await page.getByTestId("profile-logout-btn").click();

        const loginPage = new LoginPage(page);
        await expect(loginPage.loginPage).toBeVisible({ timeout: 10_000 });
        await expect(page).toHaveURL(/\/login/);
    });
});

testWithUniqueUser.describe("Profile - Account Deletion", () => {
    testWithUniqueUser("should delete account and redirect to login", async ({ page }) => {
        await skipOnboarding(page);

        const profilePage = new ProfilePage(page);
        await profilePage.goto();
        await profilePage.expectProfileVisible();

        await profilePage.deleteAccount();
        await expect(profilePage.confirmDeleteBtn).toBeVisible({ timeout: 5000 });

        await profilePage.confirmDelete();

        // Account deletion is async — wait for redirect with extended timeout
        await page.waitForURL(/\/login/, { timeout: 15_000 });
    });
});
