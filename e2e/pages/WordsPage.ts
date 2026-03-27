import { Page, Locator, expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class WordsPage extends BasePage {
  readonly searchInput: Locator;
  readonly wordsList: Locator;
  readonly wordItems: Locator;
  readonly addWordButton: Locator;
  readonly filterButtons: Locator;
  readonly modal: Locator;
  readonly modalInput: Locator;
  readonly modalSubmit: Locator;
  readonly modalClose: Locator;
  readonly ocrButton: Locator;
  readonly favoriteButtons: Locator;

  constructor(page: Page) {
    super(page);
    this.searchInput = page.locator('input[type="text"], input:not([type])').first();
    this.wordsList = page.locator('.words-list, [data-testid="words-list"], .vocabulary-list');
    this.wordItems = page.locator('.word-item, [data-testid="word-item"], .vocabulary-card');
    this.addWordButton = page.getByRole('button', { name: /добавить слово/i });
    this.filterButtons = page.locator('.filter-button, .tag, [data-testid="filter"]');
    this.modal = page.locator('.modal, [role="dialog"]');
    this.modalInput = this.modal.locator('input, textarea').first();
    this.modalSubmit = this.modal.getByRole('button', { name: /добавить|сохранить/i });
    this.modalClose = this.modal.getByRole('button', { name: /закрыть|отмена/i });
    this.ocrButton = page.getByRole('button', { name: /ocr|распознать|камера/i });
    this.favoriteButtons = page.locator('.favorite-button, [data-testid="favorite-button"]');
  }

  async goto() {
    await super.goto('/words');
    await this.page.waitForTimeout(1000);
  }

  async waitForWords() {
    await this.wordsList.waitFor({ state: 'visible', timeout: 30000 }).catch(() => {});
  }

  async search(query: string) {
    await this.searchInput.fill(query);
    await this.page.keyboard.press('Enter');
    await this.page.waitForTimeout(500);
  }

  async openAddModal() {
    await this.addWordButton.click();
    await expect(this.modal).toBeVisible({ timeout: 30000 });
  }

  async addWord(word: string, translation: string) {
    await this.openAddModal();
    const inputs = await this.modal.locator('input, textarea').all();
    if (inputs[0]) await inputs[0].fill(word);
    if (inputs[1]) await inputs[1].fill(translation);
    await this.modalSubmit.click();
  }

  async closeModal() {
    await this.modalClose.click();
  }

  async clickWord(index: number) {
    const items = await this.wordItems.all();
    if (items[index]) {
      await items[index].click();
    }
  }

  async toggleFavorite(index: number) {
    const buttons = await this.favoriteButtons.all();
    if (buttons[index]) {
      await buttons[index].click();
    }
  }

  async expectWordsVisible() {
    await expect(this.wordsList).toBeVisible({ timeout: 30000 });
  }

  async getWordCount(): Promise<number> {
    return await this.wordItems.count();
  }

  async applyFilter(filterName: string) {
    await this.page.getByRole('button', { name: filterName }).click();
    await this.page.waitForTimeout(500);
  }
}
