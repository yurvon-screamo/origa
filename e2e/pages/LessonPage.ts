import { expect, Locator, Page } from "@playwright/test";

export class LessonPage {
	readonly page: Page;
	readonly header: Locator;
	readonly backButton: Locator;
	readonly progressBar: Locator;
	readonly progressText: Locator;
	readonly cardType: Locator;
	readonly question: Locator;
	readonly showAnswerButton: Locator;
	readonly answerSection: Locator;
	readonly ratingButtons: Locator;
	readonly againButton: Locator;
	readonly hardButton: Locator;
	readonly goodButton: Locator;
	readonly easyButton: Locator;
	readonly completeScreen: Locator;
	readonly completeTitle: Locator;
	readonly homeButton: Locator;
	readonly loadingText: Locator;
	readonly emptyStateText: Locator;

	constructor(page: Page) {
		this.page = page;
		this.header = page.locator("h1", { hasText: "Урок" });
		this.backButton = page.getByRole("button", { name: "Назад" });
		this.progressBar = page.locator(".progress-track");
		this.progressText = page.locator(".progress-track").locator("..").locator("span").last();
		this.cardType = page.locator(".tag");
		this.question = page.locator("h2, h3").first();
		this.showAnswerButton = page.getByRole("button", { name: "Показать ответ" });
		this.answerSection = page.locator("text=Ответ:");
		this.ratingButtons = page.locator(".grid.grid-cols-4");
		this.againButton = page.getByRole("button", { name: "Не знаю" });
		this.hardButton = page.getByRole("button", { name: "Плохо" });
		this.goodButton = page.getByRole("button", { name: "Знаю" });
		this.easyButton = page.getByRole("button", { name: "Идеально" });
		this.completeScreen = page.locator("text=Урок завершён!");
		this.completeTitle = page.locator("text=Урок завершён!");
		this.homeButton = page.getByRole("button", { name: "На главную" });
		this.loadingText = page.locator("text=Загрузка урока...");
		this.emptyStateText = page.locator("text=Нет карточек для изучения");
	}

	async goto() {
		await this.page.goto("/lesson");
	}

	async expectVisible() {
		await expect(this.header).toBeVisible();
	}

	async expectLoading() {
		await expect(this.loadingText).toBeVisible();
	}

	async expectEmptyState() {
		await expect(this.emptyStateText).toBeVisible();
	}

	async expectProgressBarVisible() {
		await expect(this.progressBar).toBeVisible();
	}

	async expectCardVisible() {
		await expect(this.cardType).toBeVisible();
		await expect(this.question).toBeVisible();
		await expect(this.showAnswerButton).toBeVisible();
	}

	async showAnswer() {
		await this.showAnswerButton.click();
		await expect(this.answerSection).toBeVisible();
	}

	async expectRatingButtonsVisible() {
		await expect(this.againButton).toBeVisible();
		await expect(this.hardButton).toBeVisible();
		await expect(this.goodButton).toBeVisible();
		await expect(this.easyButton).toBeVisible();
	}

	async rateAgain() {
		await this.againButton.click();
	}

	async rateHard() {
		await this.hardButton.click();
	}

	async rateGood() {
		await this.goodButton.click();
	}

	async rateEasy() {
		await this.easyButton.click();
	}

	async expectCompleteScreen() {
		await expect(this.completeTitle).toBeVisible();
		await expect(this.homeButton).toBeVisible();
	}

	async goHome() {
		await this.homeButton.click();
	}

	async goBack() {
		await this.backButton.click();
	}

	async getProgressText(): Promise<string> {
		return await this.progressText.textContent() || "";
	}
}
