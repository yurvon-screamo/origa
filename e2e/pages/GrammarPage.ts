import { Page, Locator, expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class GrammarPage extends BasePage {
  readonly levelSelector: Locator;
  readonly rulesList: Locator;
  readonly ruleItems: Locator;
  readonly addRuleButton: Locator;
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
    this.rulesList = page.locator('.rules-list, [data-testid="rules-list"], .grammar-list');
    this.ruleItems = page.locator('.rule-item, [data-testid="rule-item"], .grammar-card');
    this.addRuleButton = page.getByRole('button', { name: /добавить грамматику|добавить правило/i });
    this.searchInput = page.locator('input[type="text"], input:not([type])').first();
    this.selectedCount = page.locator('.selected-count, [data-testid="selected-count"]');
    this.addToCardsButton = page.getByRole('button', { name: /добавить в карточки|добавить/i });
    this.modal = page.locator('.modal, [role="dialog"]');
    this.modalInput = this.modal.locator('input, textarea').first();
    this.modalSubmit = this.modal.getByRole('button', { name: /добавить|сохранить/i });
    this.modalClose = this.modal.getByRole('button', { name: /закрыть|отмена/i });
  }

  async goto() {
    await super.goto('/grammar');
    await this.page.waitForTimeout(1000);
  }

  async waitForRules() {
    await this.rulesList.waitFor({ state: 'visible', timeout: 30000 }).catch(() => {});
  }

  async selectLevel(level: 'N5' | 'N4' | 'N3' | 'N2' | 'N1') {
    await this.page.getByRole('button', { name: level }).click();
    await this.page.waitForTimeout(500);
  }

  async clickRule(index: number) {
    const items = await this.ruleItems.all();
    if (items[index]) {
      await items[index].click();
    }
  }

  async selectRules(indexes: number[]) {
    for (const index of indexes) {
      await this.clickRule(index);
      await this.page.waitForTimeout(200);
    }
  }

  async openAddModal() {
    await this.addRuleButton.click();
    await expect(this.modal).toBeVisible({ timeout: 30000 });
  }

  async addRule(pattern: string, meaning: string) {
    await this.openAddModal();
    const inputs = await this.modal.locator('input, textarea').all();
    if (inputs[0]) await inputs[0].fill(pattern);
    if (inputs[1]) await inputs[1].fill(meaning);
    await this.modalSubmit.click();
  }

  async closeModal() {
    await this.modalClose.click();
  }

  async addToCards() {
    await this.addToCardsButton.click();
  }

  async expectRulesVisible() {
    await expect(this.rulesList).toBeVisible({ timeout: 30000 });
  }

  async expectSelectedCount(count: number) {
    await expect(this.selectedCount).toContainText(count.toString());
  }

  async getRuleCount(): Promise<number> {
    return await this.ruleItems.count();
  }
}
