import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class GrammarPage extends BasePage {
	// Page structure
	readonly grammarPage: Locator;
	readonly grammarCard: Locator;
	readonly grammarTitle: Locator;

	// Navigation buttons
	readonly backBtn: Locator;
	readonly addBtn: Locator;

	// Search
	readonly searchInput: Locator;

	// Content
	readonly grammarGrid: Locator;
	readonly emptyState: Locator;

	constructor(page: Page) {
		super(page);

		// Page structure
		this.grammarPage = page.getByTestId("grammar-page");
		this.grammarCard = page.getByTestId("grammar-card");
		this.grammarTitle = page.getByTestId("grammar-title");

		// Navigation buttons
		this.backBtn = page.getByTestId("grammar-back-btn");
		this.addBtn = page.getByTestId("grammar-add-btn");

		// Search
		this.searchInput = page.getByTestId("grammar-search-input");

		// Content
		this.grammarGrid = page.getByTestId("grammar-grid");
		this.emptyState = page.getByTestId("grammar-empty-state");
	}

	async goto(): Promise<void> {
		await this.navigate("/grammar");
	}

	async expectGrammarVisible(): Promise<void> {
		await expect(this.grammarPage).toBeVisible();
		await expect(this.grammarCard).toBeVisible();
		await expect(this.grammarTitle).toBeVisible();
	}

	async searchGrammar(query: string): Promise<void> {
		await this.searchInput.waitFor({ state: "visible", timeout: 5000 });
		await this.searchInput.click({ force: true });
		await this.searchInput.fill(query, { force: true });
	}

	async clickBack(): Promise<void> {
		await this.backBtn.click();
	}

	async clickAddGrammar(): Promise<void> {
		await this.addBtn.click();
	}
}