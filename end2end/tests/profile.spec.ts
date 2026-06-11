import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser, testWithUniqueUser, DEFAULT_TEST_PASSWORD } from "../fixtures";
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
        await setupProfilePage(page);
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
        await expect(profilePage.langEnglish).toHaveClass(/border-\[var\(--fg-black\)\]/);
        await expect(profilePage.langRussian).toHaveClass(/border-transparent/);
        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("russian");
        await expect(profilePage.langRussian).toHaveClass(/border-\[var\(--fg-black\)\]/);
        await expect(profilePage.langEnglish).toHaveClass(/border-transparent/);
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

    testWithFreshUser("should display logout button in danger zone", async ({ page }) => {
        await setupProfilePage(page);
        await expect(page.getByTestId("profile-logout-btn")).toBeVisible();
    });

    testWithFreshUser("should display settings card with app info", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        await expect(profilePage.profileSettings).toBeVisible();
    });

    testWithFreshUser("should persist language after auto-save", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);
        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("english");
        await expect(profilePage.langEnglish).toHaveClass(/border-\[var\(--fg-black\)\]/);
        await profilePage.waitForAutoSave();
        await page.reload();
        await profilePage.waitForLoad();
        await profilePage.expectProfileVisible();
        await expect(profilePage.langEnglish).toHaveClass(/border-\[var\(--fg-black\)\]/);
    });

    testWithFreshUser("should persist daily load after auto-save", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);
        await profilePage.selectDailyLoad("hard");
        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);
        await profilePage.waitForAutoSave();
        await page.reload();
        await profilePage.waitForLoad();
        await profilePage.expectProfileVisible();
        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);
    });

    testWithFreshUser("should persist daily load after auto-save and navigation to home", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);

        await profilePage.selectDailyLoad("hard");
        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);

        await profilePage.waitForAutoSave();

        await profilePage.navigateToHomeAndBack();

        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);
    });

    testWithFreshUser("should persist language after auto-save and navigation to home", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);

        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("english");
        await expect(profilePage.langEnglish).toHaveClass(/border-\[var\(--fg-black\)\]/);

        await profilePage.waitForAutoSave();

        await profilePage.navigateToHomeAndBack();

        await expect(profilePage.langEnglish).toHaveClass(/border-\[var\(--fg-black\)\]/);
    });

    // Regression: catches merge_current_user() data loss bug
    testWithFreshUser("should persist daily load after merge triggered by home sync", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);

        await profilePage.selectDailyLoad("heavy");
        await expect(profilePage.loadHeavy).toHaveClass(/btn-olive/);

        await profilePage.waitForAutoSave();

        await profilePage.navigateToHomeAndWaitForSync();

        await profilePage.expectProfileVisible();

        await expect(profilePage.loadHeavy).toHaveClass(/btn-olive/);
    });

    testWithFreshUser("should persist both daily load and language after auto-save", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);

        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("english");
        await profilePage.selectDailyLoad("hard");

        await profilePage.waitForAutoSave();

        await profilePage.navigateToHomeAndBack();

        await expect(profilePage.langEnglish).toHaveClass(/border-\[var\(--fg-black\)\]/);
        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);
    });

    testWithFreshUser("should show autosave status on language change", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("english");
        const status = page.getByTestId("profile-autosave-status");
        await expect(status).toBeVisible({ timeout: 5_000 });
        await expect(status).toContainText(/saved|сохранено/i, { timeout: 10_000 });
    });

    testWithFreshUser("should logout and redirect to login page", async ({ page }) => {
        await setupProfilePage(page);

        await page.getByTestId("profile-logout-btn").click();

        const loginPage = new LoginPage(page);
        await expect(loginPage.loginPage).toBeVisible({ timeout: 10_000 });
        await expect(page).toHaveURL(/\/login/);
    });
});

testWithFreshUser.describe("Profile - Password Change", () => {
    testWithFreshUser("should display password change card", async ({ page }) => {
        await setupProfilePage(page);

        await page.getByTestId("profile-password").scrollIntoViewIfNeeded();
        await page.getByTestId("profile-password").click();
        await expect(page.getByTestId("current-password")).toBeVisible({ timeout: 10_000 });
        await expect(page.getByTestId("new-password")).toBeVisible();
        await expect(page.getByTestId("confirm-password")).toBeVisible();
        await expect(page.getByTestId("change-password-btn")).toBeVisible();
    });

    testWithFreshUser("should show error for mismatched passwords", async ({ page }) => {
        await setupProfilePage(page);

        await page.getByTestId("profile-password").scrollIntoViewIfNeeded();
        await page.getByTestId("profile-password").click();
        await page.getByTestId("current-password").fill("any-password");
        await page.getByTestId("new-password").fill("short");
        await page.getByTestId("confirm-password").fill("different");
        await page.getByTestId("change-password-btn").click();

        await expect(page.getByTestId("password-error")).toBeVisible({ timeout: 5000 });
    });

    testWithFreshUser("should show error for short new password", async ({ page }) => {
        await setupProfilePage(page);

        await page.getByTestId("profile-password").scrollIntoViewIfNeeded();
        await page.getByTestId("profile-password").click();
        await page.getByTestId("current-password").fill(DEFAULT_TEST_PASSWORD);
        await page.getByTestId("new-password").fill("abc");
        await page.getByTestId("confirm-password").fill("abc");
        await page.getByTestId("change-password-btn").click();

        await expect(page.getByTestId("password-error")).toBeVisible({ timeout: 5000 });
    });

    testWithFreshUser("should change password and show success", async ({ page }) => {
        await setupProfilePage(page);

        await page.getByTestId("profile-password").scrollIntoViewIfNeeded();
        await page.getByTestId("profile-password").click();
        await page.getByTestId("current-password").fill(DEFAULT_TEST_PASSWORD);
        await page.getByTestId("new-password").fill(DEFAULT_TEST_PASSWORD);
        await page.getByTestId("confirm-password").fill(DEFAULT_TEST_PASSWORD);
        await page.getByTestId("change-password-btn").click();

        await expect(page.getByTestId("password-success")).toBeVisible({ timeout: 10_000 });
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

        // Wait for login page — account deletion is async, redirect may be slow
        await page.locator('input[type="email"], input[data-testid="email-input"]').waitFor({ state: "visible", timeout: 60_000 });
    });
});
