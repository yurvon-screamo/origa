import { expect, type Locator, type Page } from "@playwright/test";

export class HomePage {
	readonly page: Page;
	readonly greeting: Locator;
	readonly totalCardsCard: Locator;
	readonly learnedCard: Locator;
	readonly inProgressCard: Locator;
	readonly newCard: Locator;
	readonly highDifficultyCard: Locator;
	readonly startLessonButton: Locator;
	readonly fixationButton: Locator;
	readonly tabBar: Locator;
	readonly tabBarHome: Locator;
	readonly tabBarWords: Locator;
	readonly tabBarKanji: Locator;
	readonly tabBarSets: Locator;
	readonly tabBarGrammar: Locator;
	readonly profileButton: Locator;

	constructor(page: Page) {
		this.page = page;
		this.greeting = page.locator("text=/Привет,/");
		this.totalCardsCard = page.locator(".card").filter({ hasText: "Total Cards" });
		this.learnedCard = page.locator(".card").filter({ hasText: "Learned" });
		this.inProgressCard = page.locator(".card").filter({ hasText: "In Progress" });
		this.newCard = page.locator(".card").filter({ hasText: "New" });
		this.highDifficultyCard = page.locator(".card").filter({ hasText: "Сложные слова" });
		this.startLessonButton = page.getByRole("link", { name: /Начать урок/ }).or(
			page.getByRole("button", { name: /Начать урок/ })
		);
		this.fixationButton = page.getByRole("link", { name: /Закрепление/ }).or(
			page.getByRole("button", { name: /Закрепление/ })
		);
		this.tabBar = page.locator("nav").filter({ has: page.getByRole("button") });
		this.tabBarHome = page.getByRole("button", { name: "Главная" });
		this.tabBarWords = page.getByRole("button", { name: "Слова" });
		this.tabBarKanji = page.getByRole("button", { name: "Кандзи" });
		this.tabBarSets = page.getByRole("button", { name: "Наборы" });
		this.tabBarGrammar = page.getByRole("button", { name: "Грамматика" });
		this.profileButton = page.locator(".avatar").or(page.getByRole("button").filter({ hasText: /[A-ZА-Я]/ }));
	}

	async goto() {
		await this.page.goto("/home");
	}

	async expectVisible() {
		await expect(this.totalCardsCard).toBeVisible({ timeout: 10000 });
		await expect(this.startLessonButton).toBeVisible({ timeout: 5000 });
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

	async navigateToWords() {
		await this.tabBarWords.click();
		await this.page.waitForURL("/words");
	}

	async navigateToKanji() {
		await this.tabBarKanji.click();
		await this.page.waitForURL("/kanji");
	}

	async navigateToSets() {
		await this.tabBarSets.click();
		await this.page.waitForURL("/sets");
	}

	async navigateToGrammar() {
		await this.tabBarGrammar.click();
		await this.page.waitForURL("/grammar");
	}

	async startLesson() {
		await this.startLessonButton.click();
	}

	async startFixation() {
		await this.fixationButton.click();
	}

	async openProfile() {
		await this.profileButton.click();
	}

	async hasFixationSection(): Promise<boolean> {
		return await this.fixationButton.isVisible({ timeout: 2000 }).catch(() => false);
	}
}
