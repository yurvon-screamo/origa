import { Page, Locator, expect } from '@playwright/test';

export abstract class BasePage {
  readonly page: Page;
  readonly spinner: Locator;
  readonly loadingOverlay: Locator;

  constructor(page: Page) {
    this.page = page;
    this.spinner = page.locator('.spinner');
    this.loadingOverlay = page.locator('.loading-overlay');
  }

  async goto(path: string = '') {
    await this.page.goto(path, { timeout: 120000 });
    await this.waitForAppReady();
  }

  async waitForAppReady() {
    try {
      await this.page.waitForLoadState('domcontentloaded', { timeout: 30000 });
    } catch (e) {
      // Игнорируем ошибки ожидания - страница может загрузиться без события
    }
    try {
      await this.page.waitForLoadState('networkidle', { timeout: 60000 });
    } catch (e) {
      // Игнорируем ошибки ожидания - страница может загрузиться без networkidle
    }
    await this.page.waitForTimeout(2000);
  }

  async waitForReady() {
    const skeleton = this.page.locator('.skeleton, [data-skeleton], [aria-busy="true"]');
    await skeleton.waitFor({ state: 'hidden', timeout: 60000 }).catch(() => {
      // Игнорируем ошибки ожидания - страница может загрузиться без skeleton
    });
    await this.waitForLoading();
  }

  async waitForLoading() {
    await this.spinner.waitFor({ state: 'hidden', timeout: 60000 }).catch(() => {
      // Игнорируем ошибки ожидания - spinner может отсутствовать
    });
    await this.loadingOverlay.waitFor({ state: 'hidden', timeout: 60000 }).catch(() => {
      // Игнорируем ошибки ожидания - overlay может отсутствовать
    });
  }

  async waitForUrl(pattern: RegExp | string, timeout = 60000) {
    await this.page.waitForURL(pattern, { timeout });
  }

  async clickButton(name: string | RegExp) {
    await this.page.getByRole('button', { name }).click();
  }

  async expectVisible(locator: Locator) {
    await expect(locator).toBeVisible();
  }

  async expectHidden(locator: Locator) {
    await expect(locator).toBeHidden();
  }

  async expectText(text: string | RegExp) {
    await expect(this.page.getByText(text)).toBeVisible();
  }
}
