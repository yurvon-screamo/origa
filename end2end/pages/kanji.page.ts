import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

type KanjiLevel = "N5" | "N4" | "N3" | "N2" | "N1";
type FilterType = "Все" | "Новые" | "Сложные" | "В процессе" | "Изученные";

export class KanjiPage extends BasePage {
	readonly kanjiPage: Locator;
	readonly kanjiCard: Locator;
	readonly kanjiTitle: Locator;
	readonly radicalsBtn: Locator;
	readonly backBtn: Locator;
	readonly addBtn: Locator;
	readonly searchInput: Locator;
	readonly kanjiGrid: Locator;
	readonly emptyState: Locator;

	readonly drawer: Locator;
	readonly drawerAddBtn: Locator;
	readonly drawerSelectAllBtn: Locator;

	readonly deleteModal: Locator;
	readonly deleteConfirmBtn: Locator;
	readonly deleteCancelBtn: Locator;

	constructor(page: Page) {
		super(page);

		this.kanjiPage = page.getByTestId("kanji-page");
		this.kanjiCard = page.getByTestId("kanji-card");
		this.kanjiTitle = page.getByTestId("kanji-title");
		this.radicalsBtn = page.getByTestId("kanji-radicals-btn");
		this.backBtn = page.getByTestId("kanji-back-btn");
		this.addBtn = page.getByTestId("kanji-add-btn");
		this.searchInput = page.getByTestId("kanji-search-input");
		this.kanjiGrid = page.getByTestId("kanji-grid");
		this.emptyState = page.getByTestId("kanji-empty-state");

		this.drawer = page.locator(".drawer-content");
		this.drawerAddBtn = this.drawer.getByRole("button", { name: "Добавить" });
		this.drawerSelectAllBtn = this.drawer.getByRole("button", { name: "Выделить все" });

		this.deleteModal = page.locator(".modal-content");
		this.deleteConfirmBtn = page.getByRole("button", { name: "Удалить" });
		this.deleteCancelBtn = page.getByRole("button", { name: "Отмена" });
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

	async openAddModal(): Promise<void> {
		await this.addBtn.click();
		await expect(this.drawer).toBeVisible({ timeout: 5000 });
	}

	async selectLevel(level: KanjiLevel): Promise<void> {
		await this.drawer.getByRole("button", { name: level }).click();
		const kanjiItem = this.drawer.locator(".border.cursor-pointer").first();
		const emptyMsg = this.drawer.getByText("Нет кандзи для выбранного уровня");
		await expect(kanjiItem.or(emptyMsg)).toBeVisible({ timeout: 10_000 });
	}

	async selectKanji(kanji: string): Promise<void> {
		const item = this.drawer.locator(".border.cursor-pointer", { hasText: kanji });
		await expect(item).toBeVisible({ timeout: 5000 });
		await item.click();
	}

	async selectAllKanji(): Promise<void> {
		await this.drawerSelectAllBtn.click();
	}

	async addSelectedKanji(): Promise<void> {
		await this.drawerAddBtn.click({ timeout: 5000 });
		await expect(this.drawer).not.toBeVisible({ timeout: 15_000 });
	}

	async selectFilter(name: FilterType): Promise<void> {
		await this.kanjiPage
			.getByRole("button", { name: new RegExp(`${name} \\(\\d+\\)`) })
			.click();
	}

	async getCardCount(): Promise<number> {
		return this.kanjiGrid.locator(".card").count();
	}

	async deleteCardByIndex(index: number): Promise<void> {
		const card = this.kanjiGrid.locator(".card").nth(index);
		await card.locator("button").last().click();
		await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
		await this.deleteConfirmBtn.click();
		await expect(this.deleteModal).not.toBeVisible({ timeout: 10_000 });
	}

	async cancelDeleteCardByIndex(index: number): Promise<void> {
		const card = this.kanjiGrid.locator(".card").nth(index);
		await card.locator("button").last().click();
		await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
		await this.deleteCancelBtn.click();
		await expect(this.deleteModal).not.toBeVisible({ timeout: 5000 });
	}
}
