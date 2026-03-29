import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

type FilterType = "Все" | "Новые" | "Сложные" | "В процессе" | "Изученные";

export class RadicalsPage extends BasePage {
	readonly radicalsPage: Locator;
	readonly radicalsCard: Locator;
	readonly radicalsHeading: Locator;
	readonly radicalsBackBtn: Locator;
	readonly radicalsSearchInput: Locator;
	readonly radicalsGrid: Locator;
	readonly radicalsEmptyState: Locator;

	constructor(page: Page) {
		super(page);

		this.radicalsPage = page.getByTestId("radicals-page");
		this.radicalsCard = page.getByTestId("radicals-card");
		this.radicalsHeading = page.getByTestId("radicals-heading");
		this.radicalsBackBtn = page.getByTestId("radicals-back-btn");
		this.radicalsSearchInput = page.getByTestId("radicals-search-input");
		this.radicalsGrid = page.getByTestId("radicals-grid");
		this.radicalsEmptyState = page.getByTestId("radicals-empty-state");
	}

	async goto(): Promise<void> {
		await this.navigate("/radicals");
	}

	async expectRadicalsVisible(): Promise<void> {
		await expect(this.radicalsPage).toBeVisible();
		await expect(this.radicalsCard).toBeVisible();
		await expect(this.radicalsHeading).toBeVisible();
	}

	async searchRadicals(query: string): Promise<void> {
		await this.radicalsSearchInput.fill(query);
	}

	async clickBack(): Promise<void> {
		await this.radicalsBackBtn.click();
	}

	async selectFilter(name: FilterType): Promise<void> {
		await this.radicalsPage
			.getByRole("button", { name: new RegExp(`${name} \\(\\d+\\)`) })
			.click();
	}

	async getCardCount(): Promise<number> {
		return this.radicalsGrid.locator(".card").count();
	}
}
