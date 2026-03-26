import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class SetsPage extends BasePage {
	// Page structure
	readonly setsPage: Locator;
	readonly setsCard: Locator;
	readonly setsTitle: Locator;

	// Navigation
	readonly backBtn: Locator;

	// Loading
	readonly loading: Locator;
	readonly loadingText: Locator;

	// Search
	readonly searchInput: Locator;

	// Level filters
	readonly levelAll: Locator;
	readonly levelN5: Locator;
	readonly levelN4: Locator;
	readonly levelN3: Locator;
	readonly levelN2: Locator;
	readonly levelN1: Locator;

	// Import filters
	readonly importAll: Locator;
	readonly importImported: Locator;
	readonly importNew: Locator;

	// Import actions
	readonly importSelectedBtn: Locator;
	readonly cancelSelectBtn: Locator;

	constructor(page: Page) {
		super(page);

		// Page structure
		this.setsPage = page.getByTestId("sets-page");
		this.setsCard = page.getByTestId("sets-card");
		this.setsTitle = page.getByTestId("sets-title");

		// Navigation
		this.backBtn = page.getByTestId("sets-back-btn");

		// Loading
		this.loading = page.getByTestId("sets-loading");
		this.loadingText = page.getByTestId("sets-loading-text");

		// Search
		this.searchInput = page.getByTestId("sets-search-input");

		// Level filters
		this.levelAll = page.getByTestId("sets-level-all");
		this.levelN5 = page.getByTestId("sets-level-n5");
		this.levelN4 = page.getByTestId("sets-level-n4");
		this.levelN3 = page.getByTestId("sets-level-n3");
		this.levelN2 = page.getByTestId("sets-level-n2");
		this.levelN1 = page.getByTestId("sets-level-n1");

		// Import filters
		this.importAll = page.getByTestId("sets-import-all");
		this.importImported = page.getByTestId("sets-import-imported");
		this.importNew = page.getByTestId("sets-import-new");

		// Import actions
		this.importSelectedBtn = page.getByTestId("sets-import-selected-btn");
		this.cancelSelectBtn = page.getByTestId("sets-cancel-select-btn");
	}

	async goto(): Promise<void> {
		await this.navigate("/sets");
	}

	async expectSetsVisible(): Promise<void> {
		await expect(this.setsPage).toBeVisible();
		await expect(this.setsCard).toBeVisible();
		await expect(this.setsTitle).toBeVisible();
	}

	async searchSets(query: string): Promise<void> {
		await this.searchInput.fill(query);
	}

	async selectLevelFilter(level: string): Promise<void> {
		await this.page.getByTestId(`sets-level-${level.toLowerCase()}`).click();
	}

	async selectTypeFilter(type: string): Promise<void> {
		await this.page.getByTestId(`sets-type-${type.toLowerCase()}`).click();
	}

	async selectImportFilter(filter: string): Promise<void> {
		await this.page.getByTestId(`sets-import-${filter.toLowerCase()}`).click();
	}

	async clickBack(): Promise<void> {
		await this.backBtn.click();
	}

	async clickImportSelected(): Promise<void> {
		await this.importSelectedBtn.click();
	}

	async waitForLoading(): Promise<void> {
		await expect(this.loading).toBeVisible();
		await expect(this.loading).toBeHidden();
	}
}