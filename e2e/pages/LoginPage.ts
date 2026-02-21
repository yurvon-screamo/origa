import { Page, Locator, expect } from '@playwright/test';

export class LoginPage {
  readonly page: Page;
  readonly usernameInput: Locator;
  readonly loginButton: Locator;
  readonly errorMessage: Locator;

  constructor(page: Page) {
    this.page = page;
    this.usernameInput = page.getByPlaceholder('Введите имя');
    this.loginButton = page.getByRole('button', { name: 'Войти' });
    this.errorMessage = page.getByText(/Ошибка:|Введите имя пользователя/);
  }

  async goto() {
    await this.page.goto('/');
  }

  async expectVisible() {
    await expect(this.usernameInput).toBeVisible();
    await expect(this.loginButton).toBeVisible();
  }

  async login(username: string) {
    await this.usernameInput.fill(username);
    await this.loginButton.click();
  }

  async loginAndNavigate(username: string) {
    await this.login(username);
    await this.page.waitForURL('/home');
  }

  async expectErrorMessage(message: string) {
    await expect(this.errorMessage).toContainText(message);
  }

  async expectNoError() {
    await expect(this.errorMessage).not.toBeVisible();
  }
}
