import { Page, Locator, expect } from '@playwright/test';
import { BasePage } from './BasePage';

export class LoginPage extends BasePage {
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly submitButton: Locator;
  readonly errorAlert: Locator;
  readonly divider: Locator;
  readonly oauthSection: Locator;

  constructor(page: Page) {
    super(page);
    // Email input - пробуем getByLabel, если не работает - используем placeholder
    this.emailInput = page.getByPlaceholder('example@mail.com');
    // Password input - по ID как fallback
    this.passwordInput = page.locator('#password');
    this.submitButton = page.getByRole('button', { name: 'Войти', exact: true });
    this.errorAlert = page.locator('.alert-error, [role="alert"]');
    this.divider = page.locator('hr, .divider').first();
    this.oauthSection = page.getByText(/или войти\/зарегистрироваться через/i);
  }

  async goto() {
    await super.goto('/login');
    await expect(this.emailInput).toBeVisible({ timeout: 60000 });
  }

  async fillEmail(email: string) {
    await this.emailInput.fill(email);
  }

  async fillPassword(password: string) {
    await this.passwordInput.fill(password);
  }

  async submit() {
    await this.submitButton.click();
  }

  async login(email: string, password: string) {
    await this.fillEmail(email);
    await this.fillPassword(password);
    await this.submit();
  }

  async expectErrorVisible() {
    await expect(this.errorAlert).toBeVisible();
  }

  async expectRedirectToHome() {
    await this.waitForUrl(/\/home/, 60000);
  }

  async expectFormVisible() {
    await expect(this.emailInput).toBeVisible();
    await expect(this.passwordInput).toBeVisible();
    await expect(this.submitButton).toBeVisible();
  }
}
