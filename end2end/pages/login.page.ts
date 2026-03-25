import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class LoginPage extends BasePage {
	readonly emailInput: Locator;
	readonly passwordInput: Locator;
	readonly submitButton: Locator;
	readonly errorAlert: Locator;
	readonly oauthButtons: Locator;

	constructor(page: Page) {
		super(page);

		this.emailInput = page.getByTestId("email-input");
		this.passwordInput = page.getByTestId("password-input");
		this.submitButton = page.getByTestId("login-submit");
		this.errorAlert = page.locator('[role="alert"], .error');
		this.oauthButtons = page.locator("[data-oauth-provider]");
	}

	async goto(): Promise<void> {
		await this.navigate("/login");
	}

	async expectLoginFormVisible(): Promise<void> {
		await expect(this.emailInput).toBeVisible();
		await expect(this.passwordInput).toBeVisible();
		await expect(this.submitButton).toBeVisible();
	}

	async fillEmail(email: string): Promise<void> {
		await this.emailInput.waitFor({ state: "visible", timeout: 5000 });
		await this.emailInput.click({ force: true });
		await this.emailInput.fill(email, { force: true });
	}

	async fillPassword(password: string): Promise<void> {
		await this.passwordInput.waitFor({ state: "visible", timeout: 5000 });
		await this.passwordInput.click({ force: true });
		await this.passwordInput.fill(password, { force: true });
	}

	async submit(): Promise<void> {
		await this.submitButton.waitFor({ state: "visible", timeout: 5000 });
		await this.submitButton.click({ force: true });
	}

	async login(email: string, password: string): Promise<void> {
		await this.fillEmail(email);
		await this.page.waitForTimeout(100);
		await this.fillPassword(password);
		await this.page.waitForTimeout(100);
		await this.submit();
	}

	async expectLoginSuccess(redirectTo = "/home"): Promise<void> {
		await this.page.waitForURL(`**${redirectTo}**`);
	}

	async expectErrorMessage(): Promise<string | null> {
		await expect(this.errorAlert).toBeVisible();
		return await this.errorAlert.textContent();
	}
}
