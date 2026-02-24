import { expect, type Locator, type Page } from "@playwright/test";

export class HomePage {
	readonly page: Page;
	readonly kanjiCard: Locator;
	readonly wordsCard: Locator;
	readonly levelCard: Locator;
	readonly todaySection: Locator;
	readonly todayCard: Locator;
	readonly startLessonButton: Locator;
	readonly fixationSection: Locator;
	readonly fixationButton: Locator;

	constructor(page: Page) {
		this.page = page;
		this.kanjiCard = page.locator(".card").filter({ hasText: "Канжи" });
		this.wordsCard = page.locator(".card").filter({ hasText: "в словаре" });
		this.levelCard = page.locator(".card").filter({ hasText: "Уровень" });
		this.todaySection = page.getByText("Сегодня");
		this.todayCard = page.getByText("Начните изучение японского языка");
		this.startLessonButton = page.getByRole("button", { name: /Начать урок|Урок/ });
		this.fixationSection = page.locator("text=/Закрепление|Сложные/i");
		this.fixationButton = page.getByRole("button", { name: /Закрепление|Закрепить/ });
	}

	async goto() {
		await this.page.goto("/home");
	}

	async expectVisible() {
		await expect(this.kanjiCard).toBeVisible();
		await expect(this.wordsCard).toBeVisible();
		await expect(this.levelCard).toBeVisible();
	}

	async getKanjiCount(): Promise<string> {
		const card = this.kanjiCard;
		const text = await card.textContent();
		return text?.match(/[\d,]+/)?.[0] || "";
	}

	async getWordsCount(): Promise<string> {
		const card = this.wordsCard;
		const text = await card.textContent();
		return text?.match(/[\d,]+/)?.[0] || "";
	}

	async getLevel(): Promise<string> {
		const card = this.levelCard;
		const text = await card.textContent();
		return text?.match(/N\d+/)?.[0] || "";
	}

	async hasFixationSection(): Promise<boolean> {
		return await this.fixationSection.isVisible({ timeout: 2000 }).catch(() => false);
	}

	async startFixation() {
		await this.fixationButton.click();
	}

	async startLesson() {
		await this.startLessonButton.click();
	}
}
