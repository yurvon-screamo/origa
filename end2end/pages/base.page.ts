import { Page, Locator } from "@playwright/test";

/**
 * Base page object with common methods
 */
export abstract class BasePage {
	constructor(protected readonly page: Page) {}

	async navigate(path: string): Promise<void> {
		await this.page.goto(path);
		await this.waitForLoad();
	}

	async waitForLoad(): Promise<void> {
		await this.page.waitForLoadState("domcontentloaded");
		await this.page.waitForTimeout(500);
		await this.page.evaluate(() => {
			const overlays = document.querySelectorAll(".loading-overlay");
			overlays.forEach((el) => {
				(el as HTMLElement).style.display = "none";
			});
		});
		try {
			await this.page.waitForSelector("input, button", {
				state: "visible",
				timeout: 10000,
			});
		} catch {}
	}

	async waitForElement(locator: Locator, timeout = 5000): Promise<void> {
		await locator.waitFor({ state: "visible", timeout });
	}

	async screenshot(name: string): Promise<void> {
		await this.page.screenshot({ path: "test-results/" + name + ".png" });
	}

	getCurrentPath(): string {
		const url = new URL(this.page.url());
		return url.pathname;
	}

	async isVisible(locator: Locator): Promise<boolean> {
		try {
			await locator.waitFor({ state: "visible", timeout: 2000 });
			return true;
		} catch {
			return false;
		}
	}
}
