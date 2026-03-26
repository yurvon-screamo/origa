import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class WordsPage extends BasePage {
	// Page structure
	readonly wordsPage: Locator;
	readonly wordsCard: Locator;
	readonly wordsTitle: Locator;

	// Navigation buttons
	readonly backButton: Locator;
	readonly setsButton: Locator;
	readonly addButton: Locator;

	// Search
	readonly searchInput: Locator;

	// Content
	readonly wordsGrid: Locator;
	readonly emptyState: Locator;

	constructor(page: Page) {
		super(page);

		// Page structure
		this.wordsPage = page.getByTestId("words-page");
		this.wordsCard = page.getByTestId("words-card");
		this.wordsTitle = page.getByTestId("words-title");

		// Navigation buttons
		this.backButton = page.getByTestId("words-back-btn");
		this.setsButton = page.getByTestId("words-sets-btn");
		this.addButton = page.getByTestId("words-add-btn");

		// Search
		this.searchInput = page.getByTestId("words-search-input");

		// Content
		this.wordsGrid = page.getByTestId("words-grid");
		this.emptyState = page.getByTestId("words-empty-state");
	}

	async goto(): Promise<void> {
		await this.navigate("/words");
	}

	async expectWordsVisible(): Promise<void> {
		await expect(this.wordsPage).toBeVisible();
		await expect(this.wordsCard).toBeVisible();
		await expect(this.wordsTitle).toBeVisible();
	}

	async searchWords(query: string): Promise<void> {
		await this.searchInput.waitFor({ state: "visible", timeout: 5000 });
		await this.searchInput.click({ force: true });
		await this.searchInput.fill(query, { force: true });
	}

	async clickBack(): Promise<void> {
		await this.backButton.click();
	}

	async clickSets(): Promise<void> {
		await this.setsButton.click();
	}

	async clickAddWords(): Promise<void> {
		await this.addButton.click();
	}
}