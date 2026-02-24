import { expect, type Locator, type Page } from "@playwright/test";

export class HomePage {
	readonly page: Page;
	readonly avatar: Locator;
	readonly greeting: Locator;
	readonly totalCardsCard: Locator;
	readonly learnedCard: Locator;
	readonly inProgressCard: Locator;
	readonly newCard: Locator;
	readonly highDifficultyCard: Locator;
	readonly historyButton: Locator;
	readonly startLessonButton: Locator;
	readonly fixationButton: Locator;
	readonly tabBar: Locator;
	readonly tabBarHome: Locator;
	readonly tabBarWords: Locator;
	readonly tabBarKanji: Locator;
	readonly tabBarGrammar: Locator;

	constructor(page: Page) {
		this.page = page;
		this.avatar = page.locator(".avatar");
		this.greeting = page.locator("text=/Привет,/");
		this.totalCardsCard = page.locator(".card").filter({ hasText: "Total Cards" });
		this.learnedCard = page.locator(".card").filter({ hasText: "Learned" });
		this.inProgressCard = page.locator(".card").filter({ hasText: "In Progress" });
		this.newCard = page.locator(".card").filter({ hasText: "New" });
		this.highDifficultyCard = page.locator(".card").filter({ hasText: "Сложные слова" });
		this.historyButton = page.getByRole("button", { name: "История" }).first();
		this.startLessonButton = page.getByRole("button", { name: /Начать урок|Урок/ });
		this.fixationButton = page.getByRole("button", { name: /Закрепление|Закрепить/ });
		this.tabBar = page.locator(".tab-bar, nav[role='tablist'], [data-testid='tab-bar']").first();
		this.tabBarHome = page.getByRole("button", { name: "Главная" }).or(page.getByText("Главная"));
		this.tabBarWords = page.getByRole("button", { name: "Слова" }).or(page.getByText("Слова"));
		this.tabBarKanji = page.getByRole("button", { name: "Кандзи" }).or(page.getByText("Кандзи"));
		this.tabBarGrammar = page.getByRole("button", { name: "Грамматика" }).or(page.getByText("Грамматика"));
	}

	async goto() {
		await this.page.goto("/home");
	}

	async expectVisible() {
		await expect(this.totalCardsCard).toBeVisible();
		await expect(this.learnedCard).toBeVisible();
		await expect(this.inProgressCard).toBeVisible();
		await expect(this.newCard).toBeVisible();
		await expect(this.highDifficultyCard).toBeVisible();
		await expect(this.startLessonButton).toBeVisible();
	}

	async getTotalCards(): Promise<string> {
		const text = await this.totalCardsCard.textContent();
		return text?.match(/[\d,]+/)?.[0] || "";
	}

	async getLearned(): Promise<string> {
		const text = await this.learnedCard.textContent();
		return text?.match(/[\d,]+/)?.[0] || "";
	}

	async getInProgress(): Promise<string> {
		const text = await this.inProgressCard.textContent();
		return text?.match(/[\d,]+/)?.[0] || "";
	}

	async getNew(): Promise<string> {
		const text = await this.newCard.textContent();
		return text?.match(/[\d,]+/)?.[0] || "";
	}

	async getHighDifficulty(): Promise<string> {
		const text = await this.highDifficultyCard.textContent();
		return text?.match(/[\d,]+/)?.[0] || "";
	}

	async clickHistory() {
		await this.historyButton.click();
	}

	async navigateToWords() {
		await this.tabBarWords.click();
	}

	async navigateToKanji() {
		await this.tabBarKanji.click();
	}

	async navigateToGrammar() {
		await this.tabBarGrammar.click();
	}

	async startLesson() {
		await this.startLessonButton.click();
	}

	async startFixation() {
		await this.fixationButton.click();
	}

	async hasFixationSection(): Promise<boolean> {
		return await this.fixationButton.isVisible({ timeout: 2000 }).catch(() => false);
	}
}
