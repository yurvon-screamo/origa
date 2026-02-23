import { expect, type Locator, type Page } from "@playwright/test";

export class LoginPage {
	readonly page: Page;
	readonly emailInput: Locator;
	readonly passwordInput: Locator;
	readonly loginButton: Locator;
	readonly registerButton: Locator;
	readonly switchToRegisterLink: Locator;
	readonly switchToLoginLink: Locator;
	readonly errorMessage: Locator;
	readonly emailConfirmationCard: Locator;
	readonly resendEmailButton: Locator;

	constructor(page: Page) {
		this.page = page;
		this.emailInput = page.getByPlaceholder("email@example.com");
		this.passwordInput = page.getByPlaceholder(/Минимум \d+ символов/);
		this.loginButton = page.getByRole("button", { name: "Войти", exact: true });
		this.registerButton = page.getByRole("button", {
			name: "Зарегистрироваться",
			exact: true,
		});
		this.switchToRegisterLink = page.getByRole("button", {
			name: "Нет аккаунта? Зарегистрироваться",
		});
		this.switchToLoginLink = page.getByRole("button", {
			name: "Уже есть аккаунт? Войти",
		});
		this.errorMessage = page.locator(".alert-error");
		this.emailConfirmationCard = page.locator(".alert-info");
		this.resendEmailButton = page.getByRole("button", {
			name: "Отправить письмо повторно",
		});
	}

	async goto() {
		await this.page.goto("/");
	}

	async expectVisible() {
		await expect(this.emailInput).toBeVisible();
		await expect(this.passwordInput).toBeVisible();
		await expect(this.loginButton).toBeVisible();
	}

	async expectRegisterFormVisible() {
		await expect(this.emailInput).toBeVisible();
		await expect(this.passwordInput).toBeVisible();
		await expect(this.registerButton).toBeVisible();
	}

	async expectEmailConfirmationVisible() {
		await expect(this.emailConfirmationCard).toBeVisible();
		await expect(this.resendEmailButton).toBeVisible();
	}

	async switchToRegister() {
		await this.switchToRegisterLink.click({ force: true });
		// Wait for the register button to appear
		await this.page.waitForTimeout(100);
		await expect(this.registerButton).toBeVisible({ timeout: 10000 });
	}

	async switchToLogin() {
		await this.switchToLoginLink.click({ force: true });
		await this.page.waitForTimeout(100);
		await expect(this.loginButton).toBeVisible({ timeout: 10000 });
	}

	async fillEmail(email: string) {
		await this.emailInput.fill(email);
	}

	async fillPassword(password: string) {
		await this.passwordInput.fill(password);
	}

	async login(email: string, password: string) {
		await this.fillEmail(email);
		await this.fillPassword(password);
		await this.loginButton.click({ force: true });
	}

	async register(email: string, password: string) {
		await this.switchToRegister();
		await this.fillEmail(email);
		await this.fillPassword(password);
		await this.registerButton.click({ force: true });
	}

	async loginAndNavigate(email: string, password: string) {
		await this.login(email, password);
		await this.page.waitForURL("/home");
	}

	async registerAndNavigate(email: string, password: string) {
		await this.register(email, password);
		await this.page.waitForURL("/home");
	}

	async expectErrorMessage(message: string) {
		await expect(this.errorMessage).toContainText(message);
	}

	async expectNoError() {
		await expect(this.errorMessage).not.toBeVisible();
	}
}
