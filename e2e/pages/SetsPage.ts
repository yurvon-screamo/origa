import { Page, Locator, expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class SetsPage extends BasePage {
  readonly searchInput: Locator;
  readonly setCards: Locator;
  readonly levelFilters: Locator;
  readonly typeFilters: Locator;
  readonly importFilters: Locator;
  readonly importButton: Locator;
  readonly previewModal: Locator;
  readonly previewConfirm: Locator;
  readonly previewClose: Locator;

  constructor(page: Page) {
    super(page);
    this.searchInput = page.locator('input[type="text"], input:not([type])').first();
    this.setCards = page.locator('.set-card, [data-testid="set-card"]');
    this.levelFilters = page.locator('.tag, [data-testid="level-filter"]');
    this.typeFilters = page.locator('.tag, [data-testid="type-filter"]');
    this.importFilters = page.locator('.tag, [data-testid="import-filter"]');
    this.importButton = page.getByRole('button', { name: /импортировать|импорт/i });
    this.previewModal = page.locator('.modal, [role="dialog"]');
    this.previewConfirm = this.previewModal.getByRole('button', { name: /импортировать|подтвердить/i });
    this.previewClose = this.previewModal.getByRole('button', { name: /закрыть|отмена/i });
  }

  async goto() {
    await super.goto('/sets');
    await this.page.waitForTimeout(1000);
  }

  async waitForSets() {
    await this.setCards.first().waitFor({ state: 'visible', timeout: 30000 }).catch(() => {});
  }

  async search(query: string) {
    await this.searchInput.fill(query);
    await this.page.waitForTimeout(500);
  }

  async filterByLevel(level: 'Все' | 'N5' | 'N4' | 'N3' | 'N2' | 'N1') {
    await this.page.getByRole('button', { name: level, exact: true }).click();
    await this.page.waitForTimeout(500);
  }

  async filterByType(type: string) {
    await this.page.getByRole('button', { name: type }).click();
    await this.page.waitForTimeout(500);
  }

  async filterByImport(status: 'Все' | 'Импортированные' | 'Новые') {
    await this.page.getByRole('button', { name: status, exact: true }).click();
    await this.page.waitForTimeout(500);
  }

  async openImportModal(setIndex: number) {
    const cards = await this.setCards.all();
    if (cards[setIndex]) {
      await cards[setIndex].getByRole('button', { name: /импортировать|импорт/i }).click();
      await expect(this.previewModal).toBeVisible({ timeout: 30000 });
    }
  }

  async confirmImport() {
    await this.previewConfirm.click();
    await this.page.waitForTimeout(1000);
  }

  async closeModal() {
    await this.previewClose.click();
  }

  async importSet(setIndex: number) {
    await this.openImportModal(setIndex);
    await this.confirmImport();
  }

  async expectSetsVisible() {
    await expect(this.setCards.first()).toBeVisible({ timeout: 30000 });
  }

  async getSetCount(): Promise<number> {
    return await this.setCards.count();
  }

  async getSetTitle(index: number): Promise<string | null> {
    const cards = await this.setCards.all();
    if (cards[index]) {
      return await cards[index].textContent();
    }
    return null;
  }
}
