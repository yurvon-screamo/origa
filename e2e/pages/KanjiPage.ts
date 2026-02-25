import { expect, type Locator, type Page } from "@playwright/test";

export class KanjiPage {
	readonly page: Page;
	readonly heading: Locator;
	readonly searchInput: Locator;
	readonly addButton: Locator;
	readonly filterAll: Locator;
	readonly filterNew: Locator;
	readonly filterHard: Locator;
	readonly filterInProgress: Locator;
	readonly filterLearned: Locator;
	readonly kanjiCards: Locator;
	readonly emptyMessage: Locator;
	readonly modal: Locator;

	constructor(page: Page) {
		this.page = page;
		this.heading = page.getByRole("heading", { name: "Кандзи" });
		this.searchInput = page.getByPlaceholder("Поиск...");
		this.addButton = page.getByRole("button", { name: "+" });
		this.filterAll = page.getByRole("button", { name: /Все/ });
		this.filterNew = page.getByRole("button", { name: /Новые/ });
		this.filterHard = page.getByRole("button", { name: /Сложные/ });
		this.filterInProgress = page.getByRole("button", { name: /В процессе/ });
		this.filterLearned = page.getByRole("button", { name: /Изученные/ });
		this.kanjiCards = page.locator(".card");
		this.emptyMessage = page.getByText("Кандзи не найдено");
		this.modal = page.locator(".modal-content");
	}

	async goto() {
		await this.page.goto("/kanji");
	}

	async expectVisible() {
		await expect(this.heading).toBeVisible({ timeout: 10000 });
		await expect(this.searchInput).toBeVisible();
		await expect(this.addButton).toBeVisible();
	}

	async expectFiltersVisible() {
		await expect(this.filterAll).toBeVisible();
		await expect(this.filterNew).toBeVisible();
		await expect(this.filterHard).toBeVisible();
		await expect(this.filterInProgress).toBeVisible();
		await expect(this.filterLearned).toBeVisible();
	}

	async search(query: string) {
		await this.searchInput.fill(query);
	}

	async clearSearch() {
		await this.searchInput.clear();
	}

	async clickFilter(filter: "all" | "new" | "hard" | "inProgress" | "learned") {
		const filterMap = {
			all: this.filterAll,
			new: this.filterNew,
			hard: this.filterHard,
			inProgress: this.filterInProgress,
			learned: this.filterLearned,
		};
		await filterMap[filter].click();
	}

	async getCardsCount(): Promise<number> {
		return await this.page.locator(".card").count();
	}

	async hasCards(): Promise<boolean> {
		const count = await this.getCardsCount();
		return count > 0;
	}

	async expectEmptyState() {
		await expect(this.emptyMessage).toBeVisible();
	}

	async expectNotEmpty() {
		const count = await this.getCardsCount();
		expect(count).toBeGreaterThan(0);
	}

	async getCardByText(text: string): Promise<Locator> {
		return this.page.locator(".card").filter({ hasText: text });
	}

	async expectCardVisible(title: string) {
		const card = await this.getCardByText(title);
		await expect(card).toBeVisible();
	}

	async expectCardNotVisible(title: string) {
		const card = await this.getCardByText(title);
		await expect(card).not.toBeVisible();
	}

	async getFilterCount(
		filter: "all" | "new" | "hard" | "inProgress" | "learned",
	): Promise<number> {
		const filterMap = {
			all: this.filterAll,
			new: this.filterNew,
			hard: this.filterHard,
			inProgress: this.filterInProgress,
			learned: this.filterLearned,
		};
		const text = await filterMap[filter].textContent();
		const match = text?.match(/\((\d+)\)/);
		return match ? parseInt(match[1], 10) : 0;
	}

	async expectFilterActive(
		filter: "all" | "new" | "hard" | "inProgress" | "learned",
	) {
		const filterMap = {
			all: this.filterAll,
			new: this.filterNew,
			hard: this.filterHard,
			inProgress: this.filterInProgress,
			learned: this.filterLearned,
		};
		await expect(filterMap[filter]).toHaveClass(/tag-filled/);
	}

	async expectModalVisible() {
		await expect(this.modal).toBeVisible();
	}

	async expectModalNotVisible() {
		await expect(this.modal).not.toBeVisible();
	}

	async selectLevel(level: "N5" | "N4" | "N3" | "N2" | "N1") {
		await this.page.getByRole("button", { name: level, exact: true }).click();
	}

	async selectKanji(kanjiChar: string) {
		const kanjiItem = this.modal.locator("div.border").filter({
			has: this.page.locator(`span.text-2xl:has-text("${kanjiChar}")`)
		}).first();
		await kanjiItem.waitFor({ state: "visible", timeout: 3000 });
		await kanjiItem.click();
	}

	async selectFirstKanji(): Promise<boolean> {
		const firstKanji = this.modal.locator("div.border").first();
		const isVisible = await firstKanji.isVisible({ timeout: 2000 }).catch(() => false);
		if (isVisible) {
			await firstKanji.click();
			return true;
		}
		return false;
	}

	async selectMultipleKanji(count: number): Promise<number> {
		const kanjiItems = this.modal.locator("div.border");
		const total = await kanjiItems.count();
		const toSelect = Math.min(count, total);
		
		for (let i = 0; i < toSelect; i++) {
			const item = kanjiItems.nth(i);
			await item.click();
			await this.page.waitForTimeout(100);
		}
		
		return toSelect;
	}

	async hasAvailableKanji(): Promise<boolean> {
		const firstKanji = this.modal.locator("div.border").first();
		return await firstKanji.isVisible({ timeout: 2000 }).catch(() => false);
	}

	async confirmAdd() {
		await this.modal.getByRole("button", { name: "Добавить" }).click();
	}

	async cancelAdd() {
		await this.modal.getByRole("button", { name: "Отмена" }).click();
	}
}
