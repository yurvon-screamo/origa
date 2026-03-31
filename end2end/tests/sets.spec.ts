import { expect, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { SetsPage, WordsPage } from "../pages";

async function setupSetsPage(page: Page): Promise<SetsPage> {
	await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10000 });
	await page.getByTestId("onboarding-skip").click();
	await page.waitForURL(/\/home$/, { timeout: 10000 });

	const setsPage = new SetsPage(page);
	await setsPage.goto();
	await setsPage.expectSetsVisible();
	await setsPage.waitForLoad();
	return setsPage;
}

async function importFirstSet(setsPage: SetsPage): Promise<void> {
	await setsPage.clickImportOnCard(0);
	await setsPage.waitForDrawerWords();
	await setsPage.importFromDrawer();
}

testWithFreshUser.describe("Sets Page - Import", () => {
	testWithFreshUser("should display sets list for new user", async ({ page }) => {
		const setsPage = await setupSetsPage(page);
		expect(await setsPage.getSetCardCount()).toBeGreaterThan(0);
		expect(await setsPage.getImportedCardCount()).toBe(0);
	});

	testWithFreshUser("should import a single set", async ({ page }) => {
		test.setTimeout(60_000);
		const setsPage = await setupSetsPage(page);
		const countBefore = await setsPage.getImportedCardCount();
		await importFirstSet(setsPage);
		expect(await setsPage.getImportedCardCount()).toBe(countBefore + 1);
	});

	testWithFreshUser("should cancel set import", async ({ page }) => {
		test.setTimeout(60_000);
		const setsPage = await setupSetsPage(page);
		const countBefore = await setsPage.getImportedCardCount();
		await setsPage.clickImportOnCard(0);
		await setsPage.waitForDrawerWords();
		await setsPage.cancelImportFromDrawer();
		await expect(setsPage.drawer).not.toBeVisible();
		expect(await setsPage.getImportedCardCount()).toBe(countBefore);
	});

	testWithFreshUser("should select multiple sets and import them", async ({ page }) => {
		test.setTimeout(60_000);
		const setsPage = await setupSetsPage(page);
		const cardCount = await setsPage.getSetCardCount();
		if (cardCount >= 2) {
			await setsPage.selectSetCheckbox(0);
			await setsPage.selectSetCheckbox(1);
			await expect(setsPage.importSelectedBtn).toBeVisible();
			await setsPage.clickImportSelected();
			await setsPage.waitForDrawerWords();
			await setsPage.importFromDrawer();
			expect(await setsPage.getImportedCardCount()).toBeGreaterThanOrEqual(2);
		}
	});

	testWithFreshUser("should cancel set selection", async ({ page }) => {
		test.setTimeout(60_000);
		const setsPage = await setupSetsPage(page);
		const cardCount = await setsPage.getSetCardCount();
		if (cardCount >= 1) {
			await setsPage.selectSetCheckbox(0);
			await expect(setsPage.importSelectedBtn).toBeVisible();
			await setsPage.cancelSelection();
			await expect(setsPage.importSelectedBtn).not.toBeVisible();
		}
	});
});

testWithFreshUser.describe("Sets Page - Search & Filters", () => {
	testWithFreshUser("should search sets", async ({ page }) => {
		test.setTimeout(60_000);
		const setsPage = await setupSetsPage(page);
		const cardCount = await setsPage.getSetCardCount();
		expect(cardCount).toBeGreaterThan(0);

		// Get the title of the first set card
		const firstTitle = await setsPage.setsPage.locator(".card").first().locator("h4").textContent();
		if (firstTitle) {
			await setsPage.searchSets(firstTitle.trim());
			expect(await setsPage.getSetCardCount()).toBeGreaterThanOrEqual(1);
		}

		await setsPage.searchSets("xyznonexistent999");
		expect(await setsPage.getSetCardCount()).toBe(0);

		await setsPage.searchSets("");
		expect(await setsPage.getSetCardCount()).toBeGreaterThan(0);
	});

	testWithFreshUser("should filter sets by level", async ({ page }) => {
		test.setTimeout(60_000);
		const setsPage = await setupSetsPage(page);
		const totalSets = await setsPage.getSetCardCount();

		await setsPage.selectLevelFilter("n5");
		const n5Count = await setsPage.getSetCardCount();
		expect(n5Count).toBeLessThanOrEqual(totalSets);

		await setsPage.selectLevelFilter("all");
		expect(await setsPage.getSetCardCount()).toBe(totalSets);
	});

	testWithFreshUser("should filter sets by import status", async ({ page }) => {
		test.setTimeout(60_000);
		const setsPage = await setupSetsPage(page);

		await setsPage.selectImportFilter("imported");
		expect(await setsPage.getSetCardCount()).toBe(0);

		await setsPage.selectImportFilter("new");
		expect(await setsPage.getSetCardCount()).toBeGreaterThan(0);

		await setsPage.selectImportFilter("all");
		expect(await setsPage.getSetCardCount()).toBeGreaterThan(0);
	});
});

testWithFreshUser.describe("Sets Page - Navigation", () => {
	testWithFreshUser("should navigate back to words page", async ({ page }) => {
		const setsPage = await setupSetsPage(page);
		await setsPage.clickBack();
		await page.waitForURL(/\/words$/, { timeout: 10000 });
		await expect(page).toHaveURL(/\/words$/);

		const wordsPage = new WordsPage(page);
		await wordsPage.expectWordsVisible();
	});
});
