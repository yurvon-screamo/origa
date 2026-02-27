import { expect, type Locator, type Page } from "@playwright/test";

export class SetsPage {
	readonly page: Page;
	readonly heading: Locator;
	readonly levelGroups: Locator;
	readonly setCards: Locator;
	readonly importButtons: Locator;
	readonly importResult: Locator;

	constructor(page: Page) {
		this.page = page;
		this.heading = page.getByRole("heading", { name: "Наборы для изучения" });
		this.levelGroups = page.locator(".sets-group");
		this.setCards = page.locator(".set-card");
		this.importButtons = page.locator(".set-card button");
		this.importResult = page.locator("text=/Импортировано|Ошибка/");
	}

	async goto() {
		await this.page.goto("/sets");
	}

	async expectVisible() {
		await expect(this.heading).toBeVisible();
	}

	async hasLevelGroups(): Promise<boolean> {
		const count = await this.levelGroups.count();
		return count > 0;
	}

	async getLevelGroupCount(): Promise<number> {
		return await this.levelGroups.count();
	}

	async getSetCardCount(): Promise<number> {
		return await this.setCards.count();
	}

	async clickFirstImport(): Promise<boolean> {
		const firstBtn = this.importButtons.first();
		const isVisible = await firstBtn.isVisible({ timeout: 2000 }).catch(() => false);
		if (isVisible && await firstBtn.isEnabled()) {
			await firstBtn.click();
			return true;
		}
		return false;
	}

	async waitForImportResult(timeout: number = 30000): Promise<string | null> {
		try {
			await this.importResult.waitFor({ state: "visible", timeout });
			return await this.importResult.textContent();
		} catch {
			return null;
		}
	}

	async navigateViaTabBar() {
		await this.page.goto("/sets");
	}
}
