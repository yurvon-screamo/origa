import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

type GrammarLevel = "N5" | "N4" | "N3" | "N2" | "N1";
type FilterType = "Все" | "Новые" | "Сложные" | "В процессе" | "Изученные";

export class GrammarPage extends BasePage {
	readonly grammarPage: Locator;
	readonly grammarCard: Locator;
	readonly grammarTitle: Locator;
	readonly backBtn: Locator;
	readonly addBtn: Locator;
	readonly searchInput: Locator;
	readonly grammarGrid: Locator;
	readonly emptyState: Locator;

	readonly drawer: Locator;
	readonly drawerAddBtn: Locator;
	readonly drawerSelectAllBtn: Locator;
	readonly drawerSearchInput: Locator;

	readonly deleteModal: Locator;
	readonly deleteConfirmBtn: Locator;
	readonly deleteCancelBtn: Locator;

	constructor(page: Page) {
		super(page);

		this.grammarPage = page.getByTestId("grammar-page");
		this.grammarCard = page.getByTestId("grammar-card");
		this.grammarTitle = page.getByTestId("grammar-title");
		this.backBtn = page.getByTestId("grammar-back-btn");
		this.addBtn = page.getByTestId("grammar-add-btn");
		this.searchInput = page.getByTestId("grammar-search-input");
		this.grammarGrid = page.getByTestId("grammar-grid");
		this.emptyState = page.getByTestId("grammar-empty-state");

		this.drawer = page.locator(".drawer-content");
		this.drawerAddBtn = this.drawer.getByRole("button", { name: "Добавить" });
		this.drawerSelectAllBtn = this.drawer.getByRole("button", { name: "Выделить все" });
		this.drawerSearchInput = this.drawer.locator(".search-input");

		this.deleteModal = page.locator(".modal-content");
		this.deleteConfirmBtn = page.getByRole("button", { name: "Удалить" });
		this.deleteCancelBtn = page.getByRole("button", { name: "Отмена" });
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
		await this.searchInput.fill(query);
	}

	async clickBack(): Promise<void> {
		await this.backBtn.click();
	}

	async openAddModal(): Promise<void> {
		await this.addBtn.click();
		await expect(this.drawer).toBeVisible({ timeout: 5000 });
	}

	async selectLevel(level: GrammarLevel): Promise<void> {
		await this.drawer.getByRole("button", { name: level }).click();
		const ruleItem = this.drawer.locator(".border.cursor-pointer").first();
		const emptyMsg = this.drawer.getByText("Нет правил для выбранного уровня");
		await expect(ruleItem.or(emptyMsg)).toBeVisible({ timeout: 10_000 });
	}

	async selectRule(title: string): Promise<void> {
		const rule = this.drawer.locator(".border.cursor-pointer", { hasText: title });
		await expect(rule).toBeVisible({ timeout: 5000 });
		await rule.click();
	}

	async selectAllRules(): Promise<void> {
		await this.drawerSelectAllBtn.click();
	}

	async addSelectedRules(): Promise<void> {
		await this.drawerAddBtn.click({ timeout: 5000 });
		await expect(this.drawer).not.toBeVisible({ timeout: 15_000 });
	}

	async selectFilter(name: FilterType): Promise<void> {
		await this.grammarPage
			.getByRole("button", { name: new RegExp(`${name} \\(\\d+\\)`) })
			.click();
	}

	async getCardCount(): Promise<number> {
		return this.grammarGrid.locator(".card").count();
	}

	async deleteCardByIndex(index: number): Promise<void> {
		const card = this.grammarGrid.locator(".card").nth(index);
		await card.locator("button").last().click();
		await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
		await this.deleteConfirmBtn.click();
		await expect(this.deleteModal).not.toBeVisible({ timeout: 10_000 });
	}

	async cancelDeleteCardByIndex(index: number): Promise<void> {
		const card = this.grammarGrid.locator(".card").nth(index);
		await card.locator("button").last().click();
		await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
		await this.deleteCancelBtn.click();
		await expect(this.deleteModal).not.toBeVisible({ timeout: 5000 });
	}
}
