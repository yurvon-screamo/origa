import { expect, type Locator, type Page } from "@playwright/test";

export class GrammarPage {
	readonly page: Page;
	readonly heading: Locator;
	readonly searchInput: Locator;
	readonly addButton: Locator;
	readonly backButton: Locator;
	readonly filterAll: Locator;
	readonly filterNew: Locator;
	readonly filterHard: Locator;
	readonly filterInProgress: Locator;
	readonly filterLearned: Locator;
	readonly grammarCards: Locator;
	readonly emptyMessage: Locator;
	readonly modal: Locator;

	constructor(page: Page) {
		this.page = page;
		this.heading = page.getByRole("heading", { name: "Грамматика" });
		this.searchInput = page.getByPlaceholder("Поиск...");
		this.addButton = page.getByRole("button", { name: "+" });
		this.backButton = page.getByRole("button", { name: "Назад" });
		this.filterAll = page.getByRole("button", { name: /Все/ });
		this.filterNew = page.getByRole("button", { name: /Новые/ });
		this.filterHard = page.getByRole("button", { name: /Сложные/ });
		this.filterInProgress = page.getByRole("button", { name: /В процессе/ });
		this.filterLearned = page.getByRole("button", { name: /Изученные/ });
		this.grammarCards = page.locator(".card");
		this.emptyMessage = page.getByText("Грамматических конструкций не найдено");
		this.modal = page.locator(".modal-content");
	}

	async goto() {
		await this.page.goto("/grammar");
	}

	async expectVisible() {
		await expect(this.heading).toBeVisible();
		await expect(this.searchInput).toBeVisible();
		await expect(this.addButton).toBeVisible();
		await expect(this.backButton).toBeVisible();
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

	async goBack() {
		await this.backButton.click();
		await this.page.waitForURL("/home");
	}

	async clickAddButton() {
		await this.addButton.click();
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

	async selectRule(ruleTitle: string) {
		await this.page
			.locator(".modal-content")
			.getByText(ruleTitle, { exact: false })
			.first()
			.click();
	}

	async confirmAdd() {
		await this.modal.getByRole("button", { name: "Добавить" }).click();
	}

	async cancelAdd() {
		await this.modal.getByRole("button", { name: "Отмена" }).click();
	}
}
