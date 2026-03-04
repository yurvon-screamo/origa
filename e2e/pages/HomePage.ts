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
    this.totalCardsCard = page
      .locator(".card")
      .filter({ hasText: "Всего карточек" });
    this.learnedCard = page.locator(".card").filter({ hasText: "Изучено" });
    this.inProgressCard = page
      .locator(".card")
      .filter({ hasText: "В процессе" });
    this.newCard = page.locator(".card").filter({ hasText: "Новые" });
    this.highDifficultyCard = page
      .locator(".card")
      .filter({ hasText: "Сложные" });
    this.startLessonButton = page
      .getByRole("link", { name: /Начать урок/ })
      .or(page.getByRole("button", { name: /Начать урок/ }))
      .first();
    this.fixationButton = page
      .getByRole("link", { name: /Закрепление/ })
      .or(page.getByRole("button", { name: /Закрепление/ }))
      .first();
    this.tabBar = page.locator("nav").filter({ has: page.getByRole("button") });
    this.tabBarHome = this.tabBar.getByRole("button", { name: "Главная" });
    this.tabBarWords = this.tabBar.getByRole("button", { name: "Слова" });
    this.tabBarKanji = this.tabBar.getByRole("button", { name: /Кандзи/ });
    this.tabBarSets = this.tabBar.getByRole("button", { name: "Наборы" });
    this.tabBarGrammar = this.tabBar.getByRole("button", {
      name: "Грамматика",
    });
    this.profileButton = page
      .locator(".avatar")
      .or(page.getByRole("button").filter({ hasText: /[A-ZА-Я]/ }));
  }

  async goto() {
    await this.page.goto("/home");
  }

  async expectVisible() {
    await expect(this.totalCardsCard).toBeVisible({ timeout: 20000 });
    await expect(this.startLessonButton).toBeVisible({ timeout: 10000 });
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
    await this.page.goto("/words");
  }

  async navigateToKanji() {
    await this.page.goto("/kanji");
  }

  async navigateToSets() {
    await this.page.goto("/sets");
  }

  async navigateToGrammar() {
    await this.page.goto("/grammar");
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
    return await this.fixationButton
      .isVisible({ timeout: 2000 })
      .catch(() => false);
  }
}
