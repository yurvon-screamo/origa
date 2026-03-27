import { Page, Locator, expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class KanjiPage extends BasePage {
  readonly levelSelector: Locator;
  readonly kanjiList: Locator;
  readonly kanjiItems: Locator;
  readonly addKanjiButton: Locator;
  readonly searchInput: Locator;
  readonly selectedCount: Locator;
  readonly addToCardsButton: Locator;
  readonly modal: Locator;
  readonly modalInput: Locator;
  readonly modalSubmit: Locator;
  readonly modalClose: Locator;

  constructor(page: Page) {
    super(page);
    this.levelSelector = page.locator('.level-selector, [data-testid="level-selector"]');
    this.kanjiList = page.locator('.kanji-list, [data-testid="kanji-list"]');
    this.kanjiItems = page.locator('.kanji-item, [data-testid="kanji-item"]');
    this.addKanjiButton = page.getByRole('button', { name: /добавить кандзи/i });
    this.searchInput = page.locator('input[type="text"], input:not([type])').first();
    this.selectedCount = page.locator('.selected-count, [data-testid="selected-count"]');
    this.addToCardsButton = page.getByRole('button', { name: /добавить в карточки|добавить/i });
    this.modal = page.locator('.modal, [role="dialog"]');
    this.modalInput = this.modal.locator('input').first();
    this.modalSubmit = this.modal.getByRole('button', { name: /добавить|сохранить/i });
    this.modalClose = this.modal.getByRole('button', { name: /закрыть|отмена/i });
  }

  async goto() {
    await super.goto('/kanji');
    await this.page.waitForTimeout(1000);
  }

  async waitForKanjiList() {
    await this.kanjiList.waitFor({ state: 'visible', timeout: 30000 }).catch(() => {});
  }

  async selectLevel(level: 'N5' | 'N4' | 'N3' | 'N2' | 'N1') {
    await this.page.getByRole('button', { name: level }).click();
    await this.page.waitForTimeout(500);
  }

  async clickKanji(index: number) {
    const items = await this.kanjiItems.all();
    if (items[index]) {
      await items[index].click();
    }
  }

  async selectKanji(indexes: number[]) {
    for (const index of indexes) {
      await this.clickKanji(index);
      await this.page.waitForTimeout(200);
    }
  }

  async openAddModal() {
    await this.addKanjiButton.click();
    await expect(this.modal).toBeVisible({ timeout: 30000 });
  }

  async addKanjiInModal(kanji: string) {
    await this.modalInput.fill(kanji);
    await this.modalSubmit.click();
  }

  async closeModal() {
    await this.modalClose.click();
  }

  async addToCards() {
    await this.addToCardsButton.click();
  }

  async expectKanjiListVisible() {
    await expect(this.kanjiList).toBeVisible({ timeout: 30000 });
  }

  async expectSelectedCount(count: number) {
    await expect(this.selectedCount).toContainText(count.toString());
  }

  async getKanjiCount(): Promise<number> {
    return await this.kanjiItems.count();
  }
}
