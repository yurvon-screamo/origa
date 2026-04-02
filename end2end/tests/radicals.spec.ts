import { test, expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { RadicalsPage } from "../pages";

async function setupRadicalsWithData(page: Page): Promise<RadicalsPage> {
	// Wait for post-login redirect
	await page.waitForLoadState("networkidle");

	// Check current URL
	const url = page.url();

	// If already on home, just navigate to radicals
	if (url.includes("/home")) {
		const radicalsPage = new RadicalsPage(page);
		await radicalsPage.goto();
		await radicalsPage.expectRadicalsVisible();
		return radicalsPage;
	}

	// If on onboarding, wait for it to be visible
	try {
		await expect(page.getByTestId("onboarding-page")).toBeVisible({ timeout: 5000 });
	} catch {
		// Onboarding not visible within 5s, maybe we need to wait more
		await page.waitForTimeout(3000);
	}

	// Now click through onboarding
	await page.getByTestId("onboarding-next").click();
	await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();
	await page.getByTestId("jlpt-option-n4").click();
	await page.getByTestId("onboarding-next").click();

	await expect(page.getByTestId("onboarding-apps-step")).toBeVisible();
	await page.getByTestId("apps-step-app-Migii-checkbox").click();
	await page.getByTestId("onboarding-next").click();
	await page.getByTestId("onboarding-next").click();

	await expect(page.getByTestId("onboarding-summary-step")).toBeVisible();
	await page.getByTestId("onboarding-import").click();
	await page.waitForURL(/\/home$/, { timeout: 120_000 });

	const radicalsPage = new RadicalsPage(page);
	await radicalsPage.goto();
	await radicalsPage.expectRadicalsVisible();
	return radicalsPage;
}

testWithFreshUser.describe("Radicals Page - Search & Filters", () => {
	testWithFreshUser("should display radicals after import", async ({ page }) => {
		test.setTimeout(180_000);
		const radicalsPage = await setupRadicalsWithData(page);
		await expect(radicalsPage.radicalsGrid).toBeVisible({ timeout: 10_000 });
		await page.waitForFunction(() => document.querySelector('[data-testid="radicals-card-item"]') !== null, { timeout: 10000 });
		await expect(radicalsPage.radicalsEmptyState).not.toBeVisible();
	});

	testWithFreshUser("should search radicals", async ({ page }) => {
		test.setTimeout(180_000);
		const radicalsPage = await setupRadicalsWithData(page);
		await expect(radicalsPage.radicalsGrid).toBeVisible({ timeout: 10_000 });

		const firstRadicalChar = await radicalsPage.radicalsGrid.locator(".card").first().locator(".font-serif").first().textContent();
		if (firstRadicalChar) {
			await radicalsPage.searchRadicals(firstRadicalChar.trim());
			await expect(radicalsPage.radicalsEmptyState).not.toBeVisible();
		}

		await radicalsPage.searchRadicals("xyznonexistent");
		await expect(radicalsPage.radicalsEmptyState).toBeVisible({ timeout: 5000 });

		await radicalsPage.searchRadicals("");
		await expect(radicalsPage.radicalsGrid).toBeVisible({ timeout: 5000 });
	});

	testWithFreshUser("should filter radicals by status", async ({ page }) => {
		test.setTimeout(180_000);
		const radicalsPage = await setupRadicalsWithData(page);
		await expect(radicalsPage.radicalsGrid).toBeVisible({ timeout: 10_000 });

		const totalBefore = await radicalsPage.getCardCount();
		expect(totalBefore).toBeGreaterThan(0);

		await radicalsPage.selectFilter("Новые");
		const newCount = await radicalsPage.getCardCount();
		expect(newCount).toBeGreaterThan(0);

		await radicalsPage.selectFilter("Изученные");
		await expect(radicalsPage.radicalsEmptyState).toBeVisible({ timeout: 5000 });
	});
});

testWithFreshUser.describe("Radicals Page - Navigation", () => {
	testWithFreshUser("should navigate back to home", async ({ page }) => {
		test.setTimeout(180_000);
		const radicalsPage = await setupRadicalsWithData(page);
		await radicalsPage.clickBack();
		await page.waitForURL(/\/home$/, { timeout: 10000 });
		await expect(page).toHaveURL(/\/home$/);
	});
});
