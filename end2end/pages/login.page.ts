import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class LoginPage extends BasePage {
    // Page structure
    readonly loginPage: Locator;
    readonly loginCard: Locator;

    // Form
    readonly loginForm: Locator;
    readonly emailInput: Locator;
    readonly passwordInput: Locator;
    readonly passwordToggle: Locator;
    readonly submitButton: Locator;
    readonly errorAlert: Locator;

    // OAuth
    readonly googleButton: Locator;
    readonly yandexButton: Locator;

    constructor(page: Page) {
        super(page);

        // Page structure
        this.loginPage = page.getByTestId("login-page");
        this.loginCard = page.getByTestId("login-card");

        // Form
        this.loginForm = page.getByTestId("login-form");
        this.emailInput = page.getByTestId("email-input");
        this.passwordInput = page.getByTestId("password-input");
        this.passwordToggle = page.getByTestId("password-input-toggle");
        this.submitButton = page.getByTestId("login-submit");
        this.errorAlert = page.getByTestId("login-form-error");

        // OAuth
        this.googleButton = page.getByTestId("oauth-google");
        this.yandexButton = page.getByTestId("oauth-yandex");
    }

    async goto(): Promise<void> {
        await this.navigate("/login");
    }

    async expectLoginFormVisible(): Promise<void> {
        await expect(this.loginPage).toBeVisible();
        await expect(this.loginCard).toBeVisible();
        await expect(this.loginForm).toBeVisible();
        await expect(this.emailInput).toBeVisible();
        await expect(this.passwordInput).toBeVisible();
        await expect(this.submitButton).toBeVisible();
        await expect(this.googleButton).toBeVisible();
        await expect(this.yandexButton).toBeVisible();
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

    async togglePasswordVisibility(): Promise<void> {
        await this.passwordToggle.click();
    }

    async submit(): Promise<void> {
        await this.submitButton.waitFor({ state: "visible", timeout: 5000 });
        await this.submitButton.click({ force: true });
    }

    async login(
        email: string,
        password: string,
    ): Promise<{ success: boolean; error?: string }> {
        try {
            await this.fillEmail(email);
            await this.page.waitForTimeout(100);
            await this.fillPassword(password);
            await this.page.waitForTimeout(100);
            await this.submit();
            // Wait for navigation to complete after submit
            await this.page.waitForLoadState("networkidle", { timeout: 30_000 });

            return { success: true };
        }
        catch (error) {
            return { success: false, error: error instanceof Error ? error.message : String(error) };
        }
    }

    async expectLoginSuccess(redirectTo: string | string[] = ["/home", "/onboarding"], timeout: number = 60000): Promise<void> {
        const paths = Array.isArray(redirectTo) ? redirectTo : [redirectTo];
        await this.page.waitForURL((url) => {
            const pathname = url.pathname;
            return paths.some(path => pathname.includes(path));
        }, { timeout });
    }

    async expectErrorMessage(): Promise<string | null> {
        await expect(this.errorAlert).toBeVisible();
        return await this.errorAlert.textContent();
    }

    async clickGoogleLogin(): Promise<void> {
        await this.googleButton.click();
    }

    async clickYandexLogin(): Promise<void> {
        await this.yandexButton.click();
    }
}
