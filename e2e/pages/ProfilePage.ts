import { Page, Locator, expect } from '@playwright/test';

export class ProfilePage {
  readonly page: Page;
  readonly heading: Locator;
  readonly usernameInput: Locator;
  readonly levelSelector: Locator;
  readonly languageSelector: Locator;
  readonly duolingoTokenInput: Locator;
  readonly remindersToggle: Locator;
  readonly remindersCheckbox: Locator;
  readonly saveButton: Locator;
  readonly logoutButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.heading = page.getByRole('heading', { name: /Профиль/ });
    this.usernameInput = page.getByText('Имя пользователя').locator('..').getByRole('textbox');
    this.levelSelector = page.getByText('Целевой уровень JLPT');
    this.languageSelector = page.getByText('Язык интерфейса');
    this.duolingoTokenInput = page.getByText('Duolingo JWT Token').locator('..').getByRole('textbox');
    this.remindersToggle = page.locator('.toggle-container');
    this.remindersCheckbox = page.locator('.toggle-container input[type="checkbox"]');
    this.saveButton = page.getByRole('button', { name: 'Сохранить изменения' });
    this.logoutButton = page.getByRole('button', { name: 'Выйти из аккаунта' });
  }

  async expectVisible() {
    await expect(this.heading).toBeVisible();
    await expect(this.usernameInput).toBeVisible();
    await expect(this.levelSelector).toBeVisible();
    await expect(this.languageSelector).toBeVisible();
    await expect(this.duolingoTokenInput).toBeVisible();
    await expect(this.remindersToggle).toBeVisible();
    await expect(this.saveButton).toBeVisible();
    await expect(this.logoutButton).toBeVisible();
  }

  async goto() {
    await this.page.goto('/profile');
  }

  async expectUsername(username: string) {
    await expect(this.usernameInput).toHaveValue(username);
  }

  async setDuolingoToken(token: string) {
    await this.duolingoTokenInput.fill(token);
  }

  async expectDuolingoToken(token: string) {
    await expect(this.duolingoTokenInput).toHaveValue(token);
  }

  async toggleReminders() {
    await this.remindersToggle.click();
  }

  async expectRemindersEnabled(enabled: boolean) {
    await expect(this.remindersCheckbox).toBeChecked({ checked: enabled });
  }

  async saveChanges() {
    await this.saveButton.click();

    try {
      await expect(this.saveButton).toHaveText('Сохранение...', { timeout: 1000 });
    } catch (error) {
      console.log('Loading state may be too fast to capture, continuing...');
    }

    await expect(this.saveButton).toHaveText('Сохранить изменения', { timeout: 5000 });
  }

  async logout() {
    await this.logoutButton.click();
    await expect(this.page).toHaveURL('/');
  }

  async expectHeadingContains(username: string) {
    await expect(this.heading).toContainText(username);
  }
}
