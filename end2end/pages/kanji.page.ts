import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class KanjiPage extends BasePage {
	// Page structure
	readonly kanjiPage: Locator;
	readonly kanjiCard: Locator;
	readonly kanjiTitle: Locator;

	// Navigation buttons
	readonly radicalsBtn: Locator;
	readonly backBtn: Locator;
	readonly addBtn: Locator;

	// Search
	readonly searchInput: Locator;

	// Content
	readonly kanjiGrid: Locator;
	readonly emptyState: Locator;

	constructor(page: Page) {
		super(page);

		// Page structure
		this.kanjiPage = page.getByTestId("kanji-page");
		this.kanjiCard = page.getByTestId("kanji-card");
		this.kanjiTitle = page.getByTestId("kanji-title");

		// Navigation buttons
		this.radicalsBtn = page.getByTestId("kanji-radicals-btn");
		this.backBtn = page.getByTestId("kanji-back-btn");
		this.addBtn = page.getByTestId("kanji-add-btn");

		// Search
		this.searchInput = page.getByTestId("kanji-search-input");

		// Content
		this.kanjiGrid = page.getByTestId("kanji-grid");
		this.emptyState = page.getByTestId("kanji-empty-state");
	}

	async goto(): Promise<void> {
		await this.navigate("/kanji");
	}

	async expectKanjiVisible(): Promise<void> {
		await expect(this.kanjiPage).toBeVisible();
		await expect(this.kanjiCard).toBeVisible();
		await expect(this.kanjiTitle).toBeVisible();
	}

	async searchKanji(query: string): Promise<void> {
		await this.searchInput.fill(query);
	}

	async clickBack(): Promise<void> {
		await this.backBtn.click();
	}

	async clickRadicals(): Promise<void> {
		await this.radicalsBtn.click();
	}

	async clickAddKanji(): Promise<void> {
		await this.addBtn.click();
	}
}