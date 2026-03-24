import { Page, Locator } from '@playwright/test';

/**
 * Base page object with common methods
 */
export abstract class BasePage {
  constructor(protected readonly page: Page) {}

  /**
   * Navigate to a path relative to baseURL
   */
  async navigate(path: string): Promise<void> {
    await this.page.goto(path);
    await this.waitForLoad();
  }

  /**
   * Wait for page to be fully loaded
   * For Leptos CSR apps, we wait for hydration
   */
  async waitForLoad(): Promise<void> {
    await this.page.waitForLoadState('domcontentloaded');
    
    await this.page.waitForFunction(
      () => {
        const overlay = document.querySelector('.loading-overlay');
        if (!overlay) return true;
        const rect = overlay.getBoundingClientRect();
        return rect.width === 0 || rect.height === 0;
      },
      { timeout: 60000 }
    );
  }

  /**
   * Wait for a specific element to be visible
   */
  async waitForElement(locator: Locator, timeout = 5000): Promise<void> {
    await locator.waitFor({ state: 'visible', timeout });
  }

  /**
   * Take a screenshot for debugging
   */
  async screenshot(name: string): Promise<void> {
    await this.page.screenshot({ path: `test-results/${name}.png` });
  }

  /**
   * Get current URL path
   */
  getCurrentPath(): string {
    const url = new URL(this.page.url());
    return url.pathname;
  }

  /**
   * Check if element is visible
   */
  async isVisible(locator: Locator): Promise<boolean> {
    try {
      await locator.waitFor({ state: 'visible', timeout: 2000 });
      return true;
    } catch {
      return false;
    }
  }
}