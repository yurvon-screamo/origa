import { Page, Locator, expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class HomePage extends BasePage {
  readonly statsSection: Locator;
  readonly lessonButton: Locator;
  readonly fixationButton: Locator;
  readonly jlptProgress: Locator;
  readonly statCards: Locator;
  readonly totalCardsStat: Locator;
  readonly learnedStat: Locator;
  readonly inProgressStat: Locator;
  readonly newCardsStat: Locator;

  constructor(page: Page) {
    super(page);
    this.statsSection = page.getByText(/статистика/i);
    this.lessonButton = page.getByRole('link', { name: /урок/i });
    this.fixationButton = page.getByRole('link', { name: /сложные/i });
    this.jlptProgress = page.locator('.jlpt-progress, [data-testid="jlpt-progress"]').first();
    this.statCards = page.locator('.stat-card, [data-testid="stat-card"]');
    this.totalCardsStat = page.getByText(/всего карточек/i).first();
    this.learnedStat = page.getByText(/изучено/i).first();
    this.inProgressStat = page.getByText(/в процессе/i).first();
    this.newCardsStat = page.getByText(/новых/i).first();
  }

  async goto() {
    await super.goto('/home');
    await this.waitForReady();
  }

  async waitForStats() {
    await this.statsSection.waitFor({ state: 'visible', timeout: 60000 }).catch(() => {});
    await super.waitForLoading();
  }

  async startLesson() {
    await this.lessonButton.click();
    await this.waitForUrl(/\/lesson/);
  }

  async startFixation() {
    await this.fixationButton.click();
    await this.waitForUrl(/\/lesson.*mode=fixation/);
  }

  async expectStatsVisible() {
    await expect(this.statsSection).toBeVisible({ timeout: 30000 });
    await expect(this.statCards.first()).toBeVisible({ timeout: 30000 });
  }

  async navigateTo(route: string) {
    await this.page.goto(route);
    await this.waitForAppReady();
  }

  async expectWelcomeContent() {
    await expect(this.page.locator('body')).toContainText(/прогресс|карточек|изучено/i);
  }
}
