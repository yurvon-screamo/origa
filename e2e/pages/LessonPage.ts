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
		// Progress can be shown as "Прогресс" text with a fraction like "1/2"
		this.progressBar = page.locator("text=Прогресс");
		this.progressText = page.locator("text=/\\d+\\/\\d+/");
		// Card type can be "Слово", "Кандзи", etc.
		this.cardType = page.locator("text=/^(Слово|Кандзи|Грамматика)$/");
		// Question is the main card heading (can be h1, h2, or h3)
		this.question = page.locator("h1, h2, h3").first();
		this.showAnswerButton = page.getByRole("button", { name: "Показать ответ" });
		this.answerSection = page.locator("text=Ответ:");
		this.ratingButtons = page.locator(".grid.grid-cols-4");
		this.againButton = page.getByRole("button", { name: "Не знаю", exact: true });
		this.hardButton = page.getByRole("button", { name: "Плохо", exact: true });
		this.goodButton = page.getByRole("button", { name: "Знаю", exact: true });
		this.easyButton = page.getByRole("button", { name: "Идеально", exact: true });
		this.completeScreen = page.locator("text=Урок завершён!");
		this.completeTitle = page.locator("text=Урок завершён!");
		this.homeButton = page.getByRole("button", { name: "На главную" });
		this.loadingText = page.locator("text=Загрузка урока...");
		this.emptyStateText = page.locator("text=Нет карточек для изучения");
	}

	async goto() {
		await this.page.goto("/lesson");
	}

	async gotoFixation() {
		await this.page.goto("/lesson?mode=fixation");
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
		// Progress can be shown as "Прогресс" text or as a progress bar
		const hasProgressText = await this.progressBar.isVisible().catch(() => false);
		const hasProgressFraction = await this.progressText.isVisible().catch(() => false);
		expect(hasProgressText || hasProgressFraction).toBe(true);
	}

	async expectCardVisible() {
		// Question and show answer button should be visible
		await expect(this.question).toBeVisible();
		await expect(this.showAnswerButton).toBeVisible();
	}

	async showAnswer() {
		// Check if answer is already shown
		const answerVisible = await this.answerSection.isVisible().catch(() => false);
		if (!answerVisible) {
			await this.showAnswerButton.click();
			await expect(this.answerSection).toBeVisible();
		}
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
