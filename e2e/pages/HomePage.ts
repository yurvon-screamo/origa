import { Page, Locator, expect } from '@playwright/test';

export class HomePage {
  readonly page: Page;
  readonly kanjiCard: Locator;
  readonly wordsCard: Locator;
  readonly levelCard: Locator;
  readonly todaySection: Locator;
  readonly todayCard: Locator;

  constructor(page: Page) {
    this.page = page;
    this.kanjiCard = page.getByText('Канжи').locator('..').locator('..');
    this.wordsCard = page.getByText('Слова').locator('..').locator('..');
    this.levelCard = page.getByText('Уровень').locator('..').locator('..');
    this.todaySection = page.getByText('Сегодня');
    this.todayCard = page.getByText('Начните изучение японского языка');
  }

  async expectVisible() {
    await expect(this.kanjiCard).toBeVisible();
    await expect(this.wordsCard).toBeVisible();
    await expect(this.levelCard).toBeVisible();
    await expect(this.todaySection).toBeVisible();
    await expect(this.todayCard).toBeVisible();
  }

  async getKanjiCount(): Promise<string> {
    const card = this.kanjiCard;
    const text = await card.textContent();
    return text?.match(/[\d,]+/)?.[0] || '';
  }

  async getWordsCount(): Promise<string> {
    const card = this.wordsCard;
    const text = await card.textContent();
    return text?.match(/[\d,]+/)?.[0] || '';
  }

  async getLevel(): Promise<string> {
    const card = this.levelCard;
    const text = await card.textContent();
    return text?.match(/N\d+/)?.[0] || '';
  }
}
