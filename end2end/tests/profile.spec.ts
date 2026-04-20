import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { ProfilePage } from "../pages";

async function setupProfilePage(page: Page): Promise<ProfilePage> {
    await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10_000 });
    await page.getByTestId("onboarding-skip").click();
    await expect(page.getByTestId("home-content")).toBeVisible({ timeout: 30_000 });
    const profilePage = new ProfilePage(page);
    await profilePage.goto();
    await profilePage.expectProfileVisible();
    return profilePage;
}

testWithFreshUser.describe("Profile Page", () => {
    testWithFreshUser("should display profile page", async ({ page }) => {
        const profilePage = await setupProfilePage(page);

        await expect(profilePage.profilePage).toBeVisible();
        await expect(profilePage.profileTitle).toBeVisible();
        await expect(profilePage.profileContent).toBeVisible();
    });

    testWithFreshUser("should navigate back to home", async ({ page }) => {
        const profilePage = await setupProfilePage(page);

        await profilePage.clickBack();

        await page.waitForURL(/\/home$/, { timeout: 10_000 });
    });

    testWithFreshUser("should display username in profile title", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        const titleText = await profilePage.profileTitle.textContent();
        expect(titleText).toContain("Профиль");
        expect(titleText?.length).toBeGreaterThan("Профиль".length);
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
        await expect(profilePage.loadImpossible).toBeVisible();
    });

    testWithFreshUser("should switch language selection", async ({ page }) => {
        const profilePage = await setupProfilePage(page);
        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("english");
        await expect(profilePage.langEnglish).toHaveClass(/btn-olive/);
        await expect(profilePage.langRussian).not.toHaveClass(/btn-olive/);
        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("russian");
        await expect(profilePage.langRussian).toHaveClass(/btn-olive/);
        await expect(profilePage.langEnglish).not.toHaveClass(/btn-olive/);
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
        test.setTimeout(60_000);
        const profilePage = await setupProfilePage(page);
        await page.mouse.click(0, 0);
        await profilePage.selectLanguage("english");
        await expect(profilePage.langEnglish).toHaveClass(/btn-olive/);
        await profilePage.saveProfile();
        await page.waitForTimeout(2000);
        await page.reload();
        await profilePage.waitForLoad();
        await profilePage.expectProfileVisible();
        await expect(profilePage.langEnglish).toHaveClass(/btn-olive/);
    });

    testWithFreshUser("should persist daily load after save", async ({ page }) => {
        test.setTimeout(60_000);
        const profilePage = await setupProfilePage(page);
        await profilePage.selectDailyLoad("hard");
        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);
        await profilePage.saveProfile();
        await page.waitForTimeout(2000);
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
        await expect(profilePage.langEnglish).toHaveClass(/btn-olive/);

        await profilePage.saveProfile();
        await profilePage.waitForSaveComplete();

        await profilePage.navigateToHomeAndBack();

        await expect(profilePage.langEnglish).toHaveClass(/btn-olive/);
    });

    // Regression: catches merge_current_user() data loss bug
    testWithFreshUser("should persist daily load after merge triggered by home sync", async ({ page }) => {
        test.setTimeout(90_000);
        const profilePage = await setupProfilePage(page);

        await profilePage.selectDailyLoad("impossible");
        await expect(profilePage.loadImpossible).toHaveClass(/btn-olive/);

        await profilePage.saveProfile();
        await profilePage.waitForSaveComplete();

        await profilePage.navigateToHomeAndWaitForSync();

        await profilePage.expectProfileVisible();

        await expect(profilePage.loadImpossible).toHaveClass(/btn-olive/);
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

        await expect(profilePage.langEnglish).toHaveClass(/btn-olive/);
        await expect(profilePage.loadHard).toHaveClass(/btn-olive/);
    });
});
