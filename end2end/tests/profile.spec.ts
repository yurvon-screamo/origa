import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { ProfilePage } from "../pages";

async function setupProfilePage(page: Page): Promise<ProfilePage> {
	await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10_000 });
	await page.getByTestId("onboarding-skip").click();
	await page.waitForURL(/\/home$/, { timeout: 10_000 });
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
});
