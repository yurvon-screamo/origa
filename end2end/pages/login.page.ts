import { Page, Locator, expect } from '@playwright/test';
import { BasePage } from './base.page';

/**
 * Login page object
 * Handles email/password authentication and OAuth flows
 */
export class LoginPage extends BasePage {
  // Locators
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly submitButton: Locator;
  readonly errorAlert: Locator;
  readonly oauthButtons: Locator;

  constructor(page: Page) {
    super(page);
    
    this.emailInput = page.getByTestId('email-input');
    this.passwordInput = page.getByTestId('password-input');
    this.submitButton = page.getByTestId('login-submit');
    this.errorAlert = page.locator('[role="alert"], .error');
    this.oauthButtons = page.locator('[data-oauth-provider]');
  }

  /**
   * Navigate to login page
   */
  async goto(): Promise<void> {
    await this.navigate('/login');
  }

  /**
   * Check if login form is visible
   */
  async expectLoginFormVisible(): Promise<void> {
    await expect(this.emailInput).toBeVisible();
    await expect(this.passwordInput).toBeVisible();
    await expect(this.submitButton).toBeVisible();
  }

  /**
   * Fill email field
   */
  async fillEmail(email: string): Promise<void> {
    await this.emailInput.fill(email, { force: true });
  }

  async fillPassword(password: string): Promise<void> {
    await this.passwordInput.fill(password, { force: true });
  }

  async submit(): Promise<void> {
    await this.submitButton.click({ force: true });
  }

  /**
   * Perform login with email and password
   */
  async login(email: string, password: string): Promise<void> {
    await this.fillEmail(email);
    await this.fillPassword(password);
    await this.submit();
  }

  /**
   * Expect login to succeed and redirect
   */
  async expectLoginSuccess(redirectTo = '/home'): Promise<void> {
    await this.page.waitForURL(`**${redirectTo}**`);
  }

  /**
   * Expect error message to be visible
   */
  async expectErrorMessage(): Promise<string | null> {
    await expect(this.errorAlert).toBeVisible();
    return await this.errorAlert.textContent();
  }
}